// src/api_logger_middleware.rs
use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage};
use futures_util::future::LocalBoxFuture;
use slog::{info, Logger};
use std::future::{ready, Ready};
use std::rc::Rc;
use crate::auth_information::Claims;

pub struct ApiLoggerMiddleware {
    pub logger: Logger,
}

impl<S, B> Transform<S, ServiceRequest> for ApiLoggerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ApiLoggerMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiLoggerMiddlewareService {
            service: Rc::new(service),
            logger: self.logger.clone(),
        }))
    }
}

pub struct ApiLoggerMiddlewareService<S> {
    service: Rc<S>,
    logger: Logger,
}

impl<S, B> Service<ServiceRequest> for ApiLoggerMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let logger = self.logger.clone();
        let method = req.method().clone();
        let path = req.path().to_string();
        let peer_addr = req.peer_addr().map(|addr| addr.to_string());
        
        // 记录请求开始
        // 提取查询参数
        let query_string = req.query_string().to_string();

        // 提取请求头信息（排除敏感信息）
        let mut headers_info = std::collections::HashMap::new();
        for (key, value) in req.headers().iter() {
            if let Ok(value_str) = value.to_str() {
                // 只记录安全的头部信息，避免记录敏感信息如Authorization
                let key_lower = key.as_str().to_lowercase();
                if !key_lower.contains("authorization") && !key_lower.contains("cookie") {
                    headers_info.insert(key.as_str().to_string(), value_str.to_string());
                }
            }
        }


        info!(logger, "API Request Started"; 
              "method" => method.as_str(), 
              "path" => &path, 
              "query_params" => &query_string,
              "peer_addr" => format!("{:?}", peer_addr),

              "headers" => serde_json::to_string(&headers_info).unwrap_or_default(),
              "request_id" => generate_request_id());

        let fut = self.service.call(req);

        Box::pin(async move {
            let start_time = std::time::Instant::now();
            let res = fut.await?;
            let duration = start_time.elapsed().as_millis() as u64;
            // 提取用户信息（如果存在）- 在认证中间件处理后
            let mut user_info = String::new();
            let mut user_id = String::new();
            if let Some(claims) = res.request().extensions().get::<Claims>() {
                if let Some(username) = &claims.preferred_username {
                    user_info = username.clone();
                }
                if let Some(sub) = &claims.sub {
                    user_id = sub.clone();
                }
            }

            // 记录响应完成
            let status = res.response().status().as_u16();
            let content_length = res.response().headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            info!(logger, "API Request Completed";
                  "method" => method.as_str(),
                  "path" => &path,
                  "status" => status,
                  "content_length" => &content_length,
                  "duration_ms" => duration,
                  "user" => &user_info,
                  "user_id" => &user_id);

            Ok(res)
        })
    }
}

fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", now)
}
