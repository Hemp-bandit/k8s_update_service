use crate::{
    commom::CommListReq, entity::user_entity::UserEntity, response::ResponseBody, DataStore,
};
use actix_web::{post, web, Responder};
use rbatis::{Page, PageRequest};

#[utoipa::path(
    tag = "kaibai_user_service",
    responses(
        (status = 200, description = "List current todo items", )
    )
)]
#[post("/create_user")]
pub async fn create_user() -> impl Responder {
    let rsp = ResponseBody::default_as_string();
    rsp
}

#[utoipa::path(
    tag = "kaibai_user_service",
    responses(
        (status = 200, description = "get_user_list")
    )
)]
#[post("/get_user_list")]
pub async fn get_user_list(
    data_store: web::Data<DataStore>,
    req_data: web::Json<CommListReq>,
) -> impl Responder {
    let db_res = UserEntity::select_page(
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

    // data_store.db
    println!("req_data {:?}", req_data);

    res
}
