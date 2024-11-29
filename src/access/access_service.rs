use super::{AccessListQuery, AccessUpdateData, CreateAccessData};
use crate::{
    access::check_access_by_id,
    entity::access_entity::AccessEntity,
    response::ResponseBody,
    user::check_user_by_user_id,
    util::{
        common::{gen_access_value, get_current_time_fmt, get_transaction_tx},
        structs::Status,
    },
    RB,
};
use actix_web::{post, web, Responder};
use rbatis::{Page, PageRequest};

#[utoipa::path(
    tag = "access",
    responses( (status = 200))
)]
#[post("/create_access")]
async fn create_access(req_data: web::Json<CreateAccessData>) -> impl Responder {
    if check_user_by_user_id(req_data.create_by).await.is_none() {
        return ResponseBody::error("用户不存在");
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

    let mut tx = get_transaction_tx().await.unwrap();
    let insert_res = AccessEntity::insert(&tx, &new_access).await;
    tx.commit().await.expect("commit error");
    match insert_res {
        Err(rbs::Error::E(error)) => {
            let res = ResponseBody::error("创建权限失败");
            log::error!(" 创建权限失败 {}", error);
            tx.rollback().await.expect("rollback error");
            return res;
        }
        Ok(res) => {
            let permission = gen_access_value(res.last_insert_id.as_u64().unwrap_or(0));
            let mut update_access = new_access.clone();
            update_access.id = Some(res.last_insert_id.as_i64().unwrap_or(0).try_into().unwrap());
            update_access.value = permission;
            update_access.update_time = get_current_time_fmt();

            let update_res = AccessEntity::update_by_column(&tx, &update_access, "id").await;
            if let Err(rbs::Error::E(error)) = update_res {
                let res = ResponseBody::error("更新权限值失败");
                log::error!(" 更新权限值失败 {}", error);
                tx.rollback().await.expect("rollback error");
                return res;
            }
        }
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
#[post("/update_access")]
pub async fn update_access_by_id(req_data: web::Json<AccessUpdateData>) -> impl Responder {
    match check_access_by_id(req_data.id).await {
        None => {
            return ResponseBody::error("权限不存在");
        }
        Some(mut access) => {
            access.name = req_data.name.clone().unwrap_or(access.name);
            access.update_time = get_current_time_fmt();
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
