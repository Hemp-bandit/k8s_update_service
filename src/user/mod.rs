use crate::{entity::user_entity::UserEntity, RB};
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

mod obs;
mod user_service;

pub mod admin;
pub mod auth_service;
pub mod user_role_service;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(user_service::create_user);
        config.service(user_service::get_user_list);
        config.service(user_service::bind_role);
        config.service(user_service::get_user_option);

        config.service(user_service::update_user_by_id);
        config.service(user_service::get_user_by_id);
        config.service(user_service::delete_user);
        config.service(user_service::get_role_binds);
    }
}

pub fn auth_configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(auth_service::login);
        config.service(auth_service::logout);
        config.service(auth_service::get_user_permission);
    }
}

pub fn obs_configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(obs::get_keys);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreateData {
    pub name: String,
    pub password: String,
    pub phone: String,
    pub user_type: i16,
    pub picture: Option<String>,
    pub introduce: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct UserUpdateData {
    pub name: Option<String>,
    pub password: Option<String>,
    pub phone: Option<String>,
    pub picture: Option<String>,
    pub introduce: Option<String>,
    pub user_type: Option<i16>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct BindRoleData {
    pub role_id: Vec<i32>,
    pub user_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct RoidS {
    pub role_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRedisValue, ToRedisArgs, PartialEq)]
pub struct RedisLoginData {
    pub auth: u64,
    pub last_login_time: i64,
    pub name: String,
    pub id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct UserListQuery {
    pub name: Option<String>,
    pub user_type: Option<i16>,
    pub page_no: i32,
    pub take: i32,
}

/**
 * 后台登陆
 */
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginData {
    pub name: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct OptionData {
    pub id: i32,
    pub name: String,
}

impl OptionData {
    pub fn default(name: &str, id: i32) -> Self {
        Self {
            name: name.to_string(),
            id,
        }
    }
}

pub async fn check_user_by_user_id(user_id: i32) -> Option<UserEntity> {
    // check in redis
    let ex_db = RB.acquire().await.expect("msg");
    let db_user: Option<UserEntity> = UserEntity::select_by_id(&ex_db, user_id.clone().into())
        .await
        .expect("查询用户失败");

    db_user
}
