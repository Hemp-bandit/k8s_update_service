use crate::{
    response::MyError,
    util::{
        common::{jwt_token_to_data, RedisCmd},
        redis_actor::ExistsData,
    },
    REDIS_ADDR, REDIS_KEY,
};
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::HeaderValue,
    middleware::Next,
    Error,
};

pub async fn jwt_mw(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if !check_is_in_whitelist(&req) {
        if !has_permission(&req).await {
            return Err(Error::from(MyError::AuthError));
        }
    }

    let res = next.call(req).await?;
    Ok(res)
}

fn check_is_in_whitelist(req: &ServiceRequest) -> bool {
    let path = req.path();
    // 白名单不校验
    let white_list: Vec<&str> = vec!["/api/auth/login", "/doc"];
    let is_in_white_list = white_list
        .iter()
        .find(|val| val.to_string() == path.to_string());
    is_in_white_list.is_some()
}
async fn has_permission(req: &ServiceRequest) -> bool {
    let value: HeaderValue = HeaderValue::from_str("").unwrap();

    let binding = req.method().to_owned();
    let ret_method = binding.as_str();
    if ret_method == "OPTIONS" {
        return true;
    }

    let token = req.headers().get("Authorization").unwrap_or(&value);
    if token.is_empty() || token.len() < 7 {
        return false;
    };

    let binding = token.to_owned();
    let jwt_token = binding.to_str().expect("msg").to_string();
    let slice = &jwt_token[7..];
    log::info!("jwt {slice}");
    let jwt_user: Option<crate::user::RedisLoginData> = jwt_token_to_data(slice.to_owned());
    log::info!("jwt_user {jwt_user:?}");
    // jwt_user.name
    match jwt_user {
        None => false,
        Some(info) => check_is_login_redis(info.name).await,
    }
}

pub async fn check_is_login_redis(user_name: String) -> bool {
    let key = format!("{}_{}", REDIS_KEY.to_string(), user_name);
    let rds = REDIS_ADDR.get().expect("msg");
    let redis_login: Result<bool, redis::RedisError> = rds
        .send(ExistsData {
            key,
            cmd: RedisCmd::Exists,
            data: None,
        })
        .await
        .expect("msg"); //.exists(key.clone());
    let is_login = match redis_login {
        Err(err) => {
            let detail = err.detail();
            log::error!("{detail:?}",);
            return false;
        }
        Ok(res) => res,
    };
    // TODO: 添加自动刷新
    is_login
}
