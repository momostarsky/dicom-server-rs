// auth_middleware.rs
use actix_web::body::{EitherBody, MessageBody};
use actix_web::{
    Error, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::future::{Ready, ready};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,
    exp: usize,
    // 其他claims字段
}

pub struct AuthMiddleware {
    jwt_secret_x: String,
}
impl AuthMiddleware {
    pub fn new(secret: String) -> Self {
        Self {
            jwt_secret_x: secret,
        }
    }
}
impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
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
            service,
            jwt_secret: self.jwt_secret_x.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 需要认证的路径
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    // 验证JWT token
                    let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
                    let validation = Validation::new(Algorithm::HS256);

                    return match decode::<Claims>(token, &decoding_key, &validation) {
                        Ok(_) => {
                            // Token有效，继续处理请求
                            let fut = self.service.call(req);
                            Box::pin(async move {
                                let res = fut.await?;
                                // 将正常响应转换为EitherBody的左分支
                                Ok(res.map_into_left_body())
                            })
                        }
                        Err(_) => {
                            // Token无效

                            Box::pin(async move {
                                // Token无效
                                // Token无效
                                let response = HttpResponse::Unauthorized().body("Invalid token");
                                // 将错误响应转换为EitherBody的右分支
                                let res = req.into_response(
                                    response.map_into_boxed_body().map_into_right_body(),
                                );
                                Ok(res)
                            })
                        }
                    };
                }
            }
        }
        Box::pin(async move {
            // 缺少Authorization头或格式不正确
            let response = HttpResponse::Unauthorized().body("Missing or invalid Authorization header");
            // 将错误响应转换为EitherBody的右分支
            let res = req.into_response(response.map_into_boxed_body().map_into_right_body());
            Ok(res)
        })
    }
}
