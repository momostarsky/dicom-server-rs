use common::license_manager::validate_client_certificate;
use common::redis_key::RedisHelper;
use common::server_config::AppConfig;
use common::utils::setup_logging;
use common::{database_factory, server_config};
use database::dicom_dbprovider::DbProvider;
use slog::{Logger, error, info};
use std::sync::Arc;

mod background;
#[derive(Clone)]
struct AppState {
    log: Logger,
    db: Arc<dyn DbProvider + Send + Sync>,
    config: AppConfig,
    redis_helper: RedisHelper,
    // 可以添加其他配置
}
fn configure_log() -> Logger {
    // let decorator = slog_term::TermDecorator::new().build();
    // let console_drain = slog_term::FullFormat::new(decorator).build().fuse();
    //
    // // It is used for Synchronization
    // let console_drain = slog_async::Async::new(console_drain).build().fuse();
    //
    // // Root logger
    // Logger::root(console_drain, o!("v"=>env!("CARGO_PKG_VERSION")))
    let log = setup_logging("wado-webworker");
    log.clone()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("执行后台工作任务:");
    println!(" 1: 生成 WADO-RS 服务需要的study_metadata ");
    println!(" 2: 根据收图日志更新SeriesRelatedInstance 取值");
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

    let db_provider = match database_factory::create_db_instance(&config.main_database).await {
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
    let server_config = config.server;
    let local_storage_config = config.local_storage;
    info!(log, "LocalStorage Config is: {:?}", local_storage_config);

    let reids_conn = g_config.redis.clone();

    let app_state = AppState {
        log: log.clone(),
        db: db_provider as Arc<dyn DbProvider + Send + Sync>, // 正确的类型转换
        config: g_config,
        redis_helper: RedisHelper::new(reids_conn),
    };

    background::background_task_manager(app_state).await;
    Ok(())
}
