use actix_web::{body::BoxBody, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResponseBody<T> {
    pub code: i16,
    pub msg: String,
    pub data: T,
}

impl<T> ResponseBody<T> {
    pub fn default(data: Option<T>) -> ResponseBody<Option<T>> {
        ResponseBody {
            code: 0,
            msg: "".to_string(),
            data,
        }
    }
    pub fn default_err(msg: &str) -> ResponseBody<Option<T>> {
        ResponseBody {
            code: 0,
            msg: msg.to_string(),
            data: None,
        }
    }
}
impl ResponseBody<String> {
    pub fn error(msg: &str) -> ResponseBody<Option<String>> {
        ResponseBody {
            code: 500,
            msg: msg.to_string(),
            data: None,
        }
    }

    pub fn success(msg: &str) -> ResponseBody<Option<String>> {
        ResponseBody {
            code: 0,
            msg: msg.to_string(),
            data: None,
        }
    }
}
impl<T: Serialize> Responder for ResponseBody<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        HttpResponse::Ok().force_close().json(&self)
    }
}
