use rbatis::{crud, impl_select, impl_select_page};
use serde::{Deserialize, Serialize};

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
impl_select_page!(UserEntity{select_page() => "`where status=1 order by create_time desc`" }, "user" );
impl_select!(UserEntity{select_by_id(id:i32) -> Option => "`where id = #{id} and status=1`"}, "user");
impl_select!(UserEntity{select_by_name_phone(name:&str, phone:&str) -> Option => "`where name = #{name} or phone= #{phone}  and status=1`"}, "user");
