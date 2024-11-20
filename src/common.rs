use chrono::{DateTime, Local, Utc};
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
