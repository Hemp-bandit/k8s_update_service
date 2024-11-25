use std::ops::Add;

use actix_web::{get, web, Responder};
use rbs::to_value;

use crate::{
    access::AccessValueData, response::ResponseBody, role::AccessData, user::check_user, RB,
};

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
            return   ResponseBody {
                code: 500,
                msg: "用户不存在".to_string(),
                data: None,
            };
        }
        Some(_db_user) => {
            let ex = RB.acquire().await.expect("msg");
            let access_ids: Option<Vec<AccessData>> = ex
                .query_decode(
                    "select role_access.access_id from role_access where role_id in (select user_role.role_id from user_role where user_id=?);",
                    vec![to_value!(user_id)],
                )
                .await
                .expect("查询权限id错误");

            println!("res {access_ids:?}");
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
