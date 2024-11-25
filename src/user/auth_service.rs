use crate::{
    access::AccessValueData,
    common::{gen_jwt_token, get_current_timestamp},
    response::ResponseBody,
    role::AccessData,
    user::{check_user, RedisLoginData},
    RB, REDIS,
};
use actix_web::{get, post, web, Responder};
use rbs::to_value;
use redis::Commands;
use serde::{Deserialize, Serialize};

use super::LoginData;

lazy_static::lazy_static! {
    static ref REDIS_KEY:String = "user_service".to_string();
}
const LOGIN_EX_TIME: u64 = 60 * 60 * 24 * 10;

#[derive(Serialize, Deserialize)]
struct PasswordData {
    password: String,
    id: i32,
}

#[utoipa::path(
  tag = "auth",
  responses( (status = 200))
)]
#[get("/get_user_permission/{user_id}")]
async fn get_user_permission(path: web::Path<i32>) -> impl Responder {
    let user_id = path.into_inner();

    let check_res = check_user(user_id).await;
    if check_res.is_none() {
        return ResponseBody {
            code: 500,
            msg: "用户不存在".to_string(),
            data: None,
        };
    }

    let vals = get_user_access_val(user_id).await;
    ResponseBody::default(Some(vals))
}

#[utoipa::path(
    tag = "auth",
    responses( (status = 200) )
)]
#[post("/login")]
async fn login(req_data: web::Json<LoginData>) -> impl Responder {
    let key = format!("{}_{}", REDIS_KEY.to_string(), req_data.name.clone());
    let mut rds = REDIS.lock().unwrap();
    let redis_login: Result<bool, redis::RedisError> = rds.exists(key.clone());

    let is_login = match redis_login {
        Err(err) => {
            let detail = err.detail().expect("msg");
            log::error!("{}", detail);
            return ResponseBody::error(detail);
        }
        Ok(res) => res,
    };

    if is_login {
        let user_info: RedisLoginData = rds.get(key).expect("get user info from redis error");
        match check_user_pass_by_name(user_info.name.clone()).await {
            None => {
                return ResponseBody::error("用户不存在");
            }
            Some(pass_data) => {
                if !pass_data.password.eq(&req_data.password) {
                    return ResponseBody::error("密码错误");
                }
            }
        }

        let jwt_token = gen_jwt_token(user_info);
        return ResponseBody::default(Some(jwt_token));
    }

    let db_user = check_user_pass_by_name(req_data.name.clone()).await;

    match db_user {
        None => {
            return ResponseBody::error("用户不存在");
        }
        Some(db_user) => {
            let is_eq = db_user.password.eq(&req_data.password);
            if !is_eq {
                return ResponseBody::error("密码错误");
            }
            let auth: u64 = get_user_access_val(db_user.id).await;
            let redis_data = RedisLoginData {
                auth,
                last_login_time: get_current_timestamp(),
                name: req_data.name.clone(),
                id: db_user.id.clone(),
            };

            let _: () = rds
                .set_ex(key.clone(), &redis_data, LOGIN_EX_TIME)
                .expect("set user login error");
            let jwt_token = gen_jwt_token(redis_data);
            return ResponseBody::default(Some(jwt_token));
        }
    }
}

async fn get_user_access_val(user_id: i32) -> u64 {
    let mut vals: u64 = 0;
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
    vals
}

async fn check_user_pass_by_name(name: String) -> Option<PasswordData> {
    let ex = RB.acquire().await.expect("msg");

    let db_user: Option<PasswordData> = ex
        .query_decode(
            "select password, id from user where user.name=?",
            vec![to_value!(name)],
        )
        .await
        .expect("获取用户失败");
    db_user
}
