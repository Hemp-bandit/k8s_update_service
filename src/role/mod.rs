use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

mod role_route;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(role_route::create_role);
        config.service(role_route::get_role_list);
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateRoleData {
    pub name: String,
    pub create_by: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct RoleListQuery {
    pub name: Option<String>,
    pub page_no: i32,
    pub take: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct RoleUpdateData {
    pub id: i32,
    pub name: Option<String>,
    pub status: Option<i16>,
}
