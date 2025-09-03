pub mod common_utils;
mod wado_rs_controller;

use crate::wado_rs_controller::{
    echo, manual_hello, retrieve_instance, retrieve_instance_frames, retrieve_series_metadata,
    retrieve_study_metadata,
};
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use common::database_provider::DbProvider;
use common::server_config::LocalStorageConfig;
use common::{database_factory, server_config};
use slog;
use slog::{Drain, Logger, error, info, o};
use slog_async;
use slog_term;
use std::sync::Arc;

fn configure_log() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();

    // It is used for Synchronization
    let console_drain = slog_async::Async::new(console_drain).build().fuse();

    // Root logger
    Logger::root(console_drain, o!("v"=>env!("CARGO_PKG_VERSION")))
}
// 定义应用状态

#[derive(Clone)]
struct AppState {
    log: Logger,
    local_storage_config: LocalStorageConfig,
    db: Arc<dyn DbProvider + Send + Sync>,
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

    // let db_config = config.database.unwrap();
    let server_config = config.server.unwrap();
    let local_storage_config = config.local_storage.unwrap();

    info!(
        log,
        "Starting the server at {}:{}", server_config.host, server_config.port
    );
    info!(log, "LocalStorage Config is: {:?}", local_storage_config);
    let db_provider = database_factory::create_db_instance().await;
    if db_provider.is_none() {
        error!(
            log,
            "Starting the server at {}:{}", server_config.host, server_config.port
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "db_provider is none",
        ));
    }

    let db_instance = db_provider.unwrap();
    let app_state = AppState {
        log: log.clone(),
        local_storage_config: local_storage_config.clone(),
        db: db_instance as Arc<dyn DbProvider + Send + Sync>, // 正确的类型转换
    };

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
