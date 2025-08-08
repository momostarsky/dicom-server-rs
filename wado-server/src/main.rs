mod wado_rs_controller;
pub mod common_utils;

use actix_cors::Cors;
use crate::wado_rs_controller::{
    echo, manual_hello, retrieve_instance, retrieve_instance_frames, retrieve_instance_metadata,
    retrieve_series, retrieve_series_metadata, retrieve_study, retrieve_study_metadata,
};
use actix_web::{web, App, HttpServer};
use slog;
use slog::{info, o, Drain, Logger};
use slog_async;
use slog_term;
use common::server_config;

fn configure_log() ->Logger{
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();

    // It is used for Synchronization
    let console_drain = slog_async::Async::new(console_drain).build().fuse();

    // Root logger
    Logger::root(console_drain,o!("v"=>env!("CARGO_PKG_VERSION")))
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log = configure_log() ;

    let config = server_config::load_config();
    let config = match config {
        Ok(config) => {
            info!(log,"Config: {:?}",config);
            config
        }
        Err(e) => {
            info!(log,"Error loading config: {:?}",e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    };

    // let db_config = config.database.unwrap();
    let server_config = config.server.unwrap();
    // let local_storage_config = config.local_storage.unwrap();

    info!(log ,"Starting the server at {}:{}",server_config.host,server_config.port);

    HttpServer::new(move || {
        let   cors = Cors::default()
            .allow_any_origin() // 🚨 开发环境可用，生产环境不推荐
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(log.clone()))
            .service(retrieve_study)
            .service(retrieve_study_metadata)
            .service(retrieve_series)
            .service(retrieve_series_metadata)
            .service(retrieve_instance)
            .service(retrieve_instance_metadata)
            .service(retrieve_instance_frames)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind((server_config.host, server_config.port))?
        .run()
        .await
}