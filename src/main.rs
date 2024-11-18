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
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};
use utoipa_actix_web::{scope, AppExt};

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

    #[derive(OpenApi)]
    #[openapi(paths(router::get_user))]
    struct ApiDoc1;

    let _ = HttpServer::new(move || {

        let (app, api) =   App::new()
        .into_utoipa_app()
        .service(
            scope::scope("/api")
            .service(scope::scope("/v1").service(router::update_deployment))
        );

        app.service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api))

        // App::new()
        //     .wrap(
        //         Cors::default()
        //             .allow_any_origin()
        //             .allowed_methods(vec!["GET", "POST", "DELETE"])
        //             .allowed_headers(vec![
        //                 http::header::AUTHORIZATION,
        //                 http::header::ACCEPT,
        //                 http::header::CONTENT_TYPE,
        //             ])
        //             .max_age(3600),
        //     )
        //     .service(web::scope("/api").configure(deployment_config))
        //     .service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![(
        //         Url::new("api1", "/api-docs/openapi1.json"),
        //         ApiDoc1::openapi(),
        //     )]))
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
