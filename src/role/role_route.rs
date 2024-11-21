use actix_web::{post, web, Responder};
use rbatis::{Page, PageRequest};

use super::{CreateRoleData, RoleListQuery, RoleUpdateData};
use crate::{
    common::{get_current_time_fmt, get_transaction_tx, Status},
    entity::role_entity::RoleEntity,
    response::ResponseBody,
    RB,
};

#[utoipa::path(
  tag = "role",
  responses( (status = 200) )
)]
#[post("/create_role")]
async fn create_role(req_data: web::Json<CreateRoleData>) -> impl Responder {
    let mut res: ResponseBody<Option<String>> = ResponseBody::default(None);

    let new_role = RoleEntity {
        id: None,
        create_time: get_current_time_fmt(),
        update_time: get_current_time_fmt(),
        name: req_data.name.clone(),
        create_by: req_data.create_by,
        status: Status::ACTIVE as i16,
    };

    let mut tx = get_transaction_tx().await.unwrap();
    let insert_res = RoleEntity::insert(&tx, &new_role).await;
    tx.commit().await.expect("commit error");

    if let Err(rbs::Error::E(error)) = insert_res {
        res.code = 500;
        res.msg = "创建角色失败".to_string();
        log::error!(" 创建角色失败 {}", error);
        tx.rollback().await.expect("rollback error");
        return res;
    }

    res.code = 0;
    res.msg = "角色创建成功".to_string();
    res
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
    let ex_db = RB.acquire().await.expect("get db ex error");
    let db_role = RoleEntity::select_by_id(&ex_db, req_data.id.clone())
        .await
        .expect("角色不存在");

    match db_role {
        None => {
            return ResponseBody::error("角色不存在");
        }
        Some(mut role) => {
            role.name = req_data.name.clone().unwrap_or(role.name);

            if let Some(status) = req_data.status.clone() {
                // 任何非法值会将状态置为无效
                let st = Status::from(status);
                role.status = st as i16;
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

    ResponseBody::default(Some("角色更新成功".to_string()))
}
