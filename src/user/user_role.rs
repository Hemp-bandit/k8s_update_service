use crate::entity::user_role_entity::UserRoleEntity;
use crate::role::check_role_by_id;
use crate::util::common::RedisKeys;
use crate::REDIS;
use redis::Commands;

use super::SubUserRoleData;

/**
 * 检查角色是否存在于cache & db
 */
pub async fn check_role_exists(role_ids: &Vec<i32>) -> Option<bool> {
    //  check in cache
    let mut rds = REDIS.lock().unwrap();
    for id in role_ids {
        let in_ids_cache: bool = rds.sismember(RedisKeys::RoleIds.to_string(), id).expect("");
        let in_info_cache: bool = rds.hexists(RedisKeys::RoleInfo.to_string(), id).expect("");
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
pub fn check_bind(user_id: &i32, role_ids: &Vec<i32>) -> (bool, Vec<i32>, Vec<i32>) {
    let mut rds = REDIS.lock().unwrap();
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), user_id);
    let cache_ids: Vec<i32> = rds.smembers(key).expect("获取角色绑定失败");
    log::info!("cache_user bind role ids {cache_ids:?}");
    if cache_ids.is_empty() {
        return (false, vec![], vec![]);
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

    (true, add_ids, sub_ids)
}

pub fn role_ids_to_add_tab(user_id: &i32, role_ids: &Vec<i32>) -> Vec<UserRoleEntity> {
    let mut rds = REDIS.lock().unwrap();
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), user_id);
    let _: () = rds.sadd(key, role_ids).expect("add new user_role error");

    let tabs: Vec<UserRoleEntity> = role_ids
        .into_iter()
        .map(|id| UserRoleEntity {
            id: None,
            user_id: *user_id,
            role_id: *id,
        })
        .collect();

    tabs
}

pub fn role_ids_to_sub_tab(user_id: &i32, role_ids: &Vec<i32>) -> Vec<SubUserRoleData> {
    let mut rds = REDIS.lock().unwrap();
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), user_id);
    let _: () = rds.srem(key, role_ids).expect("sub new user_role error");

    let tabs: Vec<SubUserRoleData> = role_ids
        .into_iter()
        .map(|id| SubUserRoleData { role_id: *id })
        .collect();

    tabs
}
