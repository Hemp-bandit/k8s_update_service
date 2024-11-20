use chrono::{DateTime, Local, Utc};
use lazy_regex::regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeployInfo {
    pub deployment_name: String,
    pub container_name: String,
    pub new_image: String,
    pub new_tag: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CommListReq {
    pub offset: u16,
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
