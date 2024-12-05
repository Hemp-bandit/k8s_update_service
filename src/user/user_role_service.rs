use crate::entity::user_role_entity::UserRoleEntity;
use crate::role::check_role_by_id;
use crate::util::common::{RedisCmd, RedisKeys};
use crate::util::redis_actor::{ExistsData, SaddData, SmembersData, SremData};
use crate::REDIS_ADDR;

///检查角色是否存在于cache & db
pub async fn check_role_exists(role_ids: &Vec<i32>) -> Option<bool> {
    //  check in cache
    let rds = REDIS_ADDR.get().expect("msg");
    for id in role_ids {
        let in_ids_cache: bool = rds
            .send(ExistsData {
                key: RedisKeys::RoleIds.to_string(),
                cmd: RedisCmd::Sismember,
                data: Some(id.to_string()),
            })
            .await
            .expect("msg")
            .expect("msg");
        let in_info_cache: bool = rds
            .send(ExistsData {
                key: RedisKeys::RoleInfo.to_string(),
                cmd: RedisCmd::Hexists,
                data: Some(id.to_string()),
            })
            .await
            .expect("msg")
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
    let rds = REDIS_ADDR.get().expect("msg");
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), user_id);
    let cache_ids: Vec<i32> = rds
        .send(SmembersData { key })
        .await
        .expect("获取角色绑定失败")
        .expect("获取角色绑定失败");
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
    let rds = REDIS_ADDR.get().expect("msg");
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), user_id);
    let mut tabs: Vec<UserRoleEntity> = vec![];
    for id in role_ids {
        let _ = rds
            .send(SaddData {
                key: key.clone(),
                id: id.clone(),
            })
            .await
            .expect("add new user_role error");

        tabs.push(UserRoleEntity {
            id: None,
            user_id: *user_id,
            role_id: *id,
        });
    }

    tabs
}

pub async fn unbind_role_from_cache(user_id: &i32, role_ids: &Vec<i32>) {
    let rds = REDIS_ADDR.get().expect("msg");
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), user_id);
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
