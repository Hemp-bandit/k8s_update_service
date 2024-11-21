use actix_web::{post, web, Responder};
use rbatis::{Page, PageRequest};

use super::{CreateRoleData, RoleListQuery};
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
