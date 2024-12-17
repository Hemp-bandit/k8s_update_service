use redis::AsyncCommands;
use rs_service_util::redis_conn;

use crate::{
    entity::{role_access_entity::RoleAccessEntity, user_role_entity::UserRoleEntity},
    util::common::RedisKeys,
    RB,
};
use std::collections::{hash_set::HashSet, HashMap};

/// 同步用户角色关系
pub async fn sync_user_role() {
    log::info!("sync_user_role start");
    let ex = RB.acquire().await.expect("msg");
    let list: Vec<UserRoleEntity> = UserRoleEntity::select_all(&ex).await.expect("msg");
    let mut map: HashMap<i32, HashSet<i32>> = HashMap::new();

    list.into_iter().for_each(|val| {
        if map.contains_key(&val.user_id) {
            let s = map.get_mut(&val.user_id).unwrap();
            s.insert(val.role_id);
        } else {
            let mut set: HashSet<i32> = HashSet::new();
            set.insert(val.role_id);
            map.insert(val.user_id, set);
        }
    });

    log::info!("sync_user_role map {map:?}");
    redis_action(RedisKeys::UserRoles.to_string(), &map).await;
    log::info!("sync_user_role end");
}

/// 同步角色权限关系
pub async fn sync_role_access() {
    log::info!("async_user_role start");
    let ex = RB.acquire().await.expect("msg");
    let list: Vec<RoleAccessEntity> = RoleAccessEntity::select_all(&ex).await.expect("msg");
    let mut map: HashMap<i32, HashSet<i32>> = HashMap::new();

    list.into_iter().for_each(|val| {
        if map.contains_key(&val.role_id) {
            let s = map.get_mut(&val.role_id).unwrap();
            s.insert(val.access_id);
        } else {
            let mut set: HashSet<i32> = HashSet::new();
            set.insert(val.access_id);
            map.insert(val.role_id, set);
        }
    });
    redis_action(RedisKeys::RoleAccess.to_string(), &map).await;
    log::info!("sync_role_access map {map:?}");
}

async fn redis_action(key: String, map: &HashMap<i32, HashSet<i32>>) {
    let mut conn = redis_conn!().await;
    for (id, set) in map.into_iter() {
        let key = format!("{key}_{id}");
        let ids: Vec<i32> = set.to_owned().into_iter().map(|val| val).collect();
        log::info!("key {key}");
        let _: () = conn.del(key.clone()).await.expect("msg");
        for id in ids {
            let _: () = conn.sadd(key.clone(), id).await.expect("msg");
        }
    }
}
