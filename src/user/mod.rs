use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

mod user_route;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(user_route::create_user);
        config.service(user_route::get_user_list);
        config.service(user_route::get_user_by_id);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreateData {
    pub name: String,
    pub password: String,
    pub phone: String,
    pub picture: Option<String>,
    pub introduce: Option<String>,
}
