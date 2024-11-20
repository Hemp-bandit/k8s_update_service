use rbatis::{crud, impl_select, impl_select_page};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleEntity {
    pub id: Option<u16>,
    pub create_time: String,
    pub update_time: String,
    pub name: String,
    pub create_by: i16, // 创建的用户id
    pub status: i16,
}

crud!(RoleEntity {}, "role");
