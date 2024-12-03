
use super::{BindRoleData, UserCreateData, UserListQuery, UserUpdateData};
use crate::entity::role_entity::RoleEntity;
use crate::response::MyError;
use crate::user::user_role::{
    check_bind, check_role_exists, role_ids_to_add_tab, role_ids_to_sub_tab,
};
use crate::util::common::{rds_str_to_list, RedisKeys};
use crate::{
    entity::{user_entity::UserEntity, user_role_entity::UserRoleEntity},
    response::ResponseBody,
    user::{check_user_by_user_id, OptionData},
    util::{
        common::{check_phone, get_current_time_fmt, get_transaction_tx},
        sql_tool::{SqlTool, SqlToolPageData},
        structs::Status,
        sync_opt::{self, SyncOptData},
    },
    RB, REDIS,
};
use actix_web::{delete, get, post, web, Responder};
use rbs::to_value;
use redis::Commands;
#[utoipa::path(
    tag = "user",
    responses( (status = 200) )
)]
#[post("/create_user")]
pub async fn create_user(req_data: web::Json<UserCreateData>) -> impl Responder {
    let phone_check_res = check_phone(&req_data.phone);
    if !phone_check_res {
        let rsp: ResponseBody<Option<String>> = ResponseBody::error("手机号不正确");
        return rsp;
    }

    let insert_user = UserEntity {
        id: None,
        create_time: get_current_time_fmt(),
        update_time: get_current_time_fmt(),
        name: req_data.name.clone(),
        password: req_data.password.clone(),
        phone: req_data.phone.clone(),
        picture: req_data.picture.clone(),
        introduce: req_data.introduce.clone(),
        user_type: req_data.user_type,
        status: Status::ACTIVE as i16,
    };

    let mut tx = get_transaction_tx().await.unwrap();
    let insert_res = UserEntity::insert(&tx, &insert_user).await;
    tx.commit().await.expect("commit transaction error ");
    match insert_res {
        Err(rbs::Error::E(error)) => {
            let rsp = ResponseBody::error("创建用户失败");
            log::error!(" 创建用户失败 {}", error);
            tx.rollback().await.expect("rollback error");
            return rsp;
        }
        Ok(res) => {
            let opt = OptionData::default(
                &req_data.name,
                res.last_insert_id.as_i64().expect("msg") as i32,
            );
            sync_opt::sync(SyncOptData::default(
                RedisKeys::UserIds,
                RedisKeys::UserInfo,
                opt.id,
                opt,
            ))
            .await;
        }
    }

    ResponseBody::success("创建用户成功")
}

#[utoipa::path(
    tag = "user",
    responses( (status = 200) )
)]
#[post("/get_user_list")]
pub async fn get_user_list(req_data: web::Json<UserListQuery>) -> impl Responder {
    let ex_db = RB.acquire().await.expect("msg");
    let mut tool = SqlTool::init("select * from user", "order by create_time desc");
    if let Some(name) = &req_data.name {
        tool.append_sql_filed("name", to_value!(name));
    }
    if let Some(user_type) = req_data.user_type {
        tool.append_sql_filed("user_type", to_value!(user_type));
    }

    tool.append_sql_filed("status", to_value!(1));
    let page_sql = tool.gen_page_sql(req_data.page_no, req_data.take);

    let db_res: Vec<UserEntity> = ex_db
        .query_decode(&page_sql, tool.opt_val.clone())
        .await
        .expect("msg");
    let conf: SqlToolPageData<UserEntity> = SqlToolPageData {
        ex_db,
        table: "user".to_string(),
        records: db_res,
        page_no: req_data.page_no as u64,
        page_size: req_data.take as u64,
    };

    let page_res = tool.page_query(conf).await;
    ResponseBody::default(Some(page_res))
}

#[utoipa::path(
    tag = "user",
    params(("id", description = "user id") ),
    responses( (status = 200) )
)]
#[get("/{id}")]
pub async fn get_user_by_id(id: web::Path<i32>) -> impl Responder {
    let ex_db = RB.acquire().await.expect("msg");
    let user_id = id.into_inner();
    let db_res: Option<UserEntity> = UserEntity::select_by_id(&ex_db, user_id)
        .await
        .expect("查询用户失败");

    ResponseBody::default(Some(db_res))
}

#[utoipa::path(
    tag = "user",
    params(("id", description = "user id") ),
    responses( (status = 200) )
)]
#[post("/{id}")]
pub async fn update_user_by_id(
    id: web::Path<i32>,
    req_data: web::Json<UserUpdateData>,
) -> impl Responder {
    if let Some(new_phone) = &req_data.phone {
        let phone_check_res = check_phone(new_phone);
        if !phone_check_res {
            let res: ResponseBody<Option<String>> = ResponseBody::error("手机号不正确");
            return res;
        }
    }

    let ex_db = RB.acquire().await.expect("msg");

    let user_id = id.into_inner();
    let db_res: Option<UserEntity> = UserEntity::select_by_id(&ex_db, user_id)
        .await
        .expect("查询用户失败");

    match db_res {
        None => {
            return ResponseBody::error("用户不存在");
        }
        Some(mut db_user) => {
            db_user.update_time = get_current_time_fmt();
            db_user.introduce = req_data.introduce.clone();
            db_user.name = req_data.name.clone().unwrap_or(db_user.name);
            db_user.password = req_data.password.clone().unwrap_or(db_user.password);
            db_user.picture = req_data.picture.clone();
            db_user.phone = req_data.phone.clone().unwrap_or(db_user.phone);
            db_user.user_type = req_data.user_type.clone().unwrap_or(db_user.user_type);

            let mut tx = get_transaction_tx().await.unwrap();
            let update_res = UserEntity::update_by_column(&tx, &db_user, "id").await;
            tx.commit().await.expect("msg");
            if let Err(rbs::Error::E(error)) = update_res {
                log::error!("更新用户失败, {}", error);
                let res = ResponseBody::error("更新用户失败");
                tx.rollback().await.expect("msg");
                return res;
            }

            let opt = OptionData::default(&db_user.name, db_user.id.clone().expect("msg"));
            sync_opt::sync(SyncOptData::default(
                RedisKeys::UserIds,
                RedisKeys::UserInfo,
                opt.id,
                opt,
            ))
            .await;
        }
    }
    ResponseBody::success("更新用户成功")
}

#[utoipa::path(
    tag = "user",
    params(("id", description = "user id") ),
    responses( (status = 200) )
)]
#[delete("/{id}")]
pub async fn delete_user(id: web::Path<i32>) -> impl Responder {
    let ex_db = RB.acquire().await.expect("msg");

    let user_id = id.into_inner();
    let db_res: Option<UserEntity> = UserEntity::select_by_id(&ex_db, user_id)
        .await
        .expect("查询用户失败");

    match db_res {
        None => {
            return ResponseBody::error("用户不存在");
        }
        Some(mut db_user) => {
            let mut tx = get_transaction_tx().await.unwrap();
            db_user.status = Status::DEACTIVE as i16;
            let update_res = UserEntity::update_by_column(&tx, &db_user, "id").await;
            tx.commit().await.expect("msg");
            if let Err(rbs::Error::E(error)) = update_res {
                log::error!("更新用户失败, {}", error);
                let res = ResponseBody::error("更新用户失败");
                tx.rollback().await.expect("msg");
                return res;
            }
            //     TODOl: delete user from cache
        }
    }

    ResponseBody::success("删除用户成功")
}

#[utoipa::path(
    tag = "user",
    responses( (status = 200) )
  )]
#[get("/user_binds/{id}")]
pub async fn get_role_binds(parma: web::Path<i32>) -> impl Responder {
    let id = parma.into_inner();
    let db_user = check_user_by_user_id(id).await;
    if db_user.is_none() {
        return ResponseBody {
            code: 500,
            msg: "用户不存在".to_string(),
            data: None,
        };
    }
    let mut rds: std::cell::RefMut<'_, redis::Connection> = REDIS.inner.exclusive_access();
    let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), id);
    let cache_ids: Vec<i32> = rds.smembers(key).expect("获取角色绑定失败");

    let ex = RB.acquire().await.expect("msg");

    let roles = if cache_ids.is_empty() {
        let roles:Vec<RoleEntity> = ex.query_decode("select role.* from user_role left join role on user_role.role_id = role.id where user_id=? and role.status = 1;",vec![to_value!(id)]).await.expect("获取用户绑定角色失败");
        let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), id);
        for ele in roles.iter() {
            let _: () = rds
                .sadd(key.clone(), ele.id.unwrap())
                .expect("add new user_role error");
        }
        roles
    } else {
        let roles = RoleEntity::select_in_column(&ex, "id", &cache_ids)
            .await
            .expect("获取用户绑定角色失败");
        roles
    };

    let res = ResponseBody::default(Some(roles));
    res
}

#[utoipa::path(
    tag = "user",
    responses( (status = 200) )
  )]
#[post("/bind_role")]
pub async fn bind_role(req_data: web::Json<BindRoleData>) -> impl Responder {
    let check_res = check_role_exists(&req_data.role_id).await;
    let db_user = check_user_by_user_id(req_data.user_id).await;
    if check_res.is_none() {
        return ResponseBody::error("角色不存在");
    }
    if db_user.is_none() {
        return ResponseBody::error("用户不存在");
    }

    let mut tx = get_transaction_tx().await.expect("get tx error");
    let (add_ids, sub_ids) = check_bind(&req_data.user_id, &req_data.role_id);

    log::debug!("add_ids {add_ids:?}");
    log::debug!("sub_ids {sub_ids:?}");

    if !sub_ids.is_empty() {
        role_ids_to_sub_tab(&req_data.user_id, &sub_ids);

        for id in sub_ids {
            let sub_res: Result<Option<()>, rbs::Error> = tx
                .query_decode(
                    "delete from user_role where role_id=? and user_id = ?",
                    vec![to_value!(id), to_value!(req_data.user_id)],
                )
                .await;
            tx.commit().await.expect("msg");
            if let Err(rbs::Error::E(error)) = sub_res {
                log::error!("删除用户角色失败, {error}");
                let res = ResponseBody::error("删除用户角色失败");
                tx.rollback().await.expect("msg");
                return res;
            }
        }
    } else {
        if req_data.role_id.is_empty() {
            let sub_res = UserRoleEntity::delete_by_column(&tx, "user_id", req_data.user_id).await;
            tx.commit().await.expect("msg");
            if let Err(rbs::Error::E(error)) = sub_res {
                log::error!("删除用户角色失败, {error}");
                let res = ResponseBody::error("删除用户角色失败");
                tx.rollback().await.expect("msg");
                return res;
            }
        }
    }

    if !add_ids.is_empty() {
        let add_tabs: Vec<UserRoleEntity> = role_ids_to_add_tab(&req_data.user_id, &add_ids);
        log::debug!("add_tabs {add_tabs:#?}");
        let add_res = UserRoleEntity::insert_batch(&tx, &add_tabs, add_tabs.len() as u64).await;

        tx.commit().await.expect("msg");
        if let Err(rbs::Error::E(error)) = add_res {
            log::error!("绑定用户角色失败, {error}");
            let res = ResponseBody::error("绑定用户角色失败");
            tx.rollback().await.expect("msg");
            return res;
        }
    }

    ResponseBody::success("绑定成功")
}

#[utoipa::path(
    tag = "user",
    responses( (status = 200) )
  )]
#[get("/get_user_option")]
pub async fn get_user_option() -> Result<impl Responder, MyError> {
    let mut rds: std::cell::RefMut<'_, redis::Connection> = REDIS.inner.exclusive_access();
    let ids: Vec<i32> = rds.smembers("user_ids").expect("get user_id rds err");
    if !ids.is_empty() {
        let res: Vec<OptionData> = rds_str_to_list(rds, ids, RedisKeys::UserInfo, |val| {
            let user_data: OptionData = serde_json::from_str(&val).expect("msg");
            user_data
        });
        return Ok(ResponseBody::default(Some(res)));
    } else {
        let ex_db = RB.acquire().await.expect("get ex err");
        let opt: Vec<OptionData> = ex_db
            .query_decode("select id, name from user where status = 1", vec![])
            .await
            .expect("select db err");

        drop(rds);
        for ele in opt.iter() {
            sync_opt::sync(SyncOptData::default(
                RedisKeys::UserIds,
                RedisKeys::UserInfo,
                ele.id,
                ele.clone(),
            ))
            .await;
        }
        Ok(ResponseBody::default(Some(opt)))
    }
}
