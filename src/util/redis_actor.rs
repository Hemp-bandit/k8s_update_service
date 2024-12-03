use actix::prelude::*;
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use serde::Serialize;

use super::common::RedisKeys;

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
#[rtype(result = "Result<Option<String>, redis::RedisError>")]
struct InfoCommand;

impl Handler<InfoCommand> for RedisActor {
    type Result = ResponseFuture<Result<Option<String>, redis::RedisError>>;

    fn handle(&mut self, _msg: InfoCommand, _: &mut Self::Context) -> Self::Result {
        let mut con = self.conn.clone();
        let cmd = redis::cmd("INFO");
        let fut = async move { cmd.query_async(&mut con).await };
        Box::pin(fut)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<Option<()>, redis::RedisError>")]
pub struct HmapData<T> {
    pub hmap_key: RedisKeys,
    pub opt_data: T,
    pub id: i32,
}

impl<T: Serialize> Handler<HmapData<T>> for RedisActor {
    type Result = ResponseFuture<Result<Option<()>, redis::RedisError>>;

    fn handle(&mut self, msg: HmapData<T>, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();

        let mut hmap_cmd = redis::cmd("HSET");
        hmap_cmd.arg(&[
            &msg.hmap_key.to_string(),
            &msg.id.to_string(),
            &serde_json::to_string(&msg.opt_data).expect("msg"),
        ]);

        let fut = async move {
            hmap_cmd.query_async(&mut rds).await
        };
        Box::pin(fut)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<Option<()>, redis::RedisError>")]
pub struct SetData {
    pub set_key: RedisKeys,
    pub id: i32,
}

impl Handler<SetData> for RedisActor {
    type Result = ResponseFuture<Result<Option<()>, redis::RedisError>>;

    fn handle(&mut self, msg: SetData, _ctx: &mut Self::Context) -> Self::Result {
        let mut rds = self.conn.clone();
        let mut set_cmd = redis::cmd("SADD");
        set_cmd.arg(&[&msg.set_key.to_string(), &msg.id.to_string()]);
        let fut = async move { set_cmd.query_async(&mut rds).await };
        Box::pin(fut)
    }
}
