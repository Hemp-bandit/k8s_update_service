use actix_web::{delete, get, post, web, Responder};
use rbs::to_value;
use redis::Commands;

use super::{BindAccessData, CreateRoleData, RoleListQueryData, RoleUpdateData};
use crate::{
    access::check_access_by_id,
    entity::{role_access_entity::RoleAccessEntity, role_entity::RoleEntity},
    response::ResponseBody,
    role::{check_role_access, check_role_by_id, CreateByData, RoleListListData},
    user::{check_user_by_user_id, OptionData},
    util::{
        common::{get_current_time_fmt, get_transaction_tx, rds_str_to_list, RedisKeys},
        sql_tool::{SqlTool, SqlToolPageData},
        structs::Status,
        sync_opt::{self, SyncOptData},
    },
    RB, REDIS,
};

#[utoipa::path(
  tag = "role",
  responses( (status = 200) )
)]
#[post("/create_role")]
async fn create_role(req_data: web::Json<CreateRoleData>) -> impl Responder {
    if check_user_by_user_id(req_data.create_by).await.is_none() {
        return ResponseBody::error("用户不存在");
    }

    let new_role = RoleEntity {
        id: None,
        create_time: get_current_time_fmt(),
        update_time: get_current_time_fmt(),
        name: req_data.name.clone(),
        create_by: req_data.create_by,
        status: Status::ACTIVE as i8,
    };

    let mut tx = get_transaction_tx().await.unwrap();
    let insert_res = RoleEntity::insert(&tx, &new_role).await;
    tx.commit().await.expect("commit error");

    if let Err(rbs::Error::E(error)) = insert_res {
        let res = ResponseBody::error("创建角色失败");
        log::error!(" 创建角色失败 {}", error);
        tx.rollback().await.expect("rollback error");
        return res;
    }

    ResponseBody::success("角色创建成功")
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
pub async fn update_role_by_id(req_data: web::Json<RoleUpdateData>) -> impl Responder {
    match check_role_by_id(req_data.id).await {
        None => {
            return ResponseBody::error("角色不存在");
        }
        Some(mut role) => {
            role.name = req_data.name.clone().unwrap_or(role.name);
            role.update_time = get_current_time_fmt();
            let mut tx = get_transaction_tx().await.expect("get tx err");
            let update_res = RoleEntity::update_by_column(&tx, &role, "id").await;
            tx.commit().await.expect("msg");

            if let Err(rbs::Error::E(error)) = update_res {
                log::error!("更新用户失败, {}", error);
                let res = ResponseBody::error("更新角色失败");
                tx.rollback().await.expect("msg");
                return res;
            }
        }
    }

    ResponseBody::success("角色更新成功")
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[delete("/{id}")]
pub async fn delete_role_by_id(id: web::Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();
    match check_role_by_id(id).await {
        None => {
            return ResponseBody::error("角色不存在");
        }
        Some(mut role) => {
            role.status = Status::DEACTIVE as i8;
            role.update_time = get_current_time_fmt();
            let mut tx = get_transaction_tx().await.expect("get tx err");
            let update_res = RoleEntity::update_by_column(&tx, &role, "id").await;
            tx.commit().await.expect("msg");

            if let Err(rbs::Error::E(error)) = update_res {
                log::error!("更新用户失败, {}", error);
                let res = ResponseBody::error("更新角色失败");
                tx.rollback().await.expect("msg");
                return res;
            }
        }
    }

    ResponseBody::success("角色删除成功")
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[post("/bind_access")]
pub async fn bind_access(req_data: web::Json<BindAccessData>) -> impl Responder {
    let db_role = check_role_by_id(req_data.role_id).await;
    let db_access = check_access_by_id(req_data.access_id).await;
    if db_role.is_none() {
        return ResponseBody::error("角色不存在");
    }
    if db_access.is_none() {
        return ResponseBody::error("权限不存在");
    }
    let db_access = check_role_access(req_data.role_id.clone(), req_data.access_id.clone()).await;
    if !db_access.is_empty() {
        return ResponseBody::error("权限已绑定");
    }

    let new_role_access = RoleAccessEntity {
        id: None,
        role_id: req_data.role_id.clone(),
        access_id: req_data.access_id.clone(),
    };

    let mut tx = get_transaction_tx().await.expect("get tx error");
    let insert_res = RoleAccessEntity::insert(&tx, &new_role_access).await;
    tx.commit().await.expect("msg");
    if let Err(rbs::Error::E(error)) = insert_res {
        log::error!("绑定权限失败, {}", error);
        let res = ResponseBody::error("绑定权限失败");
        tx.rollback().await.expect("msg");
        return res;
    }
    ResponseBody::success("绑定成功")
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[get("/role_binds/{id}")]
pub async fn get_role_binds(parma: web::Path<i32>) -> impl Responder {
    let id = parma.into_inner();
    let db_role = check_role_by_id(id.clone()).await;
    if db_role.is_none() {
        return ResponseBody {
            code: 500,
            msg: "角色不存在".to_string(),
            data: None,
        };
    }

    let ex = RB.acquire().await.expect("msg");
    let search_res = RoleAccessEntity::select_by_column(&ex, "role_id", id)
        .await
        .expect("msg");

    let res: ResponseBody<Option<Vec<RoleAccessEntity>>> = ResponseBody::default(Some(search_res));

    res
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[delete("/un_bind_role")]
pub async fn un_bind_role(req_data: web::Json<BindAccessData>) -> impl Responder {
    let db_role = check_role_by_id(req_data.role_id).await;
    let db_access = check_access_by_id(req_data.access_id).await;
    if db_role.is_none() {
        return ResponseBody::error("角色不存在");
    }
    if db_access.is_none() {
        return ResponseBody::error("权限不存在");
    }

    let mut tx = get_transaction_tx().await.expect("get tx error");
    let delete_res = RoleAccessEntity::delete_by_role_and_access(
        &tx,
        req_data.role_id.clone(),
        req_data.access_id.clone(),
    )
    .await;
    tx.commit().await.expect("msg");
    if let Err(rbs::Error::E(error)) = delete_res {
        log::error!("解除绑定权限失败, {error}");
        let res = ResponseBody::error("解除绑定权限失败");
        tx.rollback().await.expect("msg");
        return res;
    }

    ResponseBody::success("解除绑定成功")
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[get("/get_role_option")]
pub async fn get_role_option() -> impl Responder {
    let mut rds = REDIS.get_connection().expect("msg");
    let ids: Vec<i32> = rds
        .smembers(RedisKeys::RoleIds.to_string())
        .expect("get role_ids rds err");
    if !ids.is_empty() {
        let res: Vec<OptionData> = rds_str_to_list(rds, ids, RedisKeys::RoleInfo, |val| {
            let user_data: OptionData = serde_json::from_str(&val).expect("msg");
            user_data
        });
        ResponseBody::default(Some(res))
    } else {
        let ex_db = RB.acquire().await.expect("get ex err");
        let opt: Vec<OptionData> = ex_db
            .query_decode("select id, name from role where status=1", vec![])
            .await
            .expect("select db err");
        drop(rds);

        for ele in opt.iter() {
            sync_opt::sync(SyncOptData::default(
                RedisKeys::RoleIds,
                RedisKeys::RoleInfo,
                ele.id,
                ele.clone(),
            ));
        }
        ResponseBody::default(Some(opt))
    }
}
