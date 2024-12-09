use super::{BindRoleData, UserCreateData, UserListQuery, UserUpdateData};
use crate::entity::role_entity::RoleEntity;
use crate::response::MyError;
use crate::user::user_role_service::{
    bind_user_role, check_role_exists, check_user_role_bind, sync_user_auth, unbind_role_from_cache,
};
use crate::util::common::{rds_str_to_list, RedisKeys};
use crate::util::redis_actor::{HsetData, SaddData, SmembersData};
use crate::util::sync_opt::DelOptData;
use crate::REDIS_ADDR;
use crate::{
    entity::{user_entity::UserEntity, user_role_entity::UserRoleEntity},
    response::ResponseBody,
    user::{check_user_by_user_id, OptionData},
    util::{
        common::{check_phone, get_transaction_tx},
        structs::Status,
        sync_opt::{self, SyncOptData},
    },
    RB,
};
use actix_web::{delete, get, post, web, Responder};
use rbs::to_value;
use rs_service_util::sql_tool::{SqlTool, SqlToolPageData};
use rs_service_util::time::get_current_time_fmt;
#[utoipa::path(
    tag = "user",
    responses( (status = 200) )
)]
#[post("/create_user")]
pub async fn create_user(req_data: web::Json<UserCreateData>) -> Result<impl Responder, MyError> {
    let phone_check_res = check_phone(&req_data.phone);
    if !phone_check_res {
        return Err(MyError::PhoneIsError);
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

    let tx = get_transaction_tx().await.unwrap();
    let insert_res = UserEntity::insert(&tx, &insert_user).await;
    tx.commit().await.expect("commit transaction error ");
    match insert_res {
        Err(rbs::Error::E(error)) => {
            log::error!(" {} {}", error, MyError::CreateUserError);
            tx.rollback().await.expect("rollback error");
            return Err(MyError::CreateUserError);
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

    Ok(ResponseBody::success("创建用户成功"))
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
pub async fn get_user_by_id(id: web::Path<i32>) -> Result<impl Responder, MyError> {
    let ex_db = RB.acquire().await.expect("msg");
    let user_id = id.into_inner();
    let db_res: Option<UserEntity> = UserEntity::select_by_id(&ex_db, user_id)
        .await
        .expect("查询用户失败");

    Ok(ResponseBody::default(Some(db_res)))
}

#[utoipa::path(
    tag = "user",
    params(("id", description = "user id") ),
    responses( (status = 200) )
)]
#[post("/update_user/{id}")]
pub async fn update_user_by_id(
    id: web::Path<i32>,
    req_data: web::Json<UserUpdateData>,
) -> Result<impl Responder, MyError> {
    if let Some(new_phone) = &req_data.phone {
        let phone_check_res = check_phone(new_phone);
        if !phone_check_res {
            return Err(MyError::PhoneIsError);
        }
    }

    let tx = get_transaction_tx().await.unwrap();

    let user_id = id.into_inner();
    let db_res: Option<UserEntity> = UserEntity::select_by_id(&tx, user_id)
        .await
        .expect("查询用户失败");

    match db_res {
        None => {
            return Err(MyError::UserNotExist);
        }
        Some(mut db_user) => {
            log::debug!("db_user {db_user:?}");
            db_user.update_time = get_current_time_fmt();
            db_user.introduce = req_data.introduce.clone();
            db_user.name = req_data.name.clone().unwrap_or(db_user.name);
            db_user.password = req_data.password.clone().unwrap_or(db_user.password);
            db_user.picture = req_data.picture.clone();
            db_user.phone = req_data.phone.clone().unwrap_or(db_user.phone);
            db_user.user_type = req_data.user_type.clone().unwrap_or(db_user.user_type);

            let update_res: Result<Option<()>, rbs::Error> =
                tx.query_decode(
                    "update user set user.update_time=?, introduce=?, name=?, password=?, picture=?, phone=?, user_type=? where id=? ",
                    vec![ to_value!(db_user.update_time),to_value!(db_user.introduce),to_value!(db_user.name.clone()), to_value!(db_user.password.clone()), to_value!(db_user.picture), to_value!(db_user.phone.clone()), to_value!(db_user.user_type) ,to_value!(db_user.id.unwrap())]
                )
                 .await;
            tx.commit().await.expect("msg");
            if let Err(rbs::Error::E(error)) = update_res {
                log::error!("{} {}", error, MyError::UpdateUserError);
                tx.rollback().await.expect("msg");
                return Err(MyError::UpdateUserError);
            }
            let opt = OptionData::default(&db_user.name, db_user.id.clone().expect("msg"));
            let _ = REDIS_ADDR
                .get()
                .expect("get redis addr error")
                .send(HsetData {
                    key: RedisKeys::UserInfo.to_string(),
                    opt_data: serde_json::to_string(&opt).expect("msg"),
                    id: db_user.id.clone().expect("msg"),
                })
                .await;
        }
    }
    Ok(ResponseBody::success("更新用户成功"))
}

#[utoipa::path(
    tag = "user",
    params(("id", description = "user id") ),
    responses( (status = 200) )
)]
#[delete("/delete_user/{id}")]
pub async fn delete_user(id: web::Path<i32>) -> Result<impl Responder, MyError> {
    let tx = get_transaction_tx().await.unwrap();
    let user_id = id.into_inner();
    let db_res: Option<UserEntity> = UserEntity::select_by_id(&tx, user_id)
        .await
        .expect("查询用户失败");

    match db_res {
        None => {
            return Err(MyError::UserNotExist);
        }
        Some(mut db_user) => {
            db_user.status = Status::DEACTIVE as i16;
            let update_res: Result<Option<()>, rbs::Error> = tx
                .query_decode(
                    "update user set user.status = ? where user.id = ?",
                    vec![to_value!(db_user.status), to_value!(db_user.id.unwrap())],
                )
                .await;
            tx.commit().await.expect("msg");
            if let Err(rbs::Error::E(error)) = update_res {
                log::error!("{} {}", error, MyError::UpdateUserError);
                tx.rollback().await.expect("msg");
                return Err(MyError::UpdateUserError);
            }
        }
    }

    sync_opt::del(DelOptData::default(
        RedisKeys::RoleIds,
        RedisKeys::RoleInfo,
        vec![user_id],
    ))
    .await;

    Ok(ResponseBody::success("删除用户成功"))
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

    let rds = REDIS_ADDR.get().expect("msg");
    let key: String = format!("{}_{}", RedisKeys::UserRoles.to_string(), id);
    let cache_ids: Vec<i32> = rds
        .send(SmembersData { key })
        .await
        .expect("获取角色绑定失败")
        .expect("msg");

    let ex = RB.acquire().await.expect("msg");

    let roles = if cache_ids.is_empty() {
        let roles:Vec<RoleEntity> = ex.query_decode("select role.* from user_role left join role on user_role.role_id = role.id where user_id=? and role.status = 1;",vec![to_value!(id)]).await.expect("获取用户绑定角色失败");
        let key = format!("{}_{}", RedisKeys::UserRoles.to_string(), id);
        for ele in roles.iter() {
            let _ = rds
                .send(SaddData {
                    key: key.clone(),
                    id: ele.id.unwrap(),
                })
                .await
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
pub async fn bind_role(req_data: web::Json<BindRoleData>) -> Result<impl Responder, MyError> {
    let check_res = check_role_exists(&req_data.role_id).await;
    let db_user = check_user_by_user_id(req_data.user_id).await;
    if check_res.is_none() {
        return Err(MyError::RoleNotExist);
    }
    if db_user.is_none() {
        return Err(MyError::UserNotExist);
    }

    let tx = get_transaction_tx().await.expect("get tx error");
    let (add_ids, sub_ids) = check_user_role_bind(&req_data.user_id, &req_data.role_id).await;

    log::debug!("add_ids {add_ids:?}");
    log::debug!("sub_ids {sub_ids:?}");

    if !sub_ids.is_empty() {
        unbind_role_from_cache(&req_data.user_id, &sub_ids).await;

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
                tx.rollback().await.expect("msg");
                return Err(MyError::DelUserRoleError);
            }
        }
    } else {
        if req_data.role_id.is_empty() {
            let sub_res = UserRoleEntity::delete_by_column(&tx, "user_id", req_data.user_id).await;
            tx.commit().await.expect("msg");
            if let Err(rbs::Error::E(error)) = sub_res {
                log::error!("删除用户角色失败, {error}");
                tx.rollback().await.expect("msg");
                return Err(MyError::DelUserRoleError);
            }
        }
    }

    if !add_ids.is_empty() {
        let add_tabs: Vec<UserRoleEntity> = bind_user_role(&req_data.user_id, &add_ids).await;
        log::debug!("add_tabs {add_tabs:#?}");
        let add_res = UserRoleEntity::insert_batch(&tx, &add_tabs, add_tabs.len() as u64).await;

        tx.commit().await.expect("msg");
        if let Err(rbs::Error::E(error)) = add_res {
            log::error!("绑定用户角色失败, {error}");
            tx.rollback().await.expect("msg");
            return Err(MyError::BindUserRoleError);
        }
    }
    sync_user_auth(db_user.unwrap().name).await?;
    Ok(ResponseBody::success("绑定成功"))
}

#[utoipa::path(
    tag = "user",
    responses( (status = 200) )
  )]
#[get("/get_user_option")]
pub async fn get_user_option() -> Result<impl Responder, MyError> {
    let rds = REDIS_ADDR.get().expect("msg");
    let ids: Vec<i32> = rds
        .send(SmembersData {
            key: RedisKeys::UserIds.to_string(),
        })
        .await
        .expect("get user_id rds err")
        .expect("get user_id rds err");
    if !ids.is_empty() {
        let res: Vec<OptionData> = rds_str_to_list(ids, RedisKeys::UserInfo, |val| {
            let user_data: OptionData = serde_json::from_str(&val).expect("msg");
            user_data
        })
        .await;
        return Ok(ResponseBody::default(Some(res)));
    } else {
        let ex_db = RB.acquire().await.expect("get ex err");
        let opt: Vec<OptionData> = ex_db
            .query_decode("select id, name from user where status = 1", vec![])
            .await
            .expect("select db err");

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
