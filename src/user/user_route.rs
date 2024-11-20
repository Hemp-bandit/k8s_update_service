use super::UserCreateData;
use crate::{
    common::{get_current_time_fmt, CommListReq},
    entity::user_entity::{Status, UserEntity, UserType},
    response::ResponseBody,
    DataStore,
};
use actix_web::{get, post, web, Responder};
use rbatis::{dark_std::err, Page, PageRequest};

#[utoipa::path(
    tag = "kaibai_user_service",
    responses( (status = 200) )
)]
#[post("/create_user")]
pub async fn create_user(
    req_data: web::Json<UserCreateData>,
    data_store: web::Data<DataStore>,
) -> impl Responder {
    let mut rsp: ResponseBody<Option<String>> = ResponseBody::default(None);
    let db_res: Option<UserEntity> = UserEntity::select_by_name_phone(&data_store.db, &req_data.name, &req_data.phone)
        .await
        .expect("获取用户失败");

    if db_res.is_some() {
        rsp.rsp_msg = "用户已存在".to_string();
        rsp.rsp_code = 500;
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

    let insert_res: Result<rbatis::rbdc::db::ExecResult, rbs::Error> =
        UserEntity::insert(&data_store.db, &insert_user).await;

    if let Err(rbs::Error::E(error)) = insert_res {
        rsp.rsp_code = 500;
        rsp.rsp_msg = "创建用户失败".to_string();
        err!(" 创建用户失败 {}", error);
        return rsp;
    }

    rsp.rsp_code = 0;
    rsp.rsp_msg = "创建用户成功".to_string();

    rsp
}

#[utoipa::path(
    tag = "kaibai_user_service",
    responses( (status = 200) )
)]
#[post("/get_user_list")]
pub async fn get_user_list(
    data_store: web::Data<DataStore>,
    req_data: web::Json<CommListReq>,
) -> impl Responder {
    let db_res: Page<UserEntity> = UserEntity::select_page(
        &data_store.db,
        &PageRequest::new(req_data.offset as u64, req_data.take as u64),
    )
    .await
    .expect("msg");

    let res: ResponseBody<Page<UserEntity>> = ResponseBody {
        rsp_code: 0,
        rsp_msg: "".to_string(),
        data: db_res,
    };

    res
}

#[utoipa::path(
    tag = "kaibai_user_service",
    params(("id", description = "user id") ),
    responses( (status = 200) )
)]
#[get("/{id}")]
pub async fn get_user_by_id(
    id: web::Path<i32>,
    data_store: web::Data<DataStore>,
) -> impl Responder {
    let mut res: ResponseBody<Option<UserEntity>> = ResponseBody::default(None);

    let user_id = id.into_inner();
    let db_res: Option<UserEntity> = UserEntity::select_by_id(&data_store.db, user_id)
        .await
        .expect("查询用户失败");

    res.data = db_res;

    res
}
