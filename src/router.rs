use crate::response::ResponseBody;
use actix_web::{post, Responder};

#[post("/update_deployment")]
pub async fn update_deployment() -> impl Responder {
    let mut res = ResponseBody {
        rsp_code: 0,
        rsp_msg: "".to_string(),
        data: "".to_string(),
    };

    res.data = "asdfasdf".to_string();

    res
}
