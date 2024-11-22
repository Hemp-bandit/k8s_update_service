use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

use crate::{entity::access_entity::AccessEntity, RB};

mod access_service;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(access_service::create_access);
        config.service(access_service::get_access_list);
        config.service(access_service::update_access_by_id);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateAccessData {
    pub name: String,
    pub create_by: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AccessListQuery {
    pub name: Option<String>,
    pub page_no: i32,
    pub take: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AccessUpdateData {
    pub id: i32,
    pub name: Option<String>,
    pub status: Option<i8>,
}

pub async fn check_access_by_id(id: i32) -> Option<AccessEntity> {
    let ex_db = RB.acquire().await.expect("get db ex error");
    AccessEntity::select_by_id(&ex_db, id.clone())
        .await
        .expect("权限查询失败")
}
