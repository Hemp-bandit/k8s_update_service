use actix_web::{App, HttpServer};
use fast_log::{
    consts::LogSize,
    plugin::{
        file_split::{KeepType, RawFile, Rolling, RollingType},
        packer,
    },
    Config,
};
use log::info;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
use utoipa_scalar::{Scalar, Servable as ScalarServable};
mod response;
mod router;
mod user;
mod role;
mod access;

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

    #[derive(OpenApi)]
    #[openapi(
        tags(
            (name = "kaibai_user_service", description = " kaibai 用户服务")
        )
    )]
    struct ApiDoc;

    let _ = HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .service(utoipa_actix_web::scope("/api").configure(router::configure()))
            .openapi_service(|api| Scalar::with_url("/doc", api))
            .into_app()
    })
    .workers(2)
    .bind(gen_server_url())
    .expect("服务启动失败")
    .run()
    .await;
}

fn gen_server_url() -> String {
    let host = "0.0.0.0";
    let url = format!("{}:{}", host, 3000);
    info!("server is on, addr http://127.0.0.1:3000\n doc:  http://127.0.0.1:3000/doc");
    url
}
