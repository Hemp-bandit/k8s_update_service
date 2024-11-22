use actix_web::{post, web, Responder};
use rbatis::{Page, PageRequest};

use super::{AccessListQuery, AccessUpdateData, CreateAccessData};
use crate::{
    access::check_access_by_id,
    common::{get_current_time_fmt, get_transaction_tx, Status},
    entity::access_entity::AccessEntity,
    response::ResponseBody,
    user::check_user,
    RB,
};

#[utoipa::path(
    tag = "access",
    responses( (status = 200))
)]
#[post("/create_access")]
async fn create_access(req_data: web::Json<CreateAccessData>) -> impl Responder {
    if check_user(req_data.create_by).await.is_none() {
        return ResponseBody::error("用户不存在");
    }

    let new_role = AccessEntity {
        id: None,
        create_time: get_current_time_fmt(),
        update_time: get_current_time_fmt(),
        name: req_data.name.clone(),
        create_by: req_data.create_by,
        status: Status::ACTIVE as i16,
    };

    let mut tx = get_transaction_tx().await.unwrap();
    let insert_res = AccessEntity::insert(&tx, &new_role).await;
    tx.commit().await.expect("commit error");

    if let Err(rbs::Error::E(error)) = insert_res {
        let res = ResponseBody::error("创建权限失败");
        log::error!(" 创建权限失败 {}", error);
        tx.rollback().await.expect("rollback error");
        return res;
    }

    ResponseBody::success("权限创建成功")
}

#[utoipa::path(
    tag = "access",
    responses( (status = 200))
)]
#[post("/get_access_list")]
async fn get_access_list(req_data: web::Json<AccessListQuery>) -> impl Responder {
    let ex_db = RB.acquire().await.expect("msg");
    let db_res: Page<AccessEntity> = match &req_data.name {
        Some(name) => AccessEntity::select_page_by_name(
            &ex_db,
            &PageRequest::new(req_data.page_no as u64, req_data.take as u64),
            &name,
        )
        .await
        .expect("msg"),
        None => AccessEntity::select_page(
            &ex_db,
            &PageRequest::new(req_data.page_no as u64, req_data.take as u64),
        )
        .await
        .expect("msg"),
    };
    ResponseBody::default(Some(db_res))
}

#[utoipa::path(
    tag = "access",
    responses( (status = 200))
)]
#[post("/update_access_by_id")]
pub async fn update_access_by_id(req_data: web::Json<AccessUpdateData>) -> impl Responder {
    match check_access_by_id(req_data.id).await {
        None => {
            return ResponseBody::error("权限不存在");
        }
        Some(mut access) => {
            access.name = req_data.name.clone().unwrap_or(access.name);
            access.update_time = get_current_time_fmt();
            if let Some(status) = req_data.status.clone() {
                // 任何非法值会将状态置为无效
                let st = Status::from(status);
                access.status = st as i16;
            }
            let mut tx = get_transaction_tx().await.expect("get tx err");
            let update_res = AccessEntity::update_by_column(&tx, &access, "id").await;
            tx.commit().await.expect("msg");

            if let Err(rbs::Error::E(error)) = update_res {
                log::error!("更新权限失败, {}", error);
                let res = ResponseBody::error("更新权限失败");
                tx.rollback().await.expect("msg");
                return res;
            }
        }
    }

    ResponseBody::success("权限更新成功")
}
