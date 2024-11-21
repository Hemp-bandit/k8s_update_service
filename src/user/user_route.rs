use super::{UserCreateData, UserUpdateData};
use crate::{
    common::{check_phone, get_current_time_fmt, get_transaction_tx, CommListReq, Status, UserType},
    entity::user_entity::UserEntity,
    response::ResponseBody, RB,
};
use actix_web::{get, post, web, Responder};
use rbatis::{Page, PageRequest};

#[utoipa::path(
    tag = "user",
    responses( (status = 200) )
)]
#[post("/create_user")]
pub async fn create_user(
    req_data: web::Json<UserCreateData>,
) -> impl Responder {
    let mut rsp: ResponseBody<Option<String>> = ResponseBody::default(None);
    // check phone

    let phone_check_res = check_phone(&req_data.phone);
    if !phone_check_res {
        rsp.code = 500;
        rsp.msg = "手机号不正确".to_string();
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
        user_type: UserType::BIZ as i16,
        status: Status::ACTIVE as i16,
    };

    let mut tx = get_transaction_tx().await.unwrap();
    let insert_res = UserEntity::insert(&tx, &insert_user).await;
    tx.commit().await.expect("commit transaction error ");
    if let Err(rbs::Error::E(error)) = insert_res {
        rsp.code = 500;
        rsp.msg = "创建用户失败".to_string();
        log::error!(" 创建用户失败 {}", error);
        tx.rollback().await.expect("rollback error");
        return rsp;
    }

    rsp.code = 0;
    rsp.msg = "创建用户成功".to_string();

    rsp
}

#[utoipa::path(
    tag = "user",
    responses( (status = 200) )
)]
#[post("/get_user_list")]
pub async fn get_user_list(
    req_data: web::Json<CommListReq>,
) -> impl Responder {
  let ex_db = RB.acquire().await.expect("msg");
    let db_res: Page<UserEntity> = UserEntity::select_page(
        &ex_db,
        &PageRequest::new(req_data.page_no as u64, req_data.take as u64),
    )
    .await
    .expect("msg");

    ResponseBody::default(Some(db_res))
}

#[utoipa::path(
    tag = "user",
    params(("id", description = "user id") ),
    responses( (status = 200) )
)]
#[get("/{id}")]
pub async fn get_user_by_id(
    id: web::Path<i32>,
) -> impl Responder {
    let mut res: ResponseBody<Option<UserEntity>> = ResponseBody::default(None);
    let ex_db = RB.acquire().await.expect("msg");
    let user_id = id.into_inner();
    let db_res: Option<UserEntity> = UserEntity::select_by_id(&ex_db, user_id)
        .await
        .expect("查询用户失败");

    res.data = db_res;

    res
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
    let mut res: ResponseBody<Option<String>> = ResponseBody::default(None);

    if let Some(new_phone) = &req_data.phone {
        let phone_check_res = check_phone(new_phone);
        if !phone_check_res {
            res.code = 500;
            res.msg = "手机号不正确".to_string();
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
            res.code = 500;
            res.msg = "用户不存在".to_string();
            return res;
        }
        Some(mut db_user) => {
            db_user.update_time = get_current_time_fmt();
            db_user.introduce = req_data.introduce.clone();
            db_user.name = req_data.name.clone().unwrap_or(db_user.name);
            db_user.password = req_data.password.clone().unwrap_or(db_user.password);
            db_user.picture = req_data.picture.clone();
            db_user.phone = req_data.phone.clone().unwrap_or(db_user.phone);

            let mut tx = get_transaction_tx().await.unwrap();
            let update_res = UserEntity::update_by_column(&tx, &db_user, "id").await;
            tx.commit().await.expect("msg");
            if let Err(rbs::Error::E(error)) = update_res {
                log::error!("更新用户失败, {}", error);
                res.code = 500;
                res.msg = "更新用户失败".to_string();
                tx.rollback().await.expect("msg");
                return res;
            }
        }
    }
    res.code = 0;
    res.msg = "更新用户成功".to_string();
    res
}
