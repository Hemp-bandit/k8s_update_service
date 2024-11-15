use crate::response::ResponseBody;
use actix_web::{post, web, Responder};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};

#[derive(Debug, Deserialize, Serialize)]
struct DeployInfo {
    deployment_name: String,
    container_name: String,
    new_image: String,
    new_tag: String,
}

#[post("/update_deployment")]
pub async fn update_deployment(config: web::Json<DeployInfo>) -> impl Responder {
    let res = ResponseBody {
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
    let mut child = ShellUtil::spawn_new_command(cmd.to_string());
    match child.wait().await {
        Ok(status) => {
            if status.success() {
                return res;
            }
        }
        Err(e) => println!("Failed to wait for child process: {}", e),
    }
    res
}

pub struct ShellUtil;
impl ShellUtil {
    /// 创建shell环境
    pub fn spawn_new_command(shell_str: String) -> Child {
        let output = Command::new("sh")
            .arg("-c")
            .arg(shell_str)
            .spawn();

        match output {
            Ok(child) => child,
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}
