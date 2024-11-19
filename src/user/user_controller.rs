use actix_web::{post, Responder};
use crate::response::ResponseBody;

#[utoipa::path(
    tag = "kaibai_user_service",
    responses(
        (status = 200, description = "create_user", body=[ResponseBody<String>])
    )
)]
#[post("/create_user")]
pub async fn create_user() -> impl Responder {
    let rsp = ResponseBody::default_as_string();

    rsp
}
