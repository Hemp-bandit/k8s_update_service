use rbatis::{crud, impl_select};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleEntity {
    pub id: Option<i32>,
    pub create_time: String,
    pub update_time: String,
    pub name: String,
    pub create_by: i32, // 创建的用户id
    pub status: i8,
}

crud!(RoleEntity {}, "role");
impl_select!( RoleEntity{ select_by_id(id:i32) -> Option => "`where id = #{id} and status=1`" }, "role" );
impl_select!( RoleEntity{ select_by_name(name:&str) -> Option => "`where name=#{name} and status=1`" }, "role" );
