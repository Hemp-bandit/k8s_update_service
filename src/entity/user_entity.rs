use rbatis::{crud, impl_select_page, rbdc::DateTime};
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
    pub create_time: DateTime,
    pub update_time: DateTime,
    pub name: String,
    pub phone: String,
    pub picture: String,
    pub introduce: String,
    pub user_type: UserType,
    pub status: Status,
}

crud!(UserEntity {}, "user");
impl_select_page!(UserEntity{select_page() => "`order by create_time desc`" } );
