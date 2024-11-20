use actix_web::{post, web, Responder};

use super::CreateRoleData;
use crate::{entity::role_entity::RoleEntity, response::ResponseBody, DataStore};

#[utoipa::path(
  tag = "role",
  responses( (status = 200) )
)]
#[post("/create_role")]
async fn create_role(
    req_data: web::Json<CreateRoleData>,
    data_store: web::Data<DataStore>,
) -> impl Responder {
    let mut res: ResponseBody<Option<String>> = ResponseBody::default(None);
    let db_role = RoleEntity::select_by_name(&data_store.db,&req_data.name ).await.expect("获取角色失败");
    if db_role.is_some(){
      res.rsp_code = 500;
      res.rsp_msg = "角色已存在".to_string();
    }
    // let mut tx = data_store.db.acquire_begin().await.unwrap();





    res
}
