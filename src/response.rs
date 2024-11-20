use actix_web::{body::BoxBody, http::header::ContentType, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResponseBody<T> {
    pub rsp_code: i16,
    pub rsp_msg: String,
    pub data: T,
}

impl<T> ResponseBody<T> {
    pub fn default(data: Option<T>) -> ResponseBody<Option<T>> {
        ResponseBody {
            rsp_code: 0,
            rsp_msg: "".to_string(),
            data,
        }
    }
}

impl<T: Serialize> Responder for ResponseBody<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}
