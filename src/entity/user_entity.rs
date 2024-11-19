use rbatis::{crud, impl_select, impl_select_page};
use serde::{Deserialize, Serialize};



pub enum UserType {
  BIZ,
  CLIENT,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserEntity {
    pub id: Option<i16>,
    pub name: String,
    pub create_time: String,
    pub update_time: String,
    pub user_type: UserType,
}

crud!(UserEntity {}, "user");