mod register_controller;
mod cert_helper;

use crate::register_controller::{client_registe_get, client_registe_post, client_validate, manual_hello};
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};

use slog::{error, info, Drain};
use slog::{Logger, o};
use std::fs;
// 定义应用状态

#[derive(Clone)]
struct AppState {
    log: Logger,
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log = configure_log();
    let app_state = AppState { log: log.clone() };
     info!(log, "Starting server... 8888");
    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        cors = cors.allow_any_origin();

        App::new()
            // 使用.wrap()方法添加Compress中间件
            .wrap(middleware::Compress::default())
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .service(client_registe_get)
            .service(client_registe_post)
            .service(client_validate)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8888))?
    .run()
    .await
}


fn configure_log() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();

    // It is used for Synchronization
    let console_drain = slog_async::Async::new(console_drain).build().fuse();

    // Root logger
    Logger::root(console_drain, o!("v"=>env!("CARGO_PKG_VERSION")))
}
