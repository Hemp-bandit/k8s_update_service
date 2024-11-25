use actix_web::middleware::{Compress, Logger};
use actix_web::{App, HttpServer};
use rbatis::RBatis;
use rbdc_mysql::MysqlDriver;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
use utoipa_scalar::{Scalar, Servable as ScalarServiceable};
use env_logger;
use env::dotenv;


mod access;
mod common;
mod entity;
mod response;
mod role;
mod user;


lazy_static::lazy_static! {
    static ref RB:RBatis=RBatis::new();
}


#[actix_web::main]
async fn main() {
    dotenv().expect("Failed to load .env file");
    env_logger::init();
    #[derive(OpenApi)]
    #[openapi(
        tags( 
            (name = "user", description = "user 接口"),
            (name = "role", description = "role 接口"),
            (name = "access", description = "权限接口"),
            (name = "auth", description = "验权接口")
        )
    )]
    struct ApiDoc;


    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    if let Err(e) = RB.link(MysqlDriver {}, &database_url).await {
          panic!("db err: {}", e.to_string());
    }
    // RB.get_pool().unwrap().set_max_open_conns(10).await;

    let _ = HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .service(
                utoipa_actix_web::scope("/api/user") .configure(user::configure()),
            )
            .service(
                utoipa_actix_web::scope("/api/role").configure(role::configure()),
            )
            .service(
                utoipa_actix_web::scope("/api/access").configure(access::configure()),
            )
            .service(
                utoipa_actix_web::scope("/api/auth").configure(user::auth_configure()),
            )
            .openapi_service(|api| Scalar::with_url("/doc", api))
            .into_app()
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))       
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
    println!("server is on, addr http://127.0.0.1:3000\n doc:  http://127.0.0.1:3000/doc");
    url
}
