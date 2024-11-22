use rbatis::crud;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleAccessEntity {
    pub id: Option<u16>,
    pub role_id: i32,
    pub access_id: i32,
}

crud!(RoleAccessEntity {}, "role_access");
