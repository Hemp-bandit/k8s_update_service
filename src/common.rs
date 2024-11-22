use chrono::{DateTime, Local, Utc};
use lazy_regex::regex;
use rbatis::executor::RBatisTxExecutorGuard;
use rbatis::Error;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::RB;

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

#[cfg(test)]
mod test {
    use crate::common::check_phone;

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
}
