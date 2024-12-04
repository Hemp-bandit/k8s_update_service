use crate::{
    entity::role_access_entity::RoleAccessEntity, util::{
        common::RedisKeys,
        redis_actor::{SaddData, SmembersData, SremData},
    }, REDIS_ADDR
};

/// role_ids    cache_id
///
/// [1,2]       [1,2,3,4]    remove 3,4
///   
/// [1,2 ,5]    [1,2,3,4]    remove 3,4 add 5
pub async fn check_role_access_bind(role_id: &i32, access_ids: &Vec<i32>) -> (Vec<i32>, Vec<i32>) {
    let rds = REDIS_ADDR.get().expect("msg");
    let key = format!("{}_{}", RedisKeys::RoleAccess.to_string(), role_id);
    let cache_ids: Vec<i32> = rds
        .send(SmembersData { key })
        .await
        .expect("获取权限绑定失败")
        .expect("获取权限绑定失败");
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
    let rds = REDIS_ADDR.get().expect("msg");
    let key = format!("{}_{}", RedisKeys::RoleAccess.to_string(), role_id);
    for id in role_ids {
        let _ = rds
            .send(SremData {
                key: key.clone(),
                value: id.to_string(),
            })
            .await
            .expect("sub new user_role error");
    }
}


pub async fn bind_role_access(role_id: &i32, access_ids: &Vec<i32>) -> Vec<RoleAccessEntity> {
    let rds = REDIS_ADDR.get().expect("msg");
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), role_id);
    let mut tabs: Vec<RoleAccessEntity> = vec![];
    for id in access_ids {
        let _ = rds
            .send(SaddData {
                key: key.clone(),
                id: id.clone(),
            })
            .await
            .expect("add new user_role error");

        tabs.push(RoleAccessEntity {
            id: None,
            access_id: *id,
            role_id: *role_id,
        });
    }

    tabs
}
