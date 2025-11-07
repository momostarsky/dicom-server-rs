// auth_middleware.rs
use actix_web::body::{EitherBody, MessageBody};
use actix_web::{
    Error, HttpResponse, ResponseError,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::TryFutureExt;
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, jwk};
use redis::{Commands, RedisError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use slog::{Logger, error, info};
use std::future::{Ready, ready};
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    iss: String,
    aud: serde_json::Value, // 可能是字符串或数组
    exp: usize,
    // 其他字段可选
}

#[derive(Debug, thiserror::Error)]
enum AuthError {
    #[error("Missing authorization header")]
    MissingAuthHeader,
    #[error("Invalid bearer token format")]
    InvalidTokenFormat,
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("HTTP fetch error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Actix web error: {0}")]
    Actix(#[from] actix_web::Error), // 添加这一行
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Audience mismatch")]
    AudienceMismatch,
    #[error("Issuer mismatch")]
    IssuerMismatch,
}
impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::MissingAuthHeader | AuthError::InvalidTokenFormat => {
                HttpResponse::Unauthorized().json("Unauthorized")
            }
            AuthError::Jwt(_)
            | AuthError::Http(_)
            | AuthError::Io(_)
            | AuthError::Actix(_) // 添加这一行
            | AuthError::Redis(_)
            | AuthError::AudienceMismatch
            | AuthError::IssuerMismatch => HttpResponse::Unauthorized().json("Invalid token"),
        }
    }
}
const ISSUER_URL: &str = "https://keycloak.medical.org:8443/realms/dicom-org-cn";
const AUDIENCE: &str = "wado-rs-api";
const SERVER_PORT: u16 = 8080;
// const JWKS_URL: &str =    "https://keycloak.medical.org:8443/realms/dicom-org-cn/protocol/openid-connect/certs";
const JWKS_URL: &str = "https://127.0.0.1:8443/realms/dicom-org-cn/protocol/openid-connect/certs";

#[derive(Debug)]
pub struct AuthMiddleware {
    pub(crate) redis: RedisHelper, // 添加这一行
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>; // 修改为与Service实现一致
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            redis_helper: self.redis.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    redis_helper: RedisHelper, // 添加这一行
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let redis_helper = self.redis_helper.clone();
        let service = self.service.clone();
        let jwks_uri = redis_helper
            .get_jwks_url_content().unwrap();
        println!("Received jwk_urs_content: {}", jwks_uri);
        Box::pin(async move {
            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        println!("Received token: {}", token);
                        // 修改为支持RS256的验证代码：
                        let jwks: serde_json::Value = serde_json::from_str(&jwks_uri)?;
                        let decoding_key = DecodingKey::from_rsa_components(
                            &jwks["keys"][0]["n"].as_str().unwrap(),
                            &jwks["keys"][0]["e"].as_str().unwrap()
                        ).unwrap();
                        let mut validation = Validation::new(Algorithm::RS256);
                        validation.set_issuer(&[ISSUER_URL]);
                        validation.set_audience(&[AUDIENCE]);

                        return match decode::<Claims>(token, &decoding_key, &validation) {
                            Ok(_) => {
                                // Token有效，继续处理请求
                                let res =
                                    service.call(req).await.map_err(actix_web::Error::from)?;
                                Ok(res.map_into_left_body())
                            }
                            Err(_) => {
                                // Token无效
                                let response = HttpResponse::Unauthorized().body("Invalid token");
                                let res = req.into_response(
                                    response.map_into_boxed_body().map_into_right_body(),
                                );
                                Ok(res)
                            }
                        };
                    }
                }
            }

            // 缺少Authorization头或格式不正确
            let response =
                HttpResponse::Unauthorized().body("Missing or invalid Authorization header");
            let res = req.into_response(response.map_into_boxed_body().map_into_right_body());
            Ok(res)
        })
    }
}

use common::redis_key::RedisHelper;
use tokio::time::{Duration, interval};

pub(crate) async fn update_jwks_task(redis_helper: RedisHelper, log: Logger) {
    let mut interval = interval(Duration::from_secs(600)); // 10分钟 = 600秒

    loop {
        interval.tick().await;

        match fetch_and_store_jwks(&redis_helper, &log).await {
            Ok(_) => {
                info!(log, "JWKS更新成功");
            }
            Err(e) => {
                error!(log, "JWKS更新失败: {:?}", e);
            }
        }
    }
}

async fn fetch_and_store_jwks(
    redis_helper: &RedisHelper,
    log: &Logger,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .build()?;

    let response = client
        .get("https://keycloak.medical.org:8443/realms/dicom-org-cn/protocol/openid-connect/certs")
        .header("User-Agent", "dicom-wado-server/1.0") // 添加User-Agent
        .send()
        .await?;

    // 验证响应状态码
    if !response.status().is_success() {
        return Err(format!("HTTP请求失败: {}", response.status()).into());
    }

    let txt = response.text().await?;

    // 验证JSON格式
    let jwks: serde_json::Value =
        serde_json::from_str(&txt).map_err(|e| format!("JSON格式无效: {}", e))?;

    // 验证JWKS结构
    if !jwks.is_object() || !jwks["keys"].is_array() {
        return Err("JWKS格式无效".into());
    }

    println!("JWS-KEYTEXT: {}", txt);

    // 将JWKS内容存储到Redis中
    let _ = redis_helper.set_jwks_url_content(txt, 6000); // 设置10分钟过期时间

    Ok(())
}
