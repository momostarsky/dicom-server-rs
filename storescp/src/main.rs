extern crate core;

use clap::Parser;
use common::license_manager::validate_client_certificate;
use common::server_config;
use common::utils::{get_logger, setup_logging};
use dicom_core::{dicom_value, DataElement, VR};
use dicom_dictionary_std::tags;
use dicom_encoding::{snafu, TransferSyntaxIndex};
use dicom_object::{InMemDicomObject, StandardDataDictionary};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use slog::{error, info, o};
use snafu::Report;
use std::collections::HashSet;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
};

mod dicom_file_handler;
mod store_async;
mod store_sync;
mod transfer;

use store_async::run_store_async;
use store_sync::run_store_sync;

/// DICOM C-STORE SCP
#[derive(Debug, Parser)]
#[command(version)]
struct App {
    /// Verbose mode
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
    /// Calling Application Entity title
    #[arg(long = "calling-ae-title", default_value = "STORE-SCP")]
    calling_ae_title: String,
    /// Enforce max pdu length
    #[arg(short = 's', long = "strict")]
    strict: bool,
    /// Only accept native/uncompressed transfer syntaxes
    #[arg(long)]
    uncompressed_only: bool,
    /// Accept unknown SOP classes
    #[arg(long)]
    promiscuous: bool,
    /// Maximum PDU length
    #[arg(
        short = 'm',
        long = "max-pdu-length",
        default_value = "16384",
        value_parser(clap::value_parser!(u32).range(4096..=131_072))
    )]
    max_pdu_length: u32,
    /// Output directory for incoming objects
    #[arg(short = 'o', default_value = ".")]
    out_dir: PathBuf,
    /// Which port to listen on
    #[arg(short, default_value = "11111")]
    port: u16,
    /// Run in non-blocking mode (spins up an async task to handle each incoming stream)
    #[arg(short, long, default_value = "true")]
    non_blocking: bool,

    #[arg(short = 'j', long = "json-store-path", default_value = ".")]
    json_store_path: PathBuf,
}


fn create_cstore_response(
    message_id: u16,
    sop_class_uid: &str,
    sop_instance_uid: &str,
) -> InMemDicomObject<StandardDataDictionary> {
    InMemDicomObject::command_from_element_iter([
        DataElement::new(
            tags::AFFECTED_SOP_CLASS_UID,
            VR::UI,
            dicom_value!(Str, sop_class_uid),
        ),
        DataElement::new(tags::COMMAND_FIELD, VR::US, dicom_value!(U16, [0x8001])),
        DataElement::new(
            tags::MESSAGE_ID_BEING_RESPONDED_TO,
            VR::US,
            dicom_value!(U16, [message_id]),
        ),
        DataElement::new(
            tags::COMMAND_DATA_SET_TYPE,
            VR::US,
            dicom_value!(U16, [0x0101]),
        ),
        DataElement::new(tags::STATUS, VR::US, dicom_value!(U16, [0x0000])),
        DataElement::new(
            tags::AFFECTED_SOP_INSTANCE_UID,
            VR::UI,
            dicom_value!(Str, sop_instance_uid),
        ),
    ])
}

fn create_cecho_response(message_id: u16) -> InMemDicomObject<StandardDataDictionary> {
    InMemDicomObject::command_from_element_iter([
        DataElement::new(tags::COMMAND_FIELD, VR::US, dicom_value!(U16, [0x8030])),
        DataElement::new(
            tags::MESSAGE_ID_BEING_RESPONDED_TO,
            VR::US,
            dicom_value!(U16, [message_id]),
        ),
        DataElement::new(
            tags::COMMAND_DATA_SET_TYPE,
            VR::US,
            dicom_value!(U16, [0x0101]),
        ),
        DataElement::new(tags::STATUS, VR::US, dicom_value!(U16, [0x0000])),
    ])
}

#[tokio::main]
async fn main() {
    let log = setup_logging("dicom-store-scp");

    let config = server_config::load_config();
    let config = match config {
        Ok(config) => config,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(-2);
        }
    };

    let client_info = match validate_client_certificate().await {
        Ok(client_info) => {
            info!(
                log,
                "Client Certificate Validated, Client ID: {:?}, HashCode:{:?}",
                client_info.0,
                client_info.1
            );
            client_info
        }
        Err(e) => {
            let error_string = format!("{}", e);
            info!(
                log,
                "Client Certificate Validation Failed: {}", error_string
            );
            std::process::exit(-2);
        }
    };
    let (client_id, hash_code) = client_info;
    // 确保证书中的client_id和hash_code都存在
    let cert_client_id = match client_id {
        Some(id) => id,
        None => {
            info!(log, "Certificate does not contain a valid Client ID");
            std::process::exit(-2);
        }
    };

    let cert_hash_code = match hash_code {
        Some(code) => code,
        None => {
            info!(log, "Certificate does not contain a valid Hash Code");
            std::process::exit(-2);
        }
    };

    let license = match &config.dicom_license_server {
        None => {
            info!(log, "Dicom License Server Config is None");
            std::process::exit(-2);
        }
        Some(license_server) => license_server,
    };
    // 使用更安全的比较方法，避免时序攻击
    let client_id_matches = {
        let expected = &license.client_id;
        openssl::memcmp::eq(expected.as_bytes(), cert_client_id.as_bytes())
    };

    let hash_code_matches = {
        let expected = &license.license_key; // license_key 实际上存储的是 hash_code
        openssl::memcmp::eq(expected.as_bytes(), cert_hash_code.as_bytes())
    };

    if client_id_matches && hash_code_matches {
        info!(log, "License Server Validation Success");
    } else {
        info!(log, "License Server Validation Failed");
        info!(
            log,
            "Expected Client ID: {}, Certificate Client ID: {}", license.client_id, cert_client_id
        );
        info!(
            log,
            "Expected Hash Code: {}, Certificate Hash Code: {}",
            license.license_key,
            cert_hash_code
        );
        std::process::exit(-2);
    }

    let mut app = App::parse();
    let scp_config = config.dicom_store_scp;

    app.port = scp_config.port;

    app.calling_ae_title = scp_config.ae_title;
    let store_cfg = config.local_storage;

    app.out_dir = store_cfg.dicom_store_path.parse().unwrap();
    app.json_store_path = store_cfg.json_store_path.parse().unwrap();

    let out_dir = std::fs::exists(&app.out_dir);
    match out_dir {
        Ok(exists) => {
            if exists {
                info!(log, "Output Directory Exists");
            } else {
                std::fs::create_dir_all(&app.out_dir).unwrap_or_else(|e| {
                    error!(log, "Could not create output directory: {}", e);
                    std::process::exit(-2);
                });
            }
        }
        Err(_) => {
            std::fs::create_dir_all(&app.out_dir).unwrap_or_else(|e| {
                error!(log, "Could not create output directory: {}", e);
                std::process::exit(-2);
            });
        }
    }
    //TODO:  测试 out_dir  是否具有创建目录及写入权限
    test_directory_permissions(&log, &app.out_dir).unwrap_or_else(|e| {
        error!(log, "Directory permission test failed: {}", e);
        std::process::exit(-2);
    });

    let json_dir = std::fs::exists(&app.json_store_path);

    match json_dir {
        Ok(exists) => {
            if exists {
                info!(log, "Json Store Directory Exists");
            } else {
                std::fs::create_dir_all(&app.json_store_path).unwrap_or_else(|e| {
                    error!(log, "Could not create json store directory: {}", e);
                    std::process::exit(-2);
                });
            }
        }
        Err(_) => {
            std::fs::create_dir_all(&app.json_store_path).unwrap_or_else(|e| {
                error!(log, "Could not create json store directory: {}", e);
                std::process::exit(-2);
            });
        }
    }
    test_directory_permissions(&log, &app.json_store_path).unwrap_or_else(|e| {
        error!(log, "Directory permission test failed: {}", e);
        std::process::exit(-2);
    });

    info!(log, "License Server Validation Success");



    match app.non_blocking {
        true => {
            info!(log, "工作在非阻塞模式");
            // 使用已有的tokio运行时
            //可以设置最大并发连接数等参数
            run_async(app).await.unwrap_or_else(|e| {
                error!(log, "{:?}", e);
                std::process::exit(-2);
            });
        }
        false => {
            info!(log, "工作在同步模式");
            // 为同步模式创建专用的运行时
            // 同步模式适用于简单部署或调试场景
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap_or_else(|e| {
                    error!(log, "Could not create tokio runtime: {}", e);
                    std::process::exit(-2);
                });

            std::thread::spawn(move || {
                rt.block_on(async {
                    run_sync(app).await.unwrap_or_else(|e| {
                        error!(log, "{:?}", e);
                        std::process::exit(-2);
                    });
                });
            })
            .join()
            .unwrap();
        }
    }

    // if app.non_blocking {
    //     tokio::runtime::Builder::new_multi_thread()
    //         .enable_all()
    //         .build()
    //         .unwrap()
    //         .block_on(async move {
    //             run_async(app,log.clone()).await.unwrap_or_else(|e| {
    //                 error!(log, "{:?}", e);
    //                 std::process::exit(-2);
    //             });
    //         });
    // } else {
    //     run_sync(app,log.clone()).await.unwrap_or_else(|e| {
    //         error!(log, "{:?}", e);
    //         std::process::exit(-2);
    //     });
    // }
    // match app.non_blocking {
    //     true => {
    //         info!(log, "Non Blocking Mode");
    //         run_sync(app, log.clone()).await.unwrap_or_else(|e| {
    //             error!(log, "{:?}", e);
    //             std::process::exit(-2);
    //         })
    //     }
    //     false => {
    //         match tokio::runtime::Builder::new_multi_thread()
    //             .enable_all()
    //             .build()
    //         {
    //             Ok(rt) => rt.block_on(async move {
    //                 run_async(app, log.clone()).await.unwrap_or_else(|e| {
    //                     error!(log, "{:?}", e);
    //                     std::process::exit(-2);
    //                 })
    //             }),
    //             Err(_) => {
    //                 error!(log, "Could not create tokio runtime");
    //                 std::process::exit(-2);
    //             }
    //         }
    //     }
    // }
}

async fn run_async(args: App) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    let args = Arc::new(args);
    let rlogger = get_logger();
    let logger = rlogger.new(o!("storescp"=>"run_async"));
    let listen_addr = SocketAddrV4::new(Ipv4Addr::from(0), args.port);
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    info!(
        &logger,
        "{} listening on: tcp://{}", &args.calling_ae_title, listen_addr
    );

    loop {
        let (socket, _addr) = listener.accept().await?;
        let args = args.clone();
        let logs = logger.clone();
        tokio::task::spawn(async move {
            if let Err(e) = run_store_async(socket, &args).await {
                error!(logs, "{}", Report::from_error(e));
            }
        });
    }
}

async fn run_sync(args: App) -> Result<(), Box<dyn std::error::Error>> {
    let rlogger = get_logger();
    let logger = rlogger.new(o!("storescp"=>"run_sync"));
    let listen_addr = SocketAddrV4::new(Ipv4Addr::from(0), args.port);
    let listener = std::net::TcpListener::bind(listen_addr)?;
    info!(
        &logger,
        "{} listening on: tcp://{}", &args.calling_ae_title, listen_addr
    );

    for stream in listener.incoming() {
        match stream {
            Ok(scu_stream) => {
                let tcp_logger = logger.clone();
                if let Err(e) = run_store_sync(scu_stream, &args).await {
                    error!(&tcp_logger, "{}", snafu::Report::from_error(e));
                }
            }
            Err(e) => {
                error!(&logger, "{}", snafu::Report::from_error(e));
            }
        }
    }

    Ok(())
}

fn test_directory_permissions(log: &slog::Logger, out_dir: &PathBuf) -> Result<(), std::io::Error> {
    info!(log, "Test Directory: {}", out_dir.display());
    let test_dir = format!("{}/{}", out_dir.display(), "1.222/1.444/1.555");
    std::fs::create_dir_all(&test_dir)?;
    info!(log, "Test Directory: {}  Create Success !", test_dir);

    // 测试写入权限
    let test_file = format!("{}/test.dcm", test_dir);
    std::fs::write(
        &test_file,
        b"903290903234092409383404903409289899889jkkallklkj",
    )?;
    info!(log, "Test File: {}  Create Success !", test_file);

    // 清理测试文件和目录
    std::fs::remove_file(&test_file)?;
    std::fs::remove_dir_all(&test_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::App;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        App::command().debug_assert();
    }
}
