use rbatis::{crud, impl_select};
use serde::{Deserialize, Serialize};

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

crud!(UserEntity {}, "user");
impl_select!(UserEntity{select_by_id(id:i32) -> Option => "`where id = #{id} and status=1`"}, "user");
impl_select!(UserEntity{select_by_name_phone(name:&str, phone:&str) -> Option => "`where name = #{name} or phone= #{phone}  and status=1`"}, "user");
impl_select!(UserEntity{select_by_name(name:&str) -> Option => "`where name = #{name} and status=1`"}, "user");
