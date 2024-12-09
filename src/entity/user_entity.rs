use rbatis::{crud, impl_select};
use rs_service_util::time::get_current_time_fmt;
use serde::{Deserialize, Serialize};

use crate::util::structs::{Status, UserType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserEntity {
    pub id: Option<i32>,
    pub create_time: String,
    pub update_time: String,
    pub name: String,
    pub password: String,
    pub phone: String,
    pub picture: Option<String>,
    pub introduce: Option<String>,
    pub user_type: i16,
    pub status: i16,
}

impl UserEntity {
    pub fn default_adm_user() -> Self {
        Self {
            id: None,
            create_time: get_current_time_fmt(),
            update_time: get_current_time_fmt(),
            name: "ADMIN".to_string(),
            password: "ADMIN".to_string(),
            picture: None,
            phone: "15717827650".to_string(),
            introduce: None,
            user_type: UserType::ADMIN as i16,
            status: Status::ACTIVE as i16,
        }
    }
}

crud!(UserEntity {}, "user");
impl_select!(UserEntity{select_by_id(id:i32) -> Option => "`where id = #{id} and status=1`"}, "user");
impl_select!(UserEntity{select_by_name_phone(name:&str, phone:&str) -> Option => "`where name = #{name} or phone= #{phone}  and status=1`"}, "user");
impl_select!(UserEntity{select_by_name(name:&str) -> Option => "`where name = #{name} and status=1`"}, "user");
