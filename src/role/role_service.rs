use actix_web::{delete, get, post, web, Responder};
use rbatis::{Page, PageRequest};

use super::{BindAccessData, CreateRoleData, RoleListQuery, RoleUpdateData};
use crate::{
    access::check_access_by_id,
    common::{get_current_time_fmt, get_transaction_tx, Status},
    entity::{role_access_entity::RoleAccessEntity, role_entity::RoleEntity},
    response::ResponseBody,
    role::{check_role_access, check_role_by_id},
    user::check_user,
    RB,
};

#[utoipa::path(
  tag = "role",
  responses( (status = 200) )
)]
#[post("/create_role")]
async fn create_role(req_data: web::Json<CreateRoleData>) -> impl Responder {
    if check_user(req_data.create_by).await.is_none() {
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
async fn get_role_list(req_data: web::Json<RoleListQuery>) -> impl Responder {
    let ex_db = RB.acquire().await.expect("msg");
    let db_res: Page<RoleEntity> = match &req_data.name {
        Some(name) => RoleEntity::select_page_by_name(
            &ex_db,
            &PageRequest::new(req_data.page_no as u64, req_data.take as u64),
            &name,
        )
        .await
        .expect("msg"),
        None => RoleEntity::select_page(
            &ex_db,
            &PageRequest::new(req_data.page_no as u64, req_data.take as u64),
        )
        .await
        .expect("msg"),
    };
    ResponseBody::default(Some(db_res))
}

#[utoipa::path(
    tag = "role",
    responses( (status = 200) )
  )]
#[post("/update_role_by_id")]
pub async fn update_role_by_id(req_data: web::Json<RoleUpdateData>) -> impl Responder {
    match check_role_by_id(req_data.id).await {
        None => {
            return ResponseBody::error("角色不存在");
        }
        Some(mut role) => {
            role.name = req_data.name.clone().unwrap_or(role.name);
            role.update_time = get_current_time_fmt();
            if let Some(status) = req_data.status.clone() {
                // 任何非法值会将状态置为无效
                let st = Status::from(status);
                role.status = st as i8;
            }
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
