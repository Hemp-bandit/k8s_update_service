use crate::REDIS;

use super::common::RedisKeys;
use redis::Commands;
use serde::Serialize;

pub struct SyncOptData<T> {
    pub set_key: String,
    pub hmap_key: String,
    pub opt_data: T,
    pub id: i32,
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
    let mut rds = REDIS.lock().expect("get rds err");
    let _: () = rds
        .sadd(data.set_key, data.id)
        .expect("set user_id to rds err");
    let _: () = rds
        .hset(
            data.hmap_key,
            data.id,
            serde_json::to_string(&data.opt_data).expect("msg"),
        )
        .expect("hset user to rds err");
}
