use utoipa_actix_web::service_config::ServiceConfig;

mod user_controller;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(user_controller::create_user);
        config.service(user_controller::get_user_list);
    }
}
