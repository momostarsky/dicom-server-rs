mod common_utils;
mod wado_rs_controller;
mod server_config;

use crate::wado_rs_controller::{
    echo, manual_hello, retrieve_instance, retrieve_instance_frames, retrieve_instance_metadata,
    retrieve_series, retrieve_series_metadata, retrieve_study, retrieve_study_metadata,
};
use actix_web::{web, App, HttpServer};
use slog;
use slog::{info, o, Drain, Logger};
use slog_async;
use slog_term;

fn configure_log()->Logger{
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



    info!(log ,"Starting the server at http://{}:{}",config.server.host,config.server.port);

    HttpServer::new(move || {
        App::new()
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
        .bind((config.server.host, config.server.port))?
        .run()
        .await
}