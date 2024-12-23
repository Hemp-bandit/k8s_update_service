use actix_cors::Cors;
use actix_web::middleware::{from_fn, Compress, Logger};
use actix_web::{http, App, HttpServer};
use chrono::Utc;
use cron::sync_auth::{sync_role_access, sync_user_role};
use env::dotenv;
use env_logger;
use once_cell::sync::OnceCell;
use rbatis::RBatis;
use rbdc_mysql::MysqlDriver;
use rs_service_util::jwt::JWT;
use rs_service_util::redis::RedisTool;
use tokio_schedule::{every, Job};
use user::admin::check_adm;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
use utoipa_scalar::{Scalar, Servable as ScalarServiceable};

mod access;
mod cron;
mod entity;
mod response;
mod role;
mod user;
mod util;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "user", description = "user 接口"),
        (name = "role", description = "role 接口"),
        (name = "access", description = "权限接口"),
        (name = "auth", description = "验权接口")
    ),
    modifiers(&JWT),
    security(
        ("JWT" = ["edit:items", "read:items"])
    )
)]
struct ApiDoc;

lazy_static::lazy_static! {
    static ref REDIS_KEY:String = "user_service".to_string();
    static ref RB:RBatis=RBatis::new();
    static ref REDIS: OnceCell<RedisTool> = OnceCell::new();
}

#[actix_web::main]
async fn main() {
    dotenv().expect("Failed to load .env file");
    env_logger::init();
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let _ = REDIS.set(RedisTool::new(redis_url).await);

    init_db().await;

    init_corn().await;

    let _ = check_adm().await;

    let _ = HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .service(utoipa_actix_web::scope("/api/user").configure(user::configure()))
            .service(utoipa_actix_web::scope("/api/role").configure(role::configure()))
            .service(utoipa_actix_web::scope("/api/access").configure(access::configure()))
            .service(utoipa_actix_web::scope("/api/auth").configure(user::auth_configure()))
            .service(utoipa_actix_web::scope("/api/obs").configure(user::obs_configure()))
            .openapi_service(|api| Scalar::with_url("/doc", api))
            .into_app()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "DELETE", "PUT", "OPTION"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::CONTENT_TYPE,
                    ]),
            )
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(Logger::new("t %P %s %{service_call}i"))
            .wrap(from_fn(|req, next| {
                let rds = crate::REDIS.get().expect("msg");
                let conn = rds.conn.clone();
                rs_service_util::middleware::jwt_mw(req, next, conn)
            }))
    })
    .keep_alive(None)
    .shutdown_timeout(5)
    .bind(gen_server_url())
    .expect("服务启动失败")
    .run()
    .await;
}

fn gen_server_url() -> String {
    let host = "0.0.0.0";
    let url = format!("{}:{}", host, 3000);
    log::info!("server is on, addr http://127.0.0.1:3000\n doc:  http://127.0.0.1:3000/doc");
    url
}

async fn init_db() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    log::info!("database_url {database_url}");
    if let Err(e) = RB.link(MysqlDriver {}, &database_url).await {
        panic!("db err: {}", e.to_string());
    }
}

async fn init_corn() {
    let corn = std::env::var("CORN").expect("CORN must be set");
    if !corn.eq("true") {
        log::info!("corn is close");
        return;
    }
    actix_rt::spawn(async move {
        let user_role_corn = every(10).seconds().in_timezone(&Utc).perform(|| async {
            sync_user_role().await;
            sync_role_access().await;
        });
        user_role_corn.await;
    });
}
