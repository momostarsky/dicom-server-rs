mod client_register;
mod register_controller;
mod handlers;

use crate::register_controller::{
    client_registe_get, client_registe_post, get_ca_certificate, manual_hello,
};
use actix_cors::Cors;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::{App, HttpServer, middleware, web};
use rust_embed::RustEmbed;
use common::utils::setup_logging;
use slog::Logger;
use slog::info;
// 定义应用状态

#[derive(Clone)]
struct AppState {
    log: Logger,
}

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let clog: Logger = configure_log();
    let app_state = AppState { log: clog.clone() };
    info!(clog, "Starting server... 8888");

    use std::env;

    // 在 main 函数中
    let current_dir = env::current_dir().expect("Failed to get current directory");
    info!(
        &clog,
        "Starting server at current work directory: {}",
        current_dir.display()
    );
    let static_dir = format!("{}/static", current_dir.as_path().to_str().unwrap());
    info!(&clog, "License Static File Directory Is : {}", static_dir);
    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        cors = cors.allow_any_origin();
        // use actix_files as fs;

        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::from(&[0; 64]), // 生产环境需用随机安全密钥
            ))
            // 使用.wrap()方法添加Compress中间件
            .wrap(middleware::Compress::default())
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            // .service(fs::Files::new("/static", static_dir.as_str()).show_files_listing())
            .service(client_registe_get)
            .service(client_registe_post)
            .service(get_ca_certificate)
            .route(
                "/create",
                web::get().to(client_register::show_register_form),
            ) // 显示表单页面
            .route("/", web::get().to(client_register::show_login_form)) // 显示表单页面
            .route("/login", web::post().to(client_register::handle_login_post)) // 显示表单页面
            .route(
                "/register",
                web::post().to(client_register::handle_form_post),
            ) // 处理表单提交
            .route("/list", web::get().to(client_register::show_list_form))
            .route("/captcha", web::get().to(client_register::refresh_captcha))
            .route("/static/{filename:.*}", web::get().to(handlers::static_files))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8888))?
    .run()
    .await
}

fn configure_log() -> Logger {
    let log = setup_logging("wado-license");
    info!(log, "License server started");
    log.clone()
}
