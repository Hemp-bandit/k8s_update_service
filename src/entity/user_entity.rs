use rbatis::{crud, impl_select, impl_select_page};
use serde::{Deserialize, Serialize};

#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq, Debug)]
#[serde(rename = "Enum")]
pub enum UserType {
    BIZ = 0,
    CLIENT = 1,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq, Debug)]
#[serde(rename = "Enum")]
pub enum Status {
    ACTIVE = 1,
    DEACTIVE = 0,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserEntity {
    pub id: Option<u16>,
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
impl_select_page!(UserEntity{select_page() => "`order by create_time desc`" }, "user" );
impl_select!(UserEntity{select_by_id(id:i32) -> Option => "`where id = #{id}`"}, "user");
impl_select!(UserEntity{select_by_name_phone(name:&str, phone:&str) -> Option => "`where name = #{name} or phone= #{phone}`"}, "user");
