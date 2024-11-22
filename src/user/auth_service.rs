use actix_web::{get, web, Responder};
use rbs::Value;

use crate::{response::ResponseBody, user::check_user, RB};

#[utoipa::path(
  tag = "auth",
  responses( (status = 200))
)]
#[get("/get_user_permission/{user_id}")]
async fn get_user_permission(path: web::Path<i32>) -> impl Responder {
    let user_id = path.into_inner();

    match check_user(user_id).await {
        None => {
            return ResponseBody::error("用户不存在");
        }
        Some(db_user) => {
            let ex = RB.acquire().await.expect("msg");
            let res: Option<Vec<i32>> = ex
                .query_decode(
                    "select role_id form user_role where  user_id = ?",
                    vec![Value::I32(db_user.id.expect("msg"))],
                )
                .await
                .expect("msg");
            println!("res {:?}", res);
            // let roles = UserRoleEntity::select_by_column(&ex, "user_id", db_user.id.clone())
            // .await
            // .expect("获取用户角色错误");
            // roles.into_iter().for_each(|role_id| {
            //   UserRoleEntity::select_in_column(&ex, "role_id", column_values)
            // });
        }
    }

    ResponseBody::success("msg")
}
