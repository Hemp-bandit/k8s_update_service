use super::{AccessListQuery, AccessMapItem, AccessUpdateData, CreateAccessData};
use crate::{
    access::{check_access_by_id, AccessListListData},
    entity::access_entity::AccessEntity,
    response::{MyError, ResponseBody},
    user::check_user_by_user_id,
    util::{
        common::{
            gen_access_value, get_current_time_fmt, get_transaction_tx, rds_str_to_list, RedisKeys,
        },
        redis_actor::SmembersData,
        sql_tool::{SqlTool, SqlToolPageData},
        structs::{CreateByData, Status},
        sync_opt::{self, DelOptData, SyncOptData},
    },
    RB, REDIS_ADDR,
};
use actix_web::{delete, get, post, web, Responder};
use rbs::to_value;

#[utoipa::path(
    tag = "access",
    responses( (status = 200))
)]
#[post("/create_access")]
async fn create_access(req_data: web::Json<CreateAccessData>) -> Result<impl Responder, MyError> {
    if check_user_by_user_id(req_data.create_by).await.is_none() {
        return Err(MyError::UserNotExist);
    }

    let new_access = AccessEntity {
        id: None,
        create_time: get_current_time_fmt(),
        update_time: get_current_time_fmt(),
        name: req_data.name.clone(),
        create_by: req_data.create_by,
        status: Status::ACTIVE as i8,
        value: 0,
    };

    let tx = get_transaction_tx().await.unwrap();
    let insert_res = AccessEntity::insert(&tx, &new_access).await;
    tx.commit().await.expect("commit error");
    match insert_res {
        Err(rbs::Error::E(error)) => {
            log::error!(" {} {error}", MyError::CreateAccessError);
            tx.rollback().await.expect("rollback error");
            return Err(MyError::CreateAccessError);
        }
        Ok(res) => {
            let permission = gen_access_value(res.last_insert_id.as_u64().unwrap_or(0));
            let mut update_access = new_access.clone();
            update_access.id = Some(res.last_insert_id.as_i64().unwrap_or(0).try_into().unwrap());
            update_access.value = permission;
            update_access.update_time = get_current_time_fmt();

            let update_res = AccessEntity::update_by_column(&tx, &update_access, "id").await;
            if let Err(rbs::Error::E(error)) = update_res {
                log::error!(" {} {error}", MyError::UpdateAccessError);
                tx.rollback().await.expect("rollback error");
                return Err(MyError::UpdateAccessError);
            }
        }
    }

    Ok(ResponseBody::success("权限创建成功"))
}

#[utoipa::path(
    tag = "access",
    responses( (status = 200))
)]
#[post("/get_access_list")]
async fn get_access_list(req_data: web::Json<AccessListQuery>) -> impl Responder {
    let ex_db = RB.acquire().await.expect("msg");
    let mut tool = SqlTool::init("select * from access", "order by create_time desc");

    if let Some(name) = req_data.name.clone() {
        tool.append_sql_filed("name", to_value!(name));
    }
    if let Some(role_id) = req_data.role_id {
        tool.append_sql_filed("role_id", to_value!(role_id));
    }
    if let Some(create_by) = req_data.create_by {
        tool.append_sql_filed("create_by", to_value!(create_by));
    }
    tool.append_sql_filed("status", to_value!(1));

    let page_sql = tool.gen_page_sql(req_data.page_no, req_data.take);
    let db_res: Vec<AccessEntity> = ex_db
        .query_decode(&page_sql, tool.opt_val.clone())
        .await
        .expect("msg");

    let mut records: Vec<AccessListListData> = vec![];
    for val in db_res {
        let create_by: Option<CreateByData> = ex_db
            .query_decode(
                "select id, name from user where id=?",
                vec![to_value!(val.create_by)],
            )
            .await
            .expect("err");
        let val: AccessListListData = AccessListListData {
            id: val.id.expect("msg"),
            create_by,
            create_time: val.create_time,
            update_time: val.update_time,
            name: val.name,
            status: val.status,
            value: val.value,
        };
        records.push(val);
    }
    let conf = SqlToolPageData {
        ex_db,
        table: "access".to_string(),
        records,
        page_no: req_data.page_no as u64,
        page_size: req_data.take as u64,
    };
    let db_res = tool.page_query(conf).await;

    ResponseBody::default(Some(db_res))
}

#[utoipa::path(
    tag = "access",
    responses( (status = 200))
)]
#[post("/update_access")]
pub async fn update_access_by_id(
    req_data: web::Json<AccessUpdateData>,
) -> Result<impl Responder, MyError> {
    match check_access_by_id(req_data.id).await {
        None => {
            return Err(MyError::AccessNotExist);
        }
        Some(mut access) => {
            //sync db first
            access.name = req_data.name.clone().unwrap_or(access.name);
            access.update_time = get_current_time_fmt();
            let tx = get_transaction_tx().await.expect("get tx err");
            let update_res = AccessEntity::update_by_column(&tx, &access, "id").await;
            tx.commit().await.expect("msg");

            if let Err(rbs::Error::E(error)) = update_res {
                log::error!(" {} {error}", MyError::UpdateAccessError);
                tx.rollback().await.expect("rollback error");
                return Err(MyError::UpdateAccessError);
            }
            // sync cache
            let item = AccessMapItem {
                id: access.id.unwrap(),
                name: access.name,
                value: access.value,
            };

            sync_opt::sync(SyncOptData::default(
                RedisKeys::AccessMapIds,
                RedisKeys::AccessMap,
                item.id,
                item,
            ))
            .await;
        }
    }

    Ok(ResponseBody::success("权限更新成功"))
}

#[utoipa::path(
    tag = "access",
    responses( (status = 200))
)]
#[delete("/{id}")]
pub async fn delete_access(id: web::Path<i32>) -> Result<impl Responder, MyError> {
    let id = id.into_inner();

    match check_access_by_id(id).await {
        None => {
            return Err(MyError::AccessNotExist);
        }
        Some(mut access) => {
            access.status = Status::DEACTIVE as i8;
            access.update_time = get_current_time_fmt();
            let tx = get_transaction_tx().await.expect("get tx err");
            let update_res = AccessEntity::update_by_column(&tx, &access, "id").await;
            tx.commit().await.expect("msg");

            if let Err(rbs::Error::E(error)) = update_res {
                log::error!(" {} {error}", MyError::UpdateAccessError);
                tx.rollback().await.expect("rollback error");
                return Err(MyError::UpdateAccessError);
            }

            sync_opt::del(DelOptData::default(
                RedisKeys::AccessMapIds,
                RedisKeys::AccessMap,
                vec![id],
            ))
            .await;
        
        }
    }

    Ok(ResponseBody::success("权限更新成功"))
}

#[utoipa::path(
    tag = "access",
    responses( (status = 200))
)]
#[get("/access_map")]
pub async fn get_access_map() -> Result<impl Responder, MyError> {
    let rds = REDIS_ADDR.get().expect("msg");
    let cache_ids: Vec<i32> = rds
        .send(SmembersData {
            key: RedisKeys::AccessMapIds.to_string(),
        })
        .await
        .expect("msg")
        .expect("msg");

    if cache_ids.is_empty() {
        let list: Vec<AccessMapItem> = get_access().await;
        for ele in &list {
            sync_opt::sync(SyncOptData::default(
                RedisKeys::UserIds,
                RedisKeys::UserInfo,
                ele.id,
                ele.clone(),
            ))
            .await;
        }
        Ok(ResponseBody::default(Some(list)))
    } else {
        let list: Vec<AccessMapItem> = rds_str_to_list(cache_ids, RedisKeys::AccessMap, |val| {
            let item: AccessMapItem = serde_json::from_str(&val).expect("msg");
            item
        })
        .await;
        Ok(ResponseBody::default(Some(list)))
    }
}

async fn get_access() -> Vec<AccessMapItem> {
    let ex = RB.acquire().await.expect("asdf");
    let list: Vec<AccessMapItem> = ex
        .query_decode("select id,name,value from access where status=1", vec![])
        .await
        .expect("msg");
    list
}
