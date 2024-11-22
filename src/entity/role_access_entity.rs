use rbatis::{crud, impl_delete, impl_select, impl_select_page};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleAccessEntity {
    pub id: Option<u16>,
    pub role_id: i32,
    pub access_id: i32,
}

crud!(RoleAccessEntity {}, "role_access");
impl_select_page!(RoleAccessEntity{select_page_by_role_id(id:i32) => "`where role_id = #{id}`" }, "role_access" );
impl_delete!(RoleAccessEntity { delete_by_role_and_access(rid:i32, aid:i32)=> "`where role_id = #{rid} and access_id = #{aid}`"  },"role_access");
impl_select!(RoleAccessEntity {find_by_role_and_access(rid:i32,aid:i32)=>"`where role_id = #{rid} and access_id = #{aid}`"}, "role_access");
