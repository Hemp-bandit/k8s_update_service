use rbatis::{crud};
use serde::{Deserialize, Serialize};


// #[derive(Clone, Debug, Serialize)]
pub enum UserType {
    BIZ = 0,
    CLIENT = 1,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserEntity {
    pub id: Option<u16>,
    pub name: String,
    pub create_time: String,
    pub update_time: String,
    pub user_type: u8,
}

crud!(UserEntity {}, "user");
