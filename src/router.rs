use crate::commom::DeployInfo;
use crate::response::ResponseBody;
use actix_web::{post, web, Responder};
use log::info;
use utoipa_actix_web::service_config::ServiceConfig;

#[utoipa::path(
    tag = "kaibai_user_service",
    responses(
        (status = 200, description = "List current todo items", body=ResponseBody<String>)
    )
)]
#[post("/update_deployment")]
pub async fn update_deployment(config: web::Json<DeployInfo>) -> impl Responder {
    let mut res = ResponseBody {
        rsp_code: 0,
        rsp_msg: "".to_string(),
        data: "".to_string(),
    };

    println!("req data {:#?}", config);
    //  "kubectl set image deployment/<deployment_name> <container_name>=<new_image>:<new_tag>"
    let cmd = format!(
        "kubectl set image deployment/{} {}={}:{}",
        config.deployment_name, config.container_name, config.new_image, config.new_tag
    );

    info!("k8s cmd : {}", cmd);
    res.data = cmd;
    res
}

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.service(update_deployment);
    }
}
