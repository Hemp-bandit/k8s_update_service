use actix_web::{post, web, Responder};

use super::CreateRoleData;
use crate::{response::ResponseBody, DataStore};

#[utoipa::path(
  tag = "role",
  responses( (status = 200) )
)]
#[post("/create_role")]
async fn create_role(
    req_data: web::Json<CreateRoleData>,
    data_store: web::Data<DataStore>,
) -> impl Responder {
    let res: ResponseBody<Option<String>> = ResponseBody::default(None);


    // let mut tx = data_store.db.acquire_begin().await.unwrap();



    res
}
