use common::redis_key::RedisHelper;
use common::server_config::AppConfig;
use common::utils::setup_logging;
use common::{database_factory, server_config};
use database::dicom_dbprovider::DbProvider;
use slog::{Logger, error, info};
use std::sync::Arc;

mod json_creator;
#[derive(Clone)]
struct AppState {
    log: Logger,
    db: Arc<dyn DbProvider + Send + Sync>,
    config: AppConfig,
    #[allow(dead_code)]
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
    let local_storage_config = config.local_storage;
    info!(log, "LocalStorage Config is: {:?}", local_storage_config);

    let reids_conn = g_config.redis.clone();

    let app_state = AppState {
        log: log.clone(),
        db: db_provider as Arc<dyn DbProvider + Send + Sync>, // 正确的类型转换
        config: g_config,
        redis_helper: RedisHelper::new(reids_conn),
    };

    json_creator::background_task_manager(app_state).await;
    Ok(())
}
