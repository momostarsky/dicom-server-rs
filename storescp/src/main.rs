extern crate core;

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
};

use clap::Parser;
use common::ca_helper::client_register;
use common::{cert_helper, server_config};
use dicom_core::{dicom_value, DataElement, VR};
use dicom_dictionary_std::tags;
use dicom_encoding::snafu;
use dicom_object::{InMemDicomObject, StandardDataDictionary};
use slog::{error, info, o, Drain, Logger};
use snafu::Report;
use tracing::Level;

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
fn configure_log() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();

    // It is used for Synchronization
    let console_drain = slog_async::Async::new(console_drain).build().fuse();

    // Root logger
    Logger::root(console_drain, o!("v"=>env!("CARGO_PKG_VERSION")))
}
#[tokio::main]
async fn main() {
    let log = configure_log();
    let machine_id = cert_helper::read_machine_id();
    info!(log, "Machine ID: {:?}", machine_id);

    let config = server_config::load_config();
    let config = match config {
        Ok(config) => config,
        Err(e) => {
            println!("{:?}", e);
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
    info!(
        log,
        "Config License Server License Server URL: {:?}", license.url
    );
    info!(
        log,
        "Config License Server Machine ID: {:?}", license.machine_id
    );
    info!(
        log,
        "Config License Server Mac Address: {:?}", license.mac_address
    );
    info!(
        log,
        "Config License Server Client ID: {:?}", license.client_id
    );
    info!(
        log,
        "Config License Server Client Name : {:?}", license.client_name
    );
    info!(
        log,
        "Config License Server End Date: {:?}", license.end_date
    );
    match std::fs::exists(&license.license_key.as_str()) {
        Ok(true) => {
            info!(log, "License Key File Exists");
        }
        Ok(false) => match client_register(&license, &license.url).await {
            Ok(_) => {
                info!(log, "Client Register Success");
            }
            Err(e) => {
                error!(log, "Client Register Error: {:?}", e);
                std::process::exit(-2);
            }
        },
        _ => {
            error!(log, "客户端授权证书错误 Key File Error");
            std::process::exit(-2);
        }
    };
    //
    //  match cert_helper::validate_client_certificate_only(&license.license_key,"./dicom-org-cn.pem") {
    match cert_helper::validate_client_certificate_only(&license.license_key) {
        Ok(_) => {
            info!(log, "Validate My Certificate Success");
            info!(log, "✅ 证书验证成功");
        }
        Err(e) => {
            error!(log, "Validate My Certificate Error: {:?}", e);
            std::process::exit(-2);
        }
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

    match app.non_blocking {
        true => {
            info!(log, "工作在非阻塞模式");
            // 使用已有的tokio运行时
            //可以设置最大并发连接数等参数
            run_async(app, log.clone()).await.unwrap_or_else(|e| {
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
                    run_sync(app, log.clone()).await.unwrap_or_else(|e| {
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

async fn run_async(args: App, logger: Logger) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    let args = Arc::new(args);
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(if args.verbose {
                Level::DEBUG
            } else {
                Level::INFO
            })
            .finish(),
    )
    .unwrap_or_else(|e| {
        eprintln!(
            "Could not set up global logger: {}",
            snafu::Report::from_error(e)
        );
    });

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

async fn run_sync(args: App, logger: Logger) -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(if args.verbose {
                Level::DEBUG
            } else {
                Level::INFO
            })
            .finish(),
    )
    .unwrap_or_else(|e| {
        eprintln!(
            "Could not set up global logger: {}",
            snafu::Report::from_error(e)
        );
    });

    let listen_addr = SocketAddrV4::new(Ipv4Addr::from(0), args.port);
    let listener = std::net::TcpListener::bind(listen_addr)?;
    info!(
        &logger,
        "{} listening on: tcp://{}", &args.calling_ae_title, listen_addr
    );

    for stream in listener.incoming() {
        match stream {
            Ok(scu_stream) => {
                if let Err(e) = run_store_sync(scu_stream, &args).await {
                    error!(&logger, "{}", snafu::Report::from_error(e));
                }
            }
            Err(e) => {
                error!(&logger, "{}", snafu::Report::from_error(e));
            }
        }
    }

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
