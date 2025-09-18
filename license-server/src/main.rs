mod register_controller;

use crate::register_controller::{client_registe_get, client_registe_post, get_ca_certificate, manual_hello};
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};

use common::utils::setup_logging;
use slog::Logger;
use slog::info;
use common::cert_helper::generate_ca_root;
// 定义应用状态

#[derive(Clone)]
struct AppState {
    log: Logger,
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    generate_ca_root("./ca_root.pem","./ca_key_root.pem","./server.pem", "./server.key").expect("生成CA成功");
    let clog: Logger = configure_log();
    let app_state = AppState { log: clog.clone() };
    info!(clog, "Starting server... 8888");
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
            .service(get_ca_certificate)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8888))?
    .run()
    .await
}

fn configure_log() -> Logger {
    let log = setup_logging("license-server");
    info!(log, "License server started");
    log.clone()
}
