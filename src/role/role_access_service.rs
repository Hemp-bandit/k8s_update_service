use redis::AsyncCommands;
use rs_service_util::redis_conn;

use crate::{entity::role_access_entity::RoleAccessEntity, util::common::RedisKeys};

/// role_ids    cache_id
///
/// [1,2]       [1,2,3,4]    remove 3,4
///
/// [1,2 ,5]    [1,2,3,4]    remove 3,4 add 5
pub async fn check_role_access_bind(role_id: &i32, access_ids: &Vec<i32>) -> (Vec<i32>, Vec<i32>) {
    let mut conn = redis_conn!().await;
    let key = format!("{}_{}", RedisKeys::RoleAccess.to_string(), role_id);
    let cache_ids: Vec<i32> = conn.smembers(key).await.expect("msg");
    log::info!("cache_role_access bind access ids {cache_ids:?}");
    if cache_ids.is_empty() {
        return (access_ids.clone(), vec![]);
    }
    // 查看交集
    let mut add_ids: Vec<i32> = vec![];
    let mut sub_ids: Vec<i32> = vec![];

    access_ids.iter().for_each(|id| {
        let is_contain = cache_ids.iter().find(|c_id| **c_id == *id);
        if is_contain.is_none() {
            add_ids.push(*id);
        }
    });

    cache_ids.iter().for_each(|id| {
        let is_contain = access_ids.iter().find(|c_id| **c_id == *id);
        if is_contain.is_none() {
            sub_ids.push(*id);
        }
    });

    (add_ids, sub_ids)
}

pub async fn unbind_access_from_cache(role_id: &i32, role_ids: &Vec<i32>) {
    let mut conn = redis_conn!().await;
    let key = format!("{}_{}", RedisKeys::RoleAccess.to_string(), role_id);

    let _: () = conn.srem(key, role_ids.to_vec()).await.expect("msg");
}

pub async fn bind_role_access(role_id: &i32, access_ids: &Vec<i32>) -> Vec<RoleAccessEntity> {
    let mut conn = redis_conn!().await;
    let key = format!("{}_{}", RedisKeys::RoleAccess.to_string(), role_id);
    let mut tabs: Vec<RoleAccessEntity> = vec![];
    for id in access_ids {
        let _: () = conn.sadd(key.clone(), id).await.expect("msg");

        tabs.push(RoleAccessEntity {
            id: None,
            access_id: *id,
            role_id: *role_id,
        });
    }

    tabs
}
