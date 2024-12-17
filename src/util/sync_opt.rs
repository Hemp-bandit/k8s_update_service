use super::common::RedisKeys;

use redis::AsyncCommands;
use rs_service_util::redis_conn;
use serde::Serialize;

pub struct SyncOptData<T> {
    pub set_key: String,
    pub hmap_key: String,
    pub id: i32,
    pub opt_data: T,
}
impl<T: Serialize> SyncOptData<T> {
    pub fn default(set_key: RedisKeys, hmap_key: RedisKeys, id: i32, opt_data: T) -> Self {
        Self {
            set_key: set_key.to_string(),
            hmap_key: hmap_key.to_string(),
            opt_data,
            id,
        }
    }
}

pub async fn sync<T: Serialize>(data: SyncOptData<T>) {
    let mut conn = redis_conn!().await;
    let _: () = conn.sadd(data.set_key, data.id).await.expect("msg");
    let _: () = conn
        .hset(
            data.hmap_key,
            data.id,
            serde_json::to_string(&data.opt_data).expect("msg"),
        )
        .await
        .expect("msg");
}

pub struct DelOptData {
    pub set_key: String,
    pub hmap_key: String,
    pub id: Vec<i32>,
}

impl DelOptData {
    pub fn default(set_key: RedisKeys, hmap_key: RedisKeys, id: Vec<i32>) -> Self {
        Self {
            set_key: set_key.to_string(),
            hmap_key: hmap_key.to_string(),
            id,
        }
    }
}

pub async fn del(data: DelOptData) {
    let mut conn = redis_conn!().await;
    let _: () = conn.srem(data.set_key, &data.id).await.expect("msg");
    let _: () = conn.hdel(data.hmap_key, &data.id).await.expect("msg");
}
