use std::sync::Mutex;
use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use actix_web::{http, App, HttpServer};
use common::JWT;
use middleware::JwtAuth;
use rbatis::RBatis;
use rbdc_mysql::MysqlDriver;
use redis::Connection;
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
mod middleware;


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
    static ref REDIS:Mutex<Connection> = {
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
        let client =  match redis::Client::open(redis_url) {
            Err(err)=>{
                let detail = err.detail().expect("get redis err detail ");
                panic!("redis client err:  {detail}");
            },
            Ok(client)=>{
                client
            }
        };
    
        let conn: Connection =  match client.get_connection() {
          Err(err)=>{
            let detail = err.detail().expect("get redis err detail ");
            panic!("redis connect err:  {detail}");
          },
          Ok(conn)=>{
            conn
          }
        };
        Mutex::new(conn)
    };
}


#[actix_web::main]
async fn main() {
    dotenv().expect("Failed to load .env file");
    env_logger::init();

    init_db().await;

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
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(JwtAuth)
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

async fn init_db(){
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    if let Err(e) = RB.link(MysqlDriver {}, &database_url).await {
          panic!("db err: {}", e.to_string());
    }
}
