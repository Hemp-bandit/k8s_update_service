use actix_web::{body::BoxBody, http::header::ContentType, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseBody<T> {
    pub rsp_code: i8,
    pub rsp_msg: String,
    pub data: T,
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
