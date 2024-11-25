use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error,
    http::header::HeaderValue,
    Error,
};
use futures_util::future::LocalBoxFuture;

use crate::common::jwt_token_to_data;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct JwtAuth;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware { service }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if !check_is_in_whitelist(&req) {
            if !has_permission(&req) {
                return Box::pin(async move {
                    // 鉴权失败，返回未授权的响应，停止后续中间件的调用
                    Err(error::ErrorUnauthorized("Unauthorized"))
                });
            }
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

fn check_is_in_whitelist(req: &ServiceRequest) -> bool {
    let path = req.path();
    // 白名单不校验
    let white_list: Vec<&str> = vec!["/api/auth/login", "/api/auth/logout"];
    let is_in_white_list = white_list
        .iter()
        .find(|val| val.to_string() == path.to_string());
    is_in_white_list.is_some()
}
fn has_permission(req: &ServiceRequest) -> bool {
    let value: HeaderValue = HeaderValue::from_str("").unwrap();
    let token = req.headers().get("Authorization").unwrap_or(&value);

    if token.is_empty() {
        return false;
    };
    // eyJhbGciOiJIUzI1NiJ9.eyJhdXRoIjoyMDk3MTUyLCJsYXN0X2xvZ2luX3RpbWUiOjE3MzI1MjI5MDcsIm5hbWUiOiIyMjIyIiwiaWQiOjV9.9Kk9R83gHVerDglyzIxYlG07GUMSET-2i621v-WZfaA

    // let jwt_token = token.to_str().expect("msg").to_string();
    // let jwt_token = jwt_token.replace("Bearer ", "");
    let binding = token.to_owned();
    let jwt_token = binding.to_str().expect("msg").to_string();
    let slice = &jwt_token[7..];
    log::info!("jwt {slice}");
    // let jwt_user = jwt_token_to_data(jwt_token);
    // log::info!("jwt_user {jwt_user:?}");
    // jwt_user.name
    true
}
#[test]
fn get() {
    let str = "Bearer 123";
    let res = str.replace("Bearer ", "");
    println!("res {res}");
    assert_eq!(res, "123");
}
