use crate::{user::RedisLoginData, RB};
use chrono::{DateTime, Local, Utc};
use lazy_regex::regex;
use rbatis::executor::RBatisTxExecutorGuard;
use rbatis::Error;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use hmac::{Hmac, Mac};
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename = "Enum")]
pub enum UserType {
    BIZ = 0,
    CLIENT = 1,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename = "Enum")]
pub enum Status {
    ACTIVE = 1,
    DEACTIVE = 0,
}

impl Status {
    pub fn from(val: i8) -> Status {
        match val {
            0 => Status::DEACTIVE,
            1 => Status::ACTIVE,
            _ => Status::DEACTIVE,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeployInfo {
    pub deployment_name: String,
    pub container_name: String,
    pub new_image: String,
    pub new_tag: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CommListReq {
    pub page_no: u16,
    pub take: u16,
}

/**
 * 获取当前时间的时间戳
 * 格式: YYYY-MM-DD HH:mm:ss
 */
pub fn get_current_time_fmt() -> String {
    let dt = Utc::now();
    let local_dt: DateTime<Local> = dt.with_timezone(&Local);
    return local_dt.format("%Y-%m-%d %H:%M:%S").to_string();
}

/**
 * 获取当前时间戳
 * 秒
 */
pub fn get_current_timestamp() -> i64 {
    let dt = Utc::now();
    let local_dt: DateTime<Local> = dt.with_timezone(&Local);
    local_dt.timestamp()
}

/// 检测手机号是否合法
pub fn check_phone(phone: &str) -> bool {
    let max_len = 11;
    if phone.len() != max_len {
        log::error!("手机号长度不对");
        return false;
    }
    let r = regex!(r"^1[3-9]\d{9}$");

    r.is_match(phone)
}

pub async fn get_transaction_tx() -> Result<RBatisTxExecutorGuard, Error> {
    let tx = RB.acquire_begin().await.unwrap();
    let tx: RBatisTxExecutorGuard = tx.defer_async(|mut tx| async move {
        if tx.done {
            log::info!("transaction [{}] complete.", tx.tx_id);
        } else {
            let r = tx.rollback().await;
            if let Err(e) = r {
                log::error!("transaction [{}] rollback fail={}", tx.tx_id, e);
            } else {
                log::info!("transaction [{}] rollback", tx.tx_id);
            }
        }
    });
    log::info!("transaction [{}] start", tx.tx.as_ref().unwrap().tx_id);
    Ok(tx)
}

pub fn gen_access_value(bit: u64) -> u64 {
    let mod_val = bit % 31;
    let last_number = 1 << (mod_val.min(31) - 1);
    last_number
}

pub fn marge_access(arr: Vec<u64>) -> u64 {
    let mut res = 0;
    arr.into_iter().for_each(|val| {
        res += val;
    });
    res
}

pub fn has_access(auth: u64, access: Vec<u64>) -> bool {
    let mut res = false;
    access.into_iter().for_each(|val| {
        res = val & auth > 0;
    });
    res
}

pub fn gen_jwt_token(login_data: RedisLoginData) -> String {
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or("QWERTYUOas;ldfj;4u1023740^&&*()_)*&^".to_string());
    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();
    let token_str = login_data.sign_with_key(&key).unwrap();

    token_str
}

pub fn jwt_token_to_data(jwt_token: String) -> RedisLoginData {
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or("QWERTYUOas;ldfj;4u1023740^&&*()_)*&^".to_string());
    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();
    let claims: RedisLoginData = jwt_token.verify_with_key(&key).expect("msg");
    claims
}

#[cfg(test)]
mod test {
    use hmac::digest::typenum::assert_type_eq;

    use crate::{common::check_phone, user::RedisLoginData};

    use super::{gen_access_value, gen_jwt_token, jwt_token_to_data};

    #[test]
    fn test_check_phone_length_less() {
        let phone = "123123";
        let res = check_phone(phone);
        assert_eq!(res, false);
    }

    #[test]
    fn test_check_phone_length_large() {
        let phone = "2222222222222222222222222222222";
        let res = check_phone(phone);
        assert_eq!(res, false);
    }

    #[test]
    fn test_check_phone_reg_false() {
        let phone = "12717827650";
        let res = check_phone(phone);
        assert_eq!(res, false);
    }

    #[test]
    fn test_check_phone_reg_true() {
        let phone = "15717827650";
        let res = check_phone(phone);
        assert_eq!(res, true);
    }
    #[test]
    fn test_access_value() {
        // let role_p = [64, 1024];
        // let role: u32 = 64 + 1024;
        // role_p.map(|val| println!("res {}", val & role));

        let rs = gen_access_value(9999999);
        println!("res=== {rs}");
    }
    #[test]
    fn get_gen_jwt() {
        let login_data = RedisLoginData {
            auth: 123123123123,
            last_login_time: 12312312,
            name: "asdf".to_string(),
            id: 123,
        };
        let token_res = gen_jwt_token(login_data);
        println!("token_res {token_res}");
        let token = "eyJhbGciOiJIUzI1NiJ9.eyJhdXRoIjoxMjMxMjMxMjMxMjMsImxhc3RfbG9naW5fdGltZSI6MTIzMTIzMTIsIm5hbWUiOiJhc2RmIiwiaWQiOjEyM30.wqZusohUbF1MsrzttbL0Zf6jgvQXlSOwO7wwsvr06aE".to_string();
        assert_eq!(token_res, token)
    }

    #[test]
    fn test_jwt_token_to_data() {
        let token = "eyJhbGciOiJIUzI1NiJ9.eyJhdXRoIjoxMjMxMjMxMjMxMjMsImxhc3RfbG9naW5fdGltZSI6MTIzMTIzMTIsIm5hbWUiOiJhc2RmIiwiaWQiOjEyM30.wqZusohUbF1MsrzttbL0Zf6jgvQXlSOwO7wwsvr06aE".to_string();
        let login_data = RedisLoginData {
            auth: 123123123123,
            last_login_time: 12312312,
            name: "asdf".to_string(),
            id: 123,
        };
        let user_info = jwt_token_to_data(token);
        assert_eq!(login_data, user_info);
    }
}
