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
impl_select_page!(RoleEntity{select_page() => "`where status=1 order by create_time desc`" }, "role" );
impl_select_page!(RoleEntity{select_page_by_name(name:&str) => "`where status=1 and name = #{name} order by create_time desc`" }, "role" );
impl_select!( RoleEntity{ select_by_id(id:i32) -> Option => "`where id = #{id} and status=1`" }, "role" );
impl_select!( RoleEntity{ select_by_name(name:&str) -> Option => "`where name = #{name} and status=1`" }, "role" );
