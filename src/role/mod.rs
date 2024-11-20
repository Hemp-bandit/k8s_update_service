use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

mod role_route;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(role_route::create_role);
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateRoleData {
    pub name: String,
    pub create_by: i16,
}
