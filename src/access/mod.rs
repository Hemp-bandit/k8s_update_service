use redis::AsyncCommands;
use rs_service_util::redis_conn;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

use crate::{
    entity::access_entity::AccessEntity,
    util::{common::RedisKeys, structs::CreateByData},
    RB,
};

mod access_service;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(access_service::create_access);
        config.service(access_service::get_access_list);
        config.service(access_service::update_access_by_id);
        config.service(access_service::get_access_map);

        config.service(access_service::delete_access);
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
    pub create_by: Option<i32>,
    pub role_id: Option<i32>,
    pub page_no: i32,
    pub take: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AccessUpdateData {
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessListListData {
    pub id: i32,
    pub create_time: String,
    pub update_time: String,
    pub name: String,
    pub create_by: Option<CreateByData>, // 创建的用户id
    pub status: i8,
    pub value: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessMapItem {
    pub id: i32,
    pub name: String,
    pub value: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AccessValueData {
    pub value: u64,
}

pub async fn check_access_by_id(id: i32) -> Option<AccessEntity> {
    let ex_db = RB.acquire().await.expect("get db ex error");
    AccessEntity::select_by_id(&ex_db, id.clone())
        .await
        .expect("权限查询失败")
}

pub async fn check_access_by_ids(list: &Vec<i32>) -> Option<bool> {
    let mut conn = redis_conn!().await;
    for id in list {
        let in_ids_cache: bool = conn
            .sismember(RedisKeys::AccessMapIds.to_string(), id)
            .await
            .expect("msg");
        let in_info_cache: bool = conn
            .hexists(RedisKeys::AccessMap.to_string(), id)
            .await
            .expect("msg");
        if !in_ids_cache && !in_info_cache {
            let db_role = check_access_by_id(id.clone()).await;
            if db_role.is_none() {
                return None;
            }
        }
    }
    Some(true)
}
