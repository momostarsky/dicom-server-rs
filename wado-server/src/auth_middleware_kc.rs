// auth_middleware_keycloak
use actix_web::body::{EitherBody, MessageBody};
use actix_web::{
    Error, HttpMessage, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};

use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use slog::{Logger, error, info};
use std::future::{Ready, ready};
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Claims {
    iss: String,            //签发方（issuer），明确这个 JWT 是哪个认证系统生成的	必须（标准）
    sub: Option<String>,    //主题（subject），指用户唯一标识（通常为用户 ID）	必须（标准）
    aud: Value, //受众（audience），JWT 颁发给哪个客户端/应用	必须（强烈建议)
    exp: usize,             //过期时间（expiration），用于 token 有效期控制	必须（强烈建议）
    email: Option<String>,
    name: Option<String>,
    username: Option<String>,
    preferred_username: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    // // pub(crate) realm_access: Option<RealmAccess>, // realm 级别权限
    // // pub(crate) resource_access: Option<std::collections::HashMap<String, ResourceAccess>>, // 资源级别权限
    // pub(crate) scope: Option<String>, // 权限范围
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct RealmAccess {
    pub(crate) roles: Option<Vec<String>>, // realm 角色
}
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ResourceAccess {
    pub(crate) roles: Option<Vec<String>>, // 资源角色
}

// #[derive(Debug, thiserror::Error)]
// enum AuthError {
//     #[error("Missing authorization header")]
//     MissingAuthHeader,
//     #[error("Invalid bearer token format")]
//     InvalidTokenFormat,
//     #[error("JWT error: {0}")]
//     Jwt(#[from] jsonwebtoken::errors::Error),
//     #[error("HTTP fetch error: {0}")]
//     Http(#[from] reqwest::Error),
//     #[error("File I/O error: {0}")]
//     Io(#[from] std::io::Error),
//     #[error("Actix web error: {0}")]
//     Actix(#[from] Error), // 添加这一行
//     #[error("Redis error: {0}")]
//     Redis(#[from] RedisError),
//     #[error("Audience mismatch")]
//     AudienceMismatch,
//     #[error("Issuer mismatch")]
//     IssuerMismatch,
// }
// impl ResponseError for AuthError {
//     fn error_response(&self) -> HttpResponse {
//         match self {
//             AuthError::MissingAuthHeader | AuthError::InvalidTokenFormat => {
//                 HttpResponse::Unauthorized().json("Unauthorized")
//             }
//             AuthError::Jwt(_)
//             | AuthError::Http(_)
//             | AuthError::Io(_)
//             | AuthError::Actix(_) // 添加这一行
//             | AuthError::Redis(_)
//             | AuthError::AudienceMismatch
//             | AuthError::IssuerMismatch => HttpResponse::Unauthorized().json("Invalid token"),
//         }
//     }
// }
// const ISSUER_URL: &str = "https://keycloak.medical.org:8443/realms/dicom-org-cn";
// const AUDIENCE: &str = "wado-rs-api";
// const SERVER_PORT: u16 = 8080;
// // const JWKS_URL: &str =    "https://keycloak.medical.org:8443/realms/dicom-org-cn/protocol/openid-connect/certs";
// const JWKS_URL: &str = "https://127.0.0.1:8443/realms/dicom-org-cn/protocol/openid-connect/certs";
#[allow(dead_code)]
#[derive(Debug)]
pub struct AuthMiddleware {
    pub(crate) logger: Logger,
    pub(crate) redis: RedisHelper, // 添加这一行
    pub(crate) config: AppConfig,  // 添加这一行
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
            config: self.config.clone(),
            log: self.logger.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    redis_helper: RedisHelper, // 添加这一行
    config: AppConfig,
    log: Logger,
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
        let service = self.service.clone();
        let log = self.log.clone();
        let gconfig = self.config.clone();

        // 如果没有配置 OAuth2，则直接跳过认证
        if gconfig.wado_oauth2.is_none() {
            info!(log, "wado_oauth2 is not configured, skip authentication");
            return Box::pin(async move {
                let res = service.call(req).await.map_err(actix_web::Error::from)?;
                Ok(res.map_into_left_body())
            });
        }
        let redis_helper = self.redis_helper.clone();
        let oauth2_cfg = gconfig.wado_oauth2.unwrap().clone();
        let issuer_url = oauth2_cfg.issuer_url;
        let audience = oauth2_cfg.audience;

        let role_mapping = oauth2_cfg.roles.clone();
        let permission_mapping = oauth2_cfg.permissions.clone();

        if role_mapping.is_none() && permission_mapping.is_none() {
            error!(
                log,
                "role_mapping and permission_mapping are not configured, skip authentication"
            );
            return Box::pin(async move {
                let res = service.call(req).await.map_err(actix_web::Error::from)?;
                Ok(res.map_into_left_body())
            });
        }

        // 修改为模式匹配方式：
        let jwks_uri_content = match redis_helper.get_jwks_url_content() {
            Ok(content) => {
                info!(log, "get_jwks_url_content success");
                info!(log, "Received jwk_urs_content: {}", content);
                content
            }
            Err(e) => {
                error!(log, "Failed to get JWKS content from Redis: {:?}", e);
                // 返回认证不通过的响应

                let response =
                    HttpResponse::Unauthorized().body("Authentication failed: JWKS not available");
                let res = req.into_response(response.map_into_boxed_body().map_into_right_body());
                return Box::pin(async move { Ok(res) });
            }
        };

        Box::pin(async move {
            // 在 async 块内部处理所有可能的错误
            let auth_header = req.headers().get("Authorization");
            if auth_header.is_none() {
                info!(log, "Request Header  Authorization:{}", "None");
                let response = HttpResponse::Unauthorized()
                    .body("Authentication failed: no Authorization header");
                let res = req.into_response(response.map_into_boxed_body().map_into_right_body());
                return Ok(res);
            }

            let auth_str = auth_header.unwrap().to_str();
            if auth_str.is_err() {
                info!(
                    log,
                    "Authentication failed: Authorization header is not a valid string"
                );
                let response = HttpResponse::Unauthorized()
                    .body("Authentication failed: Authorization header is not a valid string");
                let res = req.into_response(response.map_into_boxed_body().map_into_right_body());
                return Ok(res);
            }

            let auth_str = auth_str.unwrap();
            if !auth_str.starts_with("Bearer ") {
                info!(
                    log,
                    "Authentication failed: Authorization header is not start with Bearer !"
                );
                let response = HttpResponse::Unauthorized()
                    .body("Authentication failed: Authorization header is not start with Bearer !");
                let res = req.into_response(response.map_into_boxed_body().map_into_right_body());
                return Ok(res);
            }

            let token = &auth_str[7..];
            info!(log, "Received AccessToken:{}", token);

            let jwks: Value = match serde_json::from_str(&jwks_uri_content) {
                Ok(jwks) => jwks,
                Err(_) => {
                    info!(log, "Invalid JWKS format");
                    let response = HttpResponse::Unauthorized().body("Invalid JWKS format");
                    let res =
                        req.into_response(response.map_into_boxed_body().map_into_right_body());
                    return Ok(res);
                }
            };
            info!(log, "JWKS loaded:{}", jwks);

            let n = match jwks["keys"][0]["n"].as_str() {
                Some(n) => n,
                None => {
                    info!(log, "Invalid RSA key format,keys[0][n] is missing");
                    let response = HttpResponse::Unauthorized().body("Invalid RSA key format");
                    let res =
                        req.into_response(response.map_into_boxed_body().map_into_right_body());
                    return Ok(res);
                }
            };

            let e = match jwks["keys"][0]["e"].as_str() {
                Some(e) => e,
                None => {
                    info!(log, "Invalid RSA key format, keys[0][e] is missing");
                    let response = HttpResponse::Unauthorized().body("Invalid RSA key format");
                    let res =
                        req.into_response(response.map_into_boxed_body().map_into_right_body());
                    return Ok(res);
                }
            };

            let decoding_key = match DecodingKey::from_rsa_components(n, e) {
                Ok(key) => key,
                Err(_) => {
                    info!(
                        log,
                        "Invalid RSA key,DecodingKey::from_rsa_components failed"
                    );
                    let response = HttpResponse::Unauthorized().body("Invalid RSA key");
                    let res =
                        req.into_response(response.map_into_boxed_body().map_into_right_body());
                    return Ok(res);
                }
            };
            let expected_alg = jwks["keys"][0]["alg"].as_str().unwrap_or("RS256");
            let algorithm = match expected_alg {
                "RS256" => Algorithm::RS256,
                "RS384" => Algorithm::RS384,
                "RS512" => Algorithm::RS512,
                _ => Algorithm::RS256,
            };
            let mut validation = Validation::new(algorithm);
            // let mut validation = Validation::new(Algorithm::RS256);
            validation.set_issuer(&[issuer_url]);
            validation.set_audience(&[audience]);

            info!(log, "系统策略");
            info!(log, "用 Realm Roles 表达“身份”（Who you are）");
            info!(
                log,
                "用 Client Roles 表达“权限”（What you can do in this app）"
            );
            match decode::<Claims>(token, &decoding_key, &validation) {
                Ok(token_data) => {
                    // Token有效（包括未过期），继续处理请求
                    let claims = token_data.claims;
                    // 将用户信息存储在请求扩展中，供后续权限检查使用
                    req.extensions_mut().insert(claims.clone());

                    info!(log, "Claims iss:{}", claims.iss);
                    info!(log, "Claims sub:{:?}", claims.sub);
                    info!(log, "Claims email:{:?}", claims.email);
                    info!(log, "Claims name:{:?}", claims.name);
                    info!(log, "Claims username:{:?}", claims.username);
                    info!(
                        log,
                        "Claims preferred_username:{:?}", claims.preferred_username
                    );
                    info!(log, "Claims given_name:{:?}", claims.given_name);
                    info!(log, "Claims family_name:{:?}", claims.family_name);

                    // 在调用 validate_user_permissions 时使用转换后的变量
                    if !validate_user_permissions(&req, &role_mapping, &permission_mapping, &log) {
                        let response = HttpResponse::Forbidden().body("Insufficient permissions");
                        let res =
                            req.into_response(response.map_into_boxed_body().map_into_right_body());
                        return Ok(res);
                    }

                    let res = service.call(req).await.map_err(actix_web::Error::from)?;
                    Ok(res.map_into_left_body())
                }
                Err(_) => {
                    // Token无效（可能包括过期、签名错误等）
                    let response = HttpResponse::Unauthorized().body("Invalid token");
                    let res =
                        req.into_response(response.map_into_boxed_body().map_into_right_body());
                    Ok(res)
                }
            }
        })
    }
}

use crate::AppState;
use common::redis_key::RedisHelper;
use common::server_config::{AppConfig, RoleRule};
use tokio::time::{Duration, interval};

pub(crate) async fn update_jwks_task(app_state: AppState) {
    let mut interval = interval(Duration::from_secs(600)); // 10分钟 = 600秒

    let jwks_url = app_state.config.wado_oauth2.unwrap().jwks_url;
    loop {
        interval.tick().await;

        match fetch_and_store_jwks(&app_state.redis_helper, &app_state.log, jwks_url.clone()).await
        {
            Ok(_) => {
                info!(app_state.log, "JWKS更新成功");
            }
            Err(e) => {
                error!(app_state.log, "JWKS更新失败: {:?}", e);
            }
        }
    }
}

async fn fetch_and_store_jwks(
    redis_helper: &RedisHelper,
    log: &Logger,
    jwks_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .build()?;

    let response = client
        .get(&jwks_url)
        .header("User-Agent", "dicom-wado-server/1.0") // 添加User-Agent
        .send()
        .await?;

    // 验证响应状态码
    if !response.status().is_success() {
        return Err(format!("HTTP请求失败: {}", response.status()).into());
    }

    let txt = response.text().await?;

    // 验证JSON格式
    let jwks: Value =
        serde_json::from_str(&txt).map_err(|e| format!("JSON格式无效: {}", e))?;

    // 验证JWKS结构
    if !jwks.is_object() || !jwks["keys"].is_array() {
        return Err("JWKS格式无效".into());
    }

    info!(log, "get {} contenxt:\n{}", &jwks_url, txt);

    // 将JWKS内容存储到Redis中
    let _ = redis_helper.set_jwks_url_content(txt, 6000); // 设置10分钟过期时间

    Ok(())
}

fn validate_user_permissions(
    req: &ServiceRequest,
    role_mapping: &Option<RoleRule>,
    permission_mapping: &Option<RoleRule>,
    log: &Logger,
) -> bool {
    // 从请求扩展中获取用户信息
    let extensions = req.extensions(); // 先绑定到一个变量
    let user_claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => return false, // 没有找到用户信息
    };
    // 将Claims序列化为JSON值以便使用JSONPath查询
    let claims_json = match serde_json::to_value(user_claims) {
        Ok(json) => json,
        Err(_) => return false,
    };
    info!(log, "Claims JSON:{}", claims_json);
    // 验证角色映射
    if role_mapping.is_some() {
        let role_mapping = role_mapping.as_ref();
        if !validate_role_or_permission(&claims_json, role_mapping.unwrap(), log) {
            return false;
        }
    }
    // 验证权限映射
    if permission_mapping.is_some() {
        let permission_mapping = permission_mapping.as_ref();
        if !validate_role_or_permission(&claims_json, permission_mapping.unwrap(), log) {
            return false;
        }
    }
    true
}

use serde_json::{Value as JsonValue, Value};

fn validate_role_or_permission(claims: &Value, rule: &RoleRule, logger: &Logger) -> bool {
    path_values_intersect(
        &claims,
        rule.json_path.as_str(),
        rule.required_values.as_slice(),
        &logger,
    )
}

use std::collections::HashSet;

/// Extract values (as strings) from `claims_json` using `json_path`.
/// Returns a Vec\<String\> of all extracted scalar/array items converted to strings.
pub fn extract_values_as_strings(claims_json: &JsonValue, json_path: &str) -> Vec<String> {
    match jsonpath_lib::select(claims_json, json_path) {
        Ok(nodes) => {
            let mut out = Vec::new();
            for node in nodes {
                match node {
                    JsonValue::String(s) => out.push(s.clone()),
                    JsonValue::Array(arr) => {
                        for v in arr {
                            match v {
                                JsonValue::String(s) => out.push(s.clone()),
                                // convert other scalar types to string representation
                                JsonValue::Number(_) | JsonValue::Bool(_) | JsonValue::Null => {
                                    out.push(v.to_string())
                                }
                                // for nested objects/arrays push their JSON text
                                _ => out.push(v.to_string()),
                            }
                        }
                    }
                    // numbers, bool, null and objects -> use their JSON text
                    _ => out.push(node.to_string()),
                }
            }
            out
        }
        Err(_) => Vec::new(),
    }
}

/// Check whether `rule_values` has any intersection with the array extracted by `json_path`.
pub fn path_values_intersect(
    claims_json: &JsonValue,
    json_path: &str,
    rule_values: &[String],
    logger: &Logger,
) -> bool {
    let extracted = extract_values_as_strings(claims_json, json_path);
    info!(
        logger,
        "path: {}  and extracted: {:?}", json_path, extracted
    );
    if extracted.is_empty() || rule_values.is_empty() {
        return false;
    }
    let extracted_set: HashSet<String> = extracted.into_iter().collect();
    rule_values.iter().any(|v| extracted_set.contains(v))
}
