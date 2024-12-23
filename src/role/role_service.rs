use actix_web::{delete, get, post, web, Responder};
use rbs::to_value;
use redis::AsyncCommands;
use rs_service_util::{
    redis_conn,
    sql_tool::{SqlTool, SqlToolPageData},
    time::get_current_time_fmt,
};

use super::{BindAccessData, CreateRoleData, RoleListQueryData, RoleUpdateData};
use crate::{
    access::check_access_by_ids,
    entity::{
        access_entity::AccessEntity, role_access_entity::RoleAccessEntity, role_entity::RoleEntity,
    },
    response::{MyError, ResponseBody},
    role::{
        check_role_by_id,
        role_access_service::{bind_role_access, check_role_access_bind, unbind_access_from_cache},
        CreateByData, RoleListListData,
    },
    user::{check_user_by_user_id, user_role_service::sync_user_auth, OptionData},
    util::{
        common::{get_transaction_tx, rds_str_to_list, RedisKeys},
        structs::Status,
        sync_opt::{self, DelOptData, SyncOptData},
    },
    RB,
};

#[utoipa::path(
  tag = "role",
  responses( (status = 200) )
)]
#[post("/create_role")]
async fn create_role(req_data: web::Json<CreateRoleData>) -> Result<impl Responder, MyError> {
    if check_user_by_user_id(req_data.create_by).await.is_none() {
        return Err(MyError::RoleNotExist);
    }

    let new_role = RoleEntity {
        id: None,
        create_time: get_current_time_fmt(),
        update_time: get_current_time_fmt(),
        name: req_data.name.clone(),
        create_by: req_data.create_by,
        status: Status::ACTIVE as i8,
    };

    let tx = get_transaction_tx().await.unwrap();
    let insert_res = RoleEntity::insert(&tx, &new_role).await;
    tx.commit().await.expect("commit error");

    if let Err(rbs::Error::E(error)) = insert_res {
        log::error!(" 创建角色失败 {}", error);
        tx.rollback().await.expect("rollback error");
        return Err(MyError::CreateRoleError);
    }

    let opt = OptionData::default(
        &req_data.name,
        insert_res
            .expect("msg")
            .last_insert_id
            .as_i64()
            .expect("msg") as i32,
    );
    sync_opt::sync(SyncOptData::default(
        RedisKeys::RoleIds,
        RedisKeys::RoleInfo,
        opt.id,
        opt,
    ))
    .await;

    Ok(ResponseBody::success("角色创建成功"))
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[post("/get_role_list")]
async fn get_role_list(req_data: web::Json<RoleListQueryData>) -> impl Responder {
    let ex_db: rbatis::executor::RBatisConnExecutor = RB.acquire().await.expect("msg");
    let mut tool = SqlTool::init("select * from role", "order by create_time desc");

    if let Some(name) = &req_data.name {
        tool.append_sql_filed("name", to_value!(name));
    }
    if let Some(create_by) = req_data.create_by {
        tool.append_sql_filed("create_by", to_value!(create_by));
    }
    tool.append_sql_filed("status", to_value!(1));

    let page_sql = tool.gen_page_sql(req_data.page_no, req_data.take);
    let db_res: Vec<RoleEntity> = ex_db
        .query_decode(&page_sql, tool.opt_val.clone())
        .await
        .expect("msg");

    let mut records: Vec<RoleListListData> = vec![];
    for val in db_res {
        let create_by: Option<CreateByData> = ex_db
            .query_decode(
                "select id, name from user where id=?",
                vec![to_value!(val.create_by)],
            )
            .await
            .expect("err");
        let val: RoleListListData = RoleListListData {
            id: val.id.expect("msg"),
            create_by,
            create_time: val.create_time,
            update_time: val.update_time,
            name: val.name,
            status: val.status,
        };
        records.push(val);
    }

    let conf = SqlToolPageData {
        ex_db,
        table: "role".to_string(),
        records,
        page_no: req_data.page_no as u64,
        page_size: req_data.take as u64,
    };
    let db_res = tool.page_query(conf).await;

    ResponseBody::default(Some(db_res))
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[post("/update_role")]
pub async fn update_role_by_id(
    req_data: web::Json<RoleUpdateData>,
) -> Result<impl Responder, MyError> {
    match check_role_by_id(req_data.id).await {
        None => {
            return Err(MyError::RoleNotExist);
        }
        Some(mut role) => {
            role.name = req_data.name.clone().unwrap_or(role.name);
            role.update_time = get_current_time_fmt();
            let tx = get_transaction_tx().await.expect("get tx err");
            let update_res = RoleEntity::update_by_column(&tx, &role, "id").await;
            tx.commit().await.expect("msg");

            if let Err(rbs::Error::E(error)) = update_res {
                log::error!("{}, {}", error, MyError::UpdateRoleError);
                tx.rollback().await.expect("msg");
                return Err(MyError::UpdateRoleError);
            }

            let item = OptionData {
                id: role.id.unwrap(),
                name: role.name,
            };

            sync_opt::sync(SyncOptData::default(
                RedisKeys::RoleIds,
                RedisKeys::RoleInfo,
                item.id,
                item,
            ))
            .await;
        }
    }

    Ok(ResponseBody::success("角色更新成功"))
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[delete("/{id}")]
pub async fn delete_role_by_id(id: web::Path<i32>) -> Result<impl Responder, MyError> {
    let id: i32 = id.into_inner();
    match check_role_by_id(id).await {
        None => {
            return Err(MyError::RoleNotExist);
        }
        Some(mut role) => {
            role.status = Status::DEACTIVE as i8;
            role.update_time = get_current_time_fmt();
            let tx = get_transaction_tx().await.expect("get tx err");
            let update_res = RoleEntity::update_by_column(&tx, &role, "id").await;
            tx.commit().await.expect("msg");

            if let Err(rbs::Error::E(error)) = update_res {
                log::error!("{}, {}", error, MyError::UpdateRoleError);
                tx.rollback().await.expect("msg");
                return Err(MyError::UpdateRoleError);
            }
        }
    }

    sync_opt::del(DelOptData::default(
        RedisKeys::RoleIds,
        RedisKeys::RoleInfo,
        vec![id],
    ))
    .await;

    Ok(ResponseBody::success("角色删除成功"))
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[post("/bind_access")]
pub async fn bind_access(req_data: web::Json<BindAccessData>) -> Result<impl Responder, MyError> {
    let db_role = check_role_by_id(req_data.role_id).await;
    let db_access = check_access_by_ids(&req_data.access_ids).await;
    if db_role.is_none() {
        return Err(MyError::RoleNotExist);
    }
    if db_access.is_none() {
        return Err(MyError::AccessNotExist);
    }
    let (add_ids, sub_ids) = check_role_access_bind(&req_data.role_id, &req_data.access_ids).await;

    log::debug!("add_ids {add_ids:?}");
    log::debug!("sub_ids {sub_ids:?}");

    if !sub_ids.is_empty() {
        unbind_access_from_cache(&req_data.role_id, &sub_ids).await;
        let tx = RB.acquire_begin().await.expect("msg");
        for id in sub_ids {
            let sub_res: Result<Option<()>, rbs::Error> = tx
                .query_decode(
                    "delete from role_access where access_id=? and role_id = ?",
                    vec![to_value!(id), to_value!(req_data.role_id)],
                )
                .await;
            if let Err(rbs::Error::E(error)) = sub_res {
                log::error!("{}, {error}", MyError::DelRoleAccessError);
                tx.rollback().await.expect("msg");
                return Err(MyError::DelRoleAccessError);
            }
            tx.commit().await.expect("msg");
        }
    } else {
        if req_data.access_ids.is_empty() {
            let tx = RB.acquire_begin().await.expect("msg");
            let sub_res =
                RoleAccessEntity::delete_by_column(&tx, "role_id", req_data.role_id).await;
            if let Err(rbs::Error::E(error)) = sub_res {
                log::error!("{}, {error}", MyError::DelRoleAccessError);
                tx.rollback().await.expect("msg");
                return Err(MyError::DelRoleAccessError);
            }
            tx.commit().await.expect("msg");
        }
    }

    if !add_ids.is_empty() {
        let add_tabs: Vec<RoleAccessEntity> = bind_role_access(&req_data.role_id, &add_ids).await;
        log::debug!("add_tabs {add_tabs:#?}");
        let tx = RB.acquire_begin().await.expect("msg");
        let add_res = RoleAccessEntity::insert_batch(&tx, &add_tabs, add_tabs.len() as u64).await;

        if let Err(rbs::Error::E(error)) = add_res {
            log::error!("{}, {error}", MyError::DelRoleAccessError);
            tx.rollback().await.expect("msg");
            return Err(MyError::DelRoleAccessError);
        }
        tx.commit().await.expect("msg");
    }
    let tx = RB.acquire().await.expect("msg");
    let user_list :Vec<OptionData> = tx.query_decode("select user.name, user.id from user_role left join user on user.id = user_role.user_id  where role_id = ?;", vec![to_value!(req_data.role_id)]).await.expect("msg");
    drop(tx);
    for ele in user_list.into_iter() {
        sync_user_auth(ele.name).await?;
    }

    Ok(ResponseBody::success("绑定成功"))
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[get("/role_binds/{id}")]
pub async fn get_role_binds(parma: web::Path<i32>) -> Result<impl Responder, MyError> {
    let id = parma.into_inner();
    let db_role = check_role_by_id(id.clone()).await;
    if db_role.is_none() {
        return Err(MyError::UserNotExist);
    }

    let mut conn = redis_conn!().await;
    let key: String = format!("{}_{}", RedisKeys::RoleAccess.to_string(), id);
    let cache_ids: Vec<i32> = conn.smembers(key).await.expect("msg");

    let ex = RB.acquire().await.expect("msg");
    let search_res: Vec<AccessEntity> = if cache_ids.is_empty() {
        let access :Vec<AccessEntity>=  ex.query_decode("select access.* from role_access left join access on role_access.access_id = access.id where role_id=? and access.status = 1;", vec![to_value!(id)]).await.expect("msg");
        let key = format!("{}_{}", RedisKeys::RoleAccess.to_string(), id);
        for ele in access.iter() {
            let _: () = conn.sadd(key.clone(), ele.id.unwrap()).await.expect("msg");
        }
        access
    } else {
        let roles = AccessEntity::select_in_column(&ex, "id", &cache_ids)
            .await
            .expect("获取角色绑定权限失败");
        roles
    };
    Ok(ResponseBody::default(Some(search_res)))
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[get("/get_role_option")]
pub async fn get_role_option() -> impl Responder {
    let mut conn = redis_conn!().await;
    let ids: Vec<i32> = conn
        .smembers(RedisKeys::RoleIds.to_string())
        .await
        .expect("msg");
    if !ids.is_empty() {
        let res: Vec<OptionData> = rds_str_to_list(ids, RedisKeys::RoleInfo, |val| {
            let user_data: OptionData = serde_json::from_str(&val).expect("msg");
            user_data
        })
        .await;
        ResponseBody::default(Some(res))
    } else {
        let ex_db = RB.acquire().await.expect("get ex err");
        let opt: Vec<OptionData> = ex_db
            .query_decode("select id, name from role where status=1", vec![])
            .await
            .expect("select db err");
        for ele in opt.iter() {
            sync_opt::sync(SyncOptData::default(
                RedisKeys::RoleIds,
                RedisKeys::RoleInfo,
                ele.id,
                ele.clone(),
            ))
            .await;
        }
        ResponseBody::default(Some(opt))
    }
}
