use rbs::to_value;
use rs_service_util::auth::gen_access_value;
use serde::{Deserialize, Serialize};

use crate::{
    entity::{
        access_entity::AccessEntity, role_access_entity::RoleAccessEntity, role_entity::RoleEntity,
        user_entity::UserEntity, user_role_entity::UserRoleEntity,
    },
    response::MyError,
    util::common::get_transaction_tx,
};
#[derive(Clone, Debug, Serialize, Deserialize)]
struct IdRes {
    id: i32,
}

///检测默认的管理员角色和用户是否存在
pub async fn check_adm() -> Result<(), MyError> {
    log::info!("check adm start");
    let tx = get_transaction_tx().await?;
    // check db user
    let db_adm_user: Option<IdRes> = tx
        .query_decode(
            "select id from user where name='ADMIN' and phone='15717827650' ",
            vec![],
        )
        .await
        .expect("msg");

    log::info!("db_adm_user {db_adm_user:?}");
    let adm_user_id = match db_adm_user {
        None => {
            let adm_user = UserEntity::default_adm_user();
            let res = UserEntity::insert(&tx, &adm_user).await.expect("msg");
            res.rows_affected as i32
        }
        Some(user) => user.id,
    };

    // check db role
    let db_role: Option<IdRes> = tx
        .query_decode("select id from role where name='ADMIN'", vec![])
        .await
        .expect("msg");
    log::info!("db_role {db_role:?}");
    let adm_role_id = match db_role {
        None => {
            let role = RoleEntity::default_adm(adm_user_id);
            let res = RoleEntity::insert(&tx, &role).await.expect("msg");
            res.rows_affected as i32
        }
        Some(role) => role.id,
    };

    // check db access
    let db_access: Option<IdRes> = tx
        .query_decode(
            "select id from access where name='ADMIN'",
            vec![],
        )
        .await
        .expect("msg");

    log::info!("db_access {db_access:?}");

    let adm_access_id = match db_access {
        None => {
            let role = AccessEntity::default_adm_access(adm_user_id);
            let res = AccessEntity::insert(&tx, &role).await.expect("msg");
            let permission = gen_access_value(res.last_insert_id.as_u64().unwrap_or(0));
            let _: () = tx
                .query_decode("update access set value=?", vec![to_value!(permission)])
                .await
                .expect("msg");

            res.rows_affected as i32
        }
        Some(role) => role.id,
    };

    // check role access
    let role_access: Option<IdRes> = tx
        .query_decode(
            "select id from role_access where role_id=? and access_id=?",
            vec![to_value!(adm_role_id), to_value!(adm_access_id)],
        )
        .await
        .expect("msg");

    log::info!("role_access {role_access:?}");

    if role_access.is_none() {
        let new_relation = RoleAccessEntity {
            id: None,
            role_id: adm_role_id,
            access_id: adm_access_id,
        };

        let _res = RoleAccessEntity::insert(&tx, &new_relation)
            .await
            .expect("msg");
    }

    // check user rol relation

    let user_role: Option<IdRes> = tx
        .query_decode(
            "select id from user_role where role_id=? and user_id=?",
            vec![to_value!(adm_role_id), to_value!(adm_user_id)],
        )
        .await
        .expect("msg");

    log::info!("user_role {user_role:?}");

    if user_role.is_none() {
        let new_relation = UserRoleEntity {
            id: None,
            role_id: adm_role_id,
            user_id: adm_user_id,
        };

        let _res = UserRoleEntity::insert(&tx, &new_relation)
            .await
            .expect("msg");
    }

    let _ = tx.commit().await;
    log::info!("check adm end");
    Ok(())
}
