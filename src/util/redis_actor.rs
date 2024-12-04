use super::common::RedisCmd;
use crate::user::RedisLoginData;
use actix::prelude::*;
use redis::{aio::MultiplexedConnection, Client};

pub struct RedisActor {
    conn: MultiplexedConnection,
}

impl RedisActor {
    pub async fn new() -> Self {
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
        let client = Client::open(redis_url).unwrap(); // not recommended
        let conn = client.get_multiplexed_async_connection().await.unwrap();
        RedisActor { conn }
    }
}

impl Actor for RedisActor {
    type Context = Context<Self>;
}

#[derive(Message, Debug)]
#[rtype(result = "Result<(), redis::RedisError>")]
pub struct HsetData {
    pub key: String,
    pub id: i32,
    pub opt_data: String,
}

impl Handler<HsetData> for RedisActor {
    type Result = ResponseFuture<Result<(), redis::RedisError>>;

    fn handle(&mut self, msg: HsetData, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();

        let mut hmap_cmd = redis::cmd(&RedisCmd::Hset.to_string());
        hmap_cmd.arg(msg.key).arg(msg.id).arg(msg.opt_data);

        let fut = async move { hmap_cmd.query_async(&mut rds).await };
        Box::pin(fut)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<Option<()>, redis::RedisError>")]
pub struct SaddData {
    pub key: String,
    pub id: i32,
}

impl Handler<SaddData> for RedisActor {
    type Result = ResponseFuture<Result<Option<()>, redis::RedisError>>;

    fn handle(&mut self, msg: SaddData, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();
        let mut set_cmd = redis::cmd(&RedisCmd::Sadd.to_string());
        set_cmd.arg(msg.key).arg(msg.id);
        let fut = async move { set_cmd.query_async(&mut rds).await };
        Box::pin(fut)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<Option<String>, redis::RedisError>")]
pub struct HgetById {
    pub key: String,
    pub id: i32,
}

impl Handler<HgetById> for RedisActor {
    type Result = ResponseFuture<Result<Option<String>, redis::RedisError>>;

    fn handle(&mut self, msg: HgetById, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();
        let mut set_cmd = redis::cmd(&RedisCmd::Hget.to_string());
        set_cmd.arg(msg.key).arg(msg.id);
        let fut = async move { set_cmd.query_async(&mut rds).await };
        Box::pin(fut)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<  Vec<i32> , redis::RedisError>")]
pub struct SmembersData {
    pub key: String,
}
impl Handler<SmembersData> for RedisActor {
    type Result = ResponseFuture<Result<Vec<i32>, redis::RedisError>>;

    fn handle(&mut self, msg: SmembersData, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();
        let mut set_cmd = redis::cmd(&RedisCmd::Smembers.to_string());
        set_cmd.arg(msg.key);
        let fut = async move { set_cmd.query_async(&mut rds).await };
        Box::pin(fut)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<  bool , redis::RedisError>")]
pub struct ExistsData {
    pub key: String,
    pub cmd: RedisCmd,
    pub data: Option<String>,
}

impl Handler<ExistsData> for RedisActor {
    type Result = ResponseFuture<Result<bool, redis::RedisError>>;

    fn handle(&mut self, msg: ExistsData, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();
        let mut set_cmd = redis::cmd(&msg.cmd.to_string());
        if let Some(data) = msg.data {
            set_cmd.arg(msg.key).arg(data);
        }
        let fut = async move { set_cmd.query_async(&mut rds).await };
        Box::pin(fut)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result< () , redis::RedisError>")]
pub struct SremData {
    pub key: String,
    pub value: String,
}

impl Handler<SremData> for RedisActor {
    type Result = ResponseFuture<Result<(), redis::RedisError>>;

    fn handle(&mut self, msg: SremData, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();
        let mut set_cmd = redis::cmd(&RedisCmd::Srem.to_string());
        set_cmd.arg(msg.key).arg(msg.value);
        let fut = async move { set_cmd.query_async(&mut rds).await };
        Box::pin(fut)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result< RedisLoginData , redis::RedisError>")]
pub struct GetRedisLogin {
    pub key: String,
}

impl Handler<GetRedisLogin> for RedisActor {
    type Result = ResponseFuture<Result<RedisLoginData, redis::RedisError>>;

    fn handle(&mut self, msg: GetRedisLogin, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();
        let mut set_cmd = redis::cmd(&RedisCmd::Get.to_string());
        set_cmd.arg(msg.key);
        let fut = async move { set_cmd.query_async(&mut rds).await };
        Box::pin(fut)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result< () , redis::RedisError>")]
pub struct SetRedisLogin {
    pub key: String,
    pub data: RedisLoginData,
    pub ex_data: u64,
}

impl Handler<SetRedisLogin> for RedisActor {
    type Result = ResponseFuture<Result<(), redis::RedisError>>;

    fn handle(&mut self, msg: SetRedisLogin, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();
        let mut set_cmd = redis::cmd(&RedisCmd::SETEX.to_string());
        set_cmd.arg(msg.key).arg(msg.data).arg(msg.ex_data);
        let fut = async move { set_cmd.query_async(&mut rds).await };
        Box::pin(fut)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result< () , redis::RedisError>")]
pub struct DelData {
    pub key: String,
}

impl Handler<DelData> for RedisActor {
    type Result = ResponseFuture<Result<(), redis::RedisError>>;

    fn handle(&mut self, msg: DelData, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();
        let mut set_cmd = redis::cmd(&RedisCmd::Del.to_string());
        set_cmd.arg(msg.key);
        let fut = async move { set_cmd.query_async(&mut rds).await };
        Box::pin(fut)
    }
}
