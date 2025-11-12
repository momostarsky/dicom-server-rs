pub mod common_utils;

mod auth_middleware_kc;
mod constants;
mod wado_rs_controller_v1;
mod wado_rs_models;

// use crate::wado_rs_controller_v1::{
//     echo_v1, retrieve_instance, retrieve_instance_frames, retrieve_series_metadata,
//     retrieve_study_metadata, retrieve_study_subseries,
// };
use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, middleware, web};

// use crate::auth_middleware_kc::AuthMiddleware;
use crate::auth_middleware_kc::{AuthMiddleware, update_jwks_task};
use crate::constants::WADO_RS_CONTEXT_PATH;
use common::license_manager::validate_client_certificate;
use common::redis_key::RedisHelper;
use common::server_config::AppConfig;
use common::utils::setup_logging;
use common::{database_factory, server_config};
use database::dicom_dbprovider::DbProvider;
use slog;
use slog::{Logger, error, info};
use std::sync::Arc;
use utoipa_actix_web::{AppExt, scope};
use utoipa_swagger_ui::SwaggerUi;

// use crate::auth_middleware::AuthMiddleware;
// 将原来的简单结构体定义替换为完整的 OpenApi 配置

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
// #[derive(OpenApi)]
// #[openapi(
//     tags((name = "WADO-RS", description = "WADO-RS API接口")),
//     paths(
//         wado_rs_controller::retrieve_study_metadata,
//         wado_rs_controller::retrieve_study_subseries,
//         wado_rs_controller::retrieve_series_metadata,
//         wado_rs_controller::retrieve_instance,
//         wado_rs_controller::retrieve_instance_frames,
//         wado_rs_controller::echo_v1,
//         wado_rs_controller::echo_v2,
//     )
// )]
// struct ApiDoc;
#[derive(Clone)]
struct AppState {
    log: Logger,
    db: Arc<dyn DbProvider + Send + Sync>,
    config: AppConfig,
    redis_helper: RedisHelper,
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
    let oauth_config = config.wado_oauth2;
    let reids_conn = g_config.redis.clone();

    let app_state = AppState {
        log: log.clone(),
        db: db_provider as Arc<dyn DbProvider + Send + Sync>, // 正确的类型转换
        config: g_config,
        redis_helper: RedisHelper::new(reids_conn),
    };

    // 在创建app_state之后，启动服务器之前添加以下代码
    if oauth_config.is_some() {
        info!(log, "OAuth2 Config is: {:?}", oauth_config);
        let oauth2_app_state = app_state.clone();
        tokio::spawn(async move {
            update_jwks_task(oauth2_app_state).await;
        });
    }

    info!(
        log,
        "Starting the server at {}:{}", server_config.host, server_config.port
    );

    HttpServer::new(move || {
        // 创建统一的 CORS 配置
        let cors_config = || {
            Cors::default()
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![
                    http::header::AUTHORIZATION,
                    http::header::ACCEPT,
                    http::header::CONTENT_TYPE,
                    http::header::REFERER,
                    http::header::USER_AGENT,
                    http::header::HeaderName::from_static("x-tenant"),
                ])
                .supports_credentials()
                .max_age(3600)
        };

        // 根据配置设置允许的 origin
        let configure_origin = |cors: Cors| -> Cors {
            if !server_config.allow_origin.is_empty() {
                let mut cors = cors;
                for origin in &server_config.allow_origin {
                    if origin == "*" {
                        cors = cors.allow_any_origin();
                        break;
                    } else {
                        cors = cors.allowed_origin(origin);
                    }
                }
                cors
            } else {
                cors.allowed_origin_fn(|origin, _req_head| {
                    origin.as_bytes().starts_with(b"http://localhost")
                        || origin.as_bytes().starts_with(b"http://127.0.0.1")
                })
            }
        };

        let cors = configure_origin(cors_config());
        let wado_rs_cors = configure_origin(cors_config());

        let (app, mut api) = App::new()
            .into_utoipa_app()
            .service(
                scope::scope(WADO_RS_CONTEXT_PATH)
                    .wrap(wado_rs_cors)
                    .service(
                        scope::scope("/v1")
                            // 关闭权限验证
                            .wrap(AuthMiddleware {
                                logger: app_state.log.clone(),
                                redis: app_state.redis_helper.clone(),
                                config: app_state.config.clone(),
                            })
                            .service(wado_rs_controller_v1::retrieve_study_metadata)
                            .service(wado_rs_controller_v1::retrieve_study_subseries)
                            .service(wado_rs_controller_v1::retrieve_series_metadata)
                            .service(wado_rs_controller_v1::retrieve_instance)
                            .service(wado_rs_controller_v1::retrieve_instance_frames),
                    )
                    .service(scope::scope("/v1").service(wado_rs_controller_v1::echo_v1)),
            )
            .split_for_parts();
        api.info.title = "WADO API".to_string();
        app.wrap(middleware::Compress::default())
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind((server_config.host, server_config.port))?
    .run()
    .await
}
pub(crate) async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
