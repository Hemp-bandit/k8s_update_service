use actix_web::{get, post, web, Responder};
use rbs::to_value;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    access::AccessValueData, response::ResponseBody, role::AccessData, user::check_user, RB,
};

use super::LoginData;

#[utoipa::path(
  tag = "auth",
  responses( (status = 200))
)]
#[get("/get_user_permission/{user_id}")]
async fn get_user_permission(path: web::Path<i32>) -> impl Responder {
    let user_id = path.into_inner();
    let mut vals: u64 = 0;
    match check_user(user_id).await {
        None => {
            return ResponseBody {
                code: 500,
                msg: "用户不存在".to_string(),
                data: None,
            };
        }
        Some(_db_user) => {
            let ex = RB.acquire().await.expect("get ex error");
            let access_ids: Option<Vec<AccessData>> = ex
                .query_decode(
                    "select role_access.access_id from role_access where role_id in (select user_role.role_id from user_role where user_id=?);",
                    vec![to_value!(user_id)],
                )
                .await
                .expect("查询权限id错误");

            if let Some(access_id_vec) = access_ids {
                let ids: Vec<String> = access_id_vec
                    .into_iter()
                    .map(|val| val.access_id.to_string())
                    .collect();
                let ids = ids.join(",");
                let access_values: Option<Vec<AccessValueData>> = ex
                    .query_decode(
                        "select value from access where id in (?)",
                        vec![to_value!(ids)],
                    )
                    .await
                    .expect("查询权限值错误");

                println!("access_values {access_values:?}");
                if let Some(access_values) = access_values {
                    access_values.into_iter().for_each(|val| {
                        vals += val.value;
                    });
                }
            }
        }
    }
    ResponseBody::default(Some(vals))
}

#[utoipa::path(
    tag = "auth",
    responses( (status = 200) )
)]
#[post("/login")]
async fn login(req_data: web::Json<LoginData>) -> impl Responder {
    let ex = RB.acquire().await.expect("msg");
    #[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
    struct PasswordData {
        password: String,
    }
    let db_user: Option<PasswordData> = ex
        .query_decode(
            "select password from user where user.name=?",
            vec![to_value!(req_data.name.clone())],
        )
        .await
        .expect("获取用户失败");

    match db_user {
        None => {
            return ResponseBody::error("用户不存在");
        }
        Some(db_user) => {
            let is_eq = db_user.password.eq(&req_data.password);
            if !is_eq {
                return ResponseBody::error("密码错误");
            }
        }
    }

    ResponseBody::success("登录成功")
}
