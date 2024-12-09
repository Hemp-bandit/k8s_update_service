use crate::response::MyError;
use crate::{RB, REDIS_ADDR};
use derive_more::derive::Display;
use lazy_regex::regex;
use rbatis::executor::RBatisTxExecutorGuard;
use rbatis::Error;

use super::redis_actor::HgetById;
#[derive(Debug, Display, Clone)]
pub enum RedisKeys {
    #[display("user_ids")]
    UserIds,

    #[display("user_info")]
    UserInfo,

    #[display("role_ids")]
    RoleIds,

    #[display("role_info")]
    RoleInfo,

    #[display("user_roles")]
    UserRoles,

    #[display("role_access")]
    RoleAccess,

    #[display("access_map")]
    AccessMap,

    #[display("access_map_ids")]
    AccessMapIds,
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

pub async fn get_transaction_tx() -> Result<RBatisTxExecutorGuard, MyError> {
    let tx = RB.acquire_begin().await.unwrap();
    let tx: RBatisTxExecutorGuard = tx.defer_async(|tx| async move {
        if tx.done() {
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
    log::info!("transaction [{}] start", tx.tx_id());
    Ok(tx)
}

pub fn gen_access_value(bit: u64) -> u64 {
    let mod_val = bit % 31;
    let last_number = 1 << (mod_val.min(31) - 1);
    last_number
}

// pub fn marge_access(arr: Vec<u64>) -> u64 {
//     let mut res = 0;
//     arr.into_iter().for_each(|val| {
//         res += val;
//     });
//     res
// }

// pub fn has_access(auth: u64, access: Vec<u64>) -> bool {
//     let mut res = false;
//     access.into_iter().for_each(|val| {
//         res = val & auth > 0;
//     });
//     res
// }

pub async fn rds_str_to_list<T, U: Fn(String) -> T>(
    ids: Vec<i32>,
    key: RedisKeys,
    cb: U,
) -> Vec<T> {
    let rds = REDIS_ADDR.get().expect("get addr err");
    let mut res: Vec<T> = vec![];
    for id in ids {
        let data = rds
            .send(HgetById {
                key: key.clone().to_string(),
                id,
            })
            .await
            .expect("msg")
            .expect("msg");
        if let Some(str) = data {
            let item: T = cb(str);
            res.push(item);
        }
    }
    res
}

#[cfg(test)]
mod test {

    use rs_service_util::auth::gen_access_value;

    use crate::util::common::check_phone;

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
}
