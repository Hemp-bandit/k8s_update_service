use actix_web::web::Data;
use actix_web::{App, HttpServer};
use fast_log::plugin::file_split::DateType;
use fast_log::plugin::packer::LogPacker;
use fast_log::{
    plugin::file_split::{KeepType, Rolling, RollingType},
    Config,
};
use log::info;
use rbatis::RBatis;
use rbdc_mysql::MysqlDriver;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
use utoipa_scalar::{Scalar, Servable as ScalarServiceable};

use env::dotenv;
mod access;
mod common;
mod entity;
mod response;
mod role;
mod router;
mod user;

struct DataStore {
    pub db: RBatis,
}

#[actix_web::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    init_log();

    #[derive(OpenApi)]
    #[openapi(
        tags( (name = "kaibai_user_service", description = " kaibai 用户服务"))
    )]
    struct ApiDoc;

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = init_db(&database_url).await;
    let store = Data::new(DataStore { db });

    let _ = HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .service(
                utoipa_actix_web::scope("/api/user")
                    .app_data(store.clone())
                    .configure(user::configure()),
            )
            .service(utoipa_actix_web::scope("/api/").configure(router::configure()))
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

fn init_log() {
    fast_log::init(Config::new().chan_len(Some(100000)).console().file_split(
        "logs/",
        Rolling::new(RollingType::ByDate(DateType::Day)),
        KeepType::KeepNum(2),
        LogPacker {},
    ))
    .unwrap();
    log::logger().flush();
}

async fn init_db(db_url: &str) -> RBatis {
    let rb = RBatis::new();
    if let Err(e) = rb.link(MysqlDriver {}, db_url).await {
        panic!("db err: {}", e.to_string());
    }
    rb
}
