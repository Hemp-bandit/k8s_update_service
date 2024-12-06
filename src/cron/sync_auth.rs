use crate::{
    entity::{role_access_entity::RoleAccessEntity, user_role_entity::UserRoleEntity},
    util::{
        common::RedisKeys,
        redis_actor::{DelData, SaddData},
    },
    RB, REDIS_ADDR,
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
    let rds = REDIS_ADDR.get().expect("msg");
    for (id, set) in map.into_iter() {
        let key = format!("{key}_{id}");
        let ids: Vec<i32> = set.to_owned().into_iter().map(|val| val).collect();
        log::info!("key {key}");
        rds.send(DelData { key: key.clone() })
            .await
            .expect("msg")
            .expect("msg");

        for id in ids {
            rds.send(SaddData {
                key: key.clone(),
                id,
            })
            .await
            .expect("msg")
            .expect("msg");
        }
    }
}
