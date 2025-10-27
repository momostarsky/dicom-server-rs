pub mod common_utils;
mod redis_helper;
mod wado_rs_controller;

use crate::wado_rs_controller::{
    echo, manual_hello, retrieve_instance, retrieve_instance_frames, retrieve_series_metadata,
    retrieve_study_metadata,
};
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use common::database_provider::DbProvider;
use common::license_manager::validate_client_certificate;
use common::server_config::AppConfig;
use common::utils::setup_logging;
use common::{database_factory, server_config};
use slog;
use slog::{ Logger, error, info};
use std::sync::Arc;

fn configure_log() -> Logger {
    // let decorator = slog_term::TermDecorator::new().build();
    // let console_drain = slog_term::FullFormat::new(decorator).build().fuse();
    //
    // // It is used for Synchronization
    // let console_drain = slog_async::Async::new(console_drain).build().fuse();
    //
    // // Root logger
    // Logger::root(console_drain, o!("v"=>env!("CARGO_PKG_VERSION")))
    let log = setup_logging("wado-server");
    info!(log, "Wado server started");
    log.clone()
}
// 定义应用状态

#[derive(Clone)]
struct AppState {
    log: Logger,
    db: Arc<dyn DbProvider + Send + Sync>,
    config: AppConfig,
    // 可以添加其他配置
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log = configure_log();
    let config = server_config::load_config();
    let config = match config {
        Ok(config) => {
            info!(log, "Config: {:?}", config);
            config
        }
        Err(e) => {
            info!(log, "Error loading config: {:?}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
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
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error_string));
        }
    };
    let (client_id, hash_code) = client_info;
    // 确保证书中的client_id和hash_code都存在
    let cert_client_id = match client_id {
        Some(id) => id,
        None => {
            info!(log, "Certificate does not contain a valid Client ID");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid Client ID in certificate",
            ));
        }
    };

    let cert_hash_code = match hash_code {
        Some(code) => code,
        None => {
            info!(log, "Certificate does not contain a valid Hash Code");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid Hash Code in certificate",
            ));
        }
    };

    let license = match &config.dicom_license_server {
        None => {
            info!(log, "Dicom License Server Config is None");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Dicom License Server Config is None",
            ));
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
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "License Server Validation Failed",
        ));
    }
    // info!(
    //     log,
    //     "Config License Server License Server URL: {:?}", license.url
    // );
    // info!(
    //     log,
    //     "Config License Server Machine ID: {:?}", license.machine_id
    // );
    // info!(
    //     log,
    //     "Config License Server Mac Address: {:?}", license.mac_address
    // );
    // info!(
    //     log,
    //     "Config License Server Client ID: {:?}", license.client_id
    // );
    // info!(
    //     log,
    //     "Config License Server Client Name : {:?}", license.client_name
    // );
    // info!(
    //     log,
    //     "Config License Server End Date: {:?}", license.end_date
    // );
    // match std::fs::exists(&license.license_key.as_str()) {
    //     Ok(true) => {
    //         info!(log, "License Key File Exists");
    //     }
    //     Ok(false) => {
    //         return Err(std::io::Error::new(
    //             std::io::ErrorKind::Other,
    //             format!("License Key File Not Exists: {:?}", &license.license_key),
    //         ));
    //     }
    //     _ => {
    //         return Err(std::io::Error::new(
    //             std::io::ErrorKind::Other,
    //             format!("客户端授权证书错误: {:?}", &license.license_key),
    //         ));
    //     }
    // };
    // //
    // //  match cert_helper::validate_client_certificate_only(&license.license_key,"./dicom-org-cn.pem") {
    // match cert_helper::validate_client_certificate_with_ca(&license.license_key,"./dicom-org-cn.pem") {
    //     Ok(_) => {
    //         info!(log, "Validate My Certificate Success");
    //         info!(log, "✅ 证书验证成功");
    //     }
    //     Err(e) => {
    //         error!(log, "Validate My Certificate Error: {:?}", e);
    //         return Err(std::io::Error::new(
    //             std::io::ErrorKind::Other,
    //             format!("Validate My Certificate Error: {:?}", e),
    //         ));
    //     }
    // }


    let db_provider =match database_factory::create_db_instance(&config.main_database).await{
        Ok(db_provider) => db_provider,
        Err(e) => {
            error!(log, "create_db_instance error: {:?}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("create_db_instance error: {:?}", e),
            ));
        }
    };

    let g_config = config.clone();
    // let db_config = config.database.unwrap();
    let server_config = config.server;
    let local_storage_config = config.local_storage;
    info!(log, "LocalStorage Config is: {:?}", local_storage_config);

    let app_state = AppState {
        log: log.clone(),
        db: db_provider as Arc<dyn DbProvider + Send + Sync>, // 正确的类型转换
        config: g_config,
    };
    info!(
        log,
        "Starting the server at {}:{}", server_config.host, server_config.port
    );
    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        // 根据配置设置允许的origin
        if !server_config.allow_origin.is_empty() {
            for origin in &server_config.allow_origin {
                if origin == "*" {
                    cors = cors.allow_any_origin();
                    break;
                } else {
                    cors = cors.allowed_origin(origin);
                }
            }
        } else {
            // 如果没有配置，则默认只允许localhost（保持原有行为）
            cors = cors.allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"http://localhost")
            });
        }

        App::new()
            // 使用.wrap()方法添加Compress中间件
            .wrap(middleware::Compress::default())
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .service(retrieve_study_metadata)
            .service(retrieve_series_metadata)
            .service(retrieve_instance)
            .service(retrieve_instance_frames)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind((server_config.host, server_config.port))?
    .run()
    .await
}
