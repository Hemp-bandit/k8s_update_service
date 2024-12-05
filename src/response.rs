use actix_web::{
    body::BoxBody,
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse, Responder,
};
use derive_more::derive::{Display, Error};
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

#[derive(Debug, Display, Error)]
pub enum MyError {
    // #[display("internal error")]
    // InternalError = 0,
    #[display("用户不存在")]
    UserNotExist,

    #[display("角色不存在")]
    RoleNotExist,

    #[display("权限不存在")]
    AccessNotExist,

    #[display("用户不正确")]
    UserIsWrong,

    #[display("密码错误")]
    PassWordError,

    #[display("权限验证失败")]
    AuthError,

    #[display("删除角色权限失败")]
    DelRoleAccessError,

    #[display("删除用户角色失败")]
    DelUserRoleError,

    #[display("绑定用户角色失败")]
    BindUserRoleError,

    #[display("更新角色失败")]
    UpdateRoleError,

    #[display("创建角色失败")]
    CreateRoleError,

    #[display("创建权限失败")]
    CreateAccessError,

    #[display("更新权限失败")]
    UpdateAccessError,

    #[display("删除权限失败")]
    DeleteAccessError,

    #[display("手机号不正确")]
    PhoneIsError,

    #[display("创建用户失败")]
    CreateUserError,

    #[display("更新用户失败")]
    UpdateUserError,
}

impl error::ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        let rsp_data = match self {
            MyError::AuthError => {
                let res: ResponseBody<Option<String>> = ResponseBody {
                    code: StatusCode::UNAUTHORIZED.as_u16() as i16,
                    msg: self.to_string(),
                    data: None,
                };
                res
            }
            _ => ResponseBody::error(&self.to_string()),
        };

        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .insert_header(("Access-Control-Allow-Origin", "*"))
            .json(rsp_data)
    }
    fn status_code(&self) -> StatusCode {
        match self {
            MyError::AuthError => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
