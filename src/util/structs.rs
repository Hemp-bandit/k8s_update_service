use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateByData {
    pub id: Option<i32>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename = "Enum")]
pub enum UserType {
    BIZ = 0,
    CLIENT = 1,
    ADMIN = 2,
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
#[schema(example = json!({"page_no": 1, "take": 10}))]
pub struct CommListReq {
    pub page_no: u16,
    pub take: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NameListQuery {
    pub name: Option<String>,
    pub page_no: i32,
    pub take: i32,
}
