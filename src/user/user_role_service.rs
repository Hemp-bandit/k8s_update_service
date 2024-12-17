use redis::AsyncCommands;
use rs_service_util::{redis_conn, RedisLoginData};

use crate::entity::user_role_entity::UserRoleEntity;
use crate::response::MyError;
use crate::role::check_role_by_id;
use crate::user::auth_service::get_user_access_val;
use crate::util::common::RedisKeys;
use crate::REDIS_KEY;

///检查角色是否存在于cache & db
pub async fn check_role_exists(role_ids: &Vec<i32>) -> Option<bool> {
    //  check in cache
    let mut conn = redis_conn!().await;
    for id in role_ids {
        let in_ids_cache: bool = conn
            .sismember(RedisKeys::RoleIds.to_string(), id)
            .await
            .expect("msg");
        let in_info_cache: bool = conn
            .hexists(RedisKeys::RoleInfo.to_string(), id)
            .await
            .expect("msg");
        if !in_ids_cache && !in_info_cache {
            let db_role = check_role_by_id(id.clone()).await;
            if db_role.is_none() {
                return None;
            }
        }
    }

    Some(true)
}

/// role_ids    cache_id
///
/// [1,2]       [1,2,3,4]    remove 3,4
///
/// [1,2 ,5]    [1,2,3,4]    remove 3,4 add 5
///
///
pub async fn check_user_role_bind(user_id: &i32, role_ids: &Vec<i32>) -> (Vec<i32>, Vec<i32>) {
    let mut conn = redis_conn!().await;
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), user_id);
    let cache_ids: Vec<i32> = conn.smembers(key).await.expect("msg");
    log::info!("cache_user bind role ids {cache_ids:?}");
    if cache_ids.is_empty() {
        return (role_ids.clone(), vec![]);
    }
    // 查看交集
    let mut add_ids: Vec<i32> = vec![];
    let mut sub_ids: Vec<i32> = vec![];

    role_ids.iter().for_each(|id| {
        // 新加的角色不存在 过滤出来
        let is_contain = cache_ids.iter().find(|c_id| **c_id == *id);
        if is_contain.is_none() {
            add_ids.push(*id);
        }
    });

    cache_ids.iter().for_each(|id| {
        // 新加的角色不存在 过滤出来
        let is_contain = role_ids.iter().find(|c_id| **c_id == *id);
        if is_contain.is_none() {
            sub_ids.push(*id);
        }
    });

    (add_ids, sub_ids)
}

pub async fn bind_user_role(user_id: &i32, role_ids: &Vec<i32>) -> Vec<UserRoleEntity> {
    let mut conn = redis_conn!().await;
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), user_id);
    let mut tabs: Vec<UserRoleEntity> = vec![];
    for id in role_ids {
        let _: () = conn.sadd(key.clone(), id).await.expect("msg");
        tabs.push(UserRoleEntity {
            id: None,
            user_id: *user_id,
            role_id: *id,
        });
    }

    tabs
}

pub async fn unbind_role_from_cache(user_id: &i32, role_ids: &Vec<i32>) {
    let mut conn = redis_conn!().await;
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), user_id);
    let _: () = conn
        .srem(key.clone(), role_ids.to_vec())
        .await
        .expect("msg");
}

pub async fn sync_user_auth(name: String) -> Result<u64, MyError> {
    let key = format!("{}_{}", REDIS_KEY.to_string(), name);
    let mut conn = redis_conn!().await;
    let cache_info: Option<String> = conn.get(&key).await.expect("msg");

    log::info!("key {key}");
    log::info!("cache_info {cache_info:#?}");

    if let Some(info) = cache_info {
        let mut login_info: RedisLoginData = serde_json::from_str(&info).expect("msg");
        let new_auth = get_user_access_val(login_info.id).await;
        login_info.auth = new_auth;

        let ttl: u64 = conn.ttl(&key).await.expect("msg");
        let json = serde_json::to_string(&login_info).unwrap();
        let _: () = conn.set_ex(key, json, ttl).await.expect("msg");
        return Ok(new_auth);
    }
    Ok(0)
}
