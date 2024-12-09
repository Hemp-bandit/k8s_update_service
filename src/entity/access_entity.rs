use rbatis::{crud, impl_select, impl_select_page};
use rs_service_util::time::get_current_time_fmt;
use serde::{Deserialize, Serialize};

use crate::util::structs::Status;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessEntity {
    pub id: Option<i32>,
    pub create_time: String,
    pub update_time: String,
    pub name: String,
    pub create_by: i32, // 创建的用户id
    pub status: i8,
    pub value: u64,
}
impl AccessEntity {
    pub fn default_adm_access(adm_user_id: i32) -> Self {
        Self {
            id: None,
            create_by: adm_user_id,
            status: Status::ACTIVE as i8,
            create_time: get_current_time_fmt(),
            update_time: get_current_time_fmt(),
            name: "ADMIN".to_string(),
            value: 0,
        }
    }
}
crud!(AccessEntity {}, "access");
impl_select_page!(AccessEntity{select_page() => "`where status=1 order by create_time desc`" }, "access" );
impl_select_page!(AccessEntity{select_page_by_name(name:&str) => "`where status=1 and name = #{name} order by create_time desc`" }, "access" );
impl_select!( AccessEntity{ select_by_id(id:i32) -> Option => "`where id = #{id} and status=1`" }, "access" );
impl_select!( AccessEntity{ select_by_name(name:&str) -> Option => "`where name = #{name} and status=1`" }, "access" );
