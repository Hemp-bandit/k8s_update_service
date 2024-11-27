use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

use crate::{
    entity::{user_entity::UserEntity, user_role_entity::UserRoleEntity},
    RB,
};

pub mod auth_service;
mod user_service;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(user_service::create_user);
        config.service(user_service::get_user_list);
        config.service(user_service::get_user_by_id);
        config.service(user_service::update_user_by_id);

        config.service(user_service::bind_role);
        config.service(user_service::un_bind_role);
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
    pub role_id: i32,
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

/**
 * 后台登陆
 */
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginData {
    pub name: String,
    pub password: String,
}

pub async fn check_user_by_user_id(user_id: i32) -> Option<UserEntity> {
    let ex_db = RB.acquire().await.expect("msg");
    let db_user: Option<UserEntity> = UserEntity::select_by_id(&ex_db, user_id.clone().into())
        .await
        .expect("查询用户失败");

    db_user
}

pub async fn check_user_role(role_id: i32, user_id: i32) -> Vec<UserRoleEntity> {
    let ex_db = RB.acquire().await.expect("get db ex error");
    let db_res: Vec<UserRoleEntity> =
        UserRoleEntity::find_by_role_and_user(&ex_db, role_id.into(), user_id)
            .await
            .expect("角色关系查询失败");
    db_res
}
