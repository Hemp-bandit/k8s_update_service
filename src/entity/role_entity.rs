use rbatis::{crud, impl_select};
use rs_service_util::time::get_current_time_fmt;
use serde::{Deserialize, Serialize};

use crate::util::structs::Status;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleEntity {
    pub id: Option<i32>,
    pub create_time: String,
    pub update_time: String,
    pub name: String,
    pub create_by: i32, // 创建的用户id
    pub status: i8,
}

impl RoleEntity {
    pub fn default_adm(adm_user_id: i32) -> Self {
        Self {
            id: None,
            create_time: get_current_time_fmt(),
            update_time: get_current_time_fmt(),
            name: "ADMIN".to_string(),
            create_by: adm_user_id,
            status: Status::ACTIVE as i8,
        }
    }
}

crud!(RoleEntity {}, "role");
impl_select!( RoleEntity{ select_by_id(id:i32) -> Option => "`where id = #{id} and status=1`" }, "role" );
impl_select!( RoleEntity{ select_by_name(name:&str) -> Option => "`where name=#{name} and status=1`" }, "role" );
