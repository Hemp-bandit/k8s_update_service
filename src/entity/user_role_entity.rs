use rbatis::{crud, impl_delete, impl_select, impl_select_page};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserRoleEntity {
    pub id: Option<i32>,
    pub role_id: i32,
    pub user_id: i32,
}

crud!(UserRoleEntity {}, "user_role");
impl_select_page!(UserRoleEntity{select_page_by_user_id(id:i32) => "`where user_id = #{id}`" }, "user_role" );
impl_delete!(UserRoleEntity { delete_by_role_and_user(rid:i32, uid:i32)=> "`where role_id = #{rid} and user_id = #{uid}`"  },"user_role");
impl_select!(UserRoleEntity {find_by_role_and_user(rid:i32,uid:i32)=>"`where role_id = #{rid} and user_id = #{uid}`"}, "user_role");
