use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

use crate::{entity::user_entity::UserEntity, RB};

mod user_service;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(user_service::create_user);
        config.service(user_service::get_user_list);
        config.service(user_service::get_user_by_id);
        config.service(user_service::update_user_by_id);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreateData {
    pub name: String,
    pub password: String,
    pub phone: String,
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
}

pub async fn check_user(user_id: i16) -> Option<UserEntity> {
    let ex_db = RB.acquire().await.expect("msg");
    let db_user: Option<UserEntity> = UserEntity::select_by_id(&ex_db, user_id.clone().into())
        .await
        .expect("查询用户失败");

    db_user
}
