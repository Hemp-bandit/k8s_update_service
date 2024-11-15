use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use fast_log::{
    consts::LogSize,
    plugin::{
        file_split::{KeepType, RawFile, Rolling, RollingType},
        packer,
    },
    Config,
};
use log::info;

mod response;
mod router;

#[actix_web::main]
async fn main() {
    let log_conf = Config::new()
        .console()
        .chan_len(Some(100000))
        .split::<RawFile, _, _, _>(
            "logs/",
            KeepType::All,
            packer::LogPacker {},
            Rolling::new(RollingType::BySize(LogSize::MB(1))),
        );
    fast_log::init(log_conf).unwrap();

    let _ = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "DELETE"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
            .service(web::scope("/").configure(deployment_config))
    })
    .workers(2)
    .bind(gen_server_url())
    .expect("服务启动失败")
    .run()
    .await;
}

/**
 * 流程接口router
 */
fn deployment_config(cfg: &mut web::ServiceConfig) {
    cfg.service(router::update_deployment);
}

fn gen_server_url() -> String {
    let host = "0.0.0.0";
    let url = format!("{}:{}", host, 3001);
    info!("server is on, addr http://{}", url);
    url
}
