use actix_web::web::Data;
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
use rbatis::RBatis;
use rbdc_mysql::MysqlDriver;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
use utoipa_scalar::{Scalar, Servable as ScalarServiceable};

use env::dotenv;
mod access;
mod commom;
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
}

async fn init_db(db_url: &str) -> RBatis {
    let rb = RBatis::new();
    if let Err(e) = rb.link(MysqlDriver {}, db_url).await {
        panic!("db err: {}", e.to_string());
    }
    rb
}
