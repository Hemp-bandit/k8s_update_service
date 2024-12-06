use crate::REDIS_ADDR;

use super::{
    common::RedisKeys,
    redis_actor::{HdelData, HsetData, SaddData, SremData},
};
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
    let rds = REDIS_ADDR.get().expect("msg");

    let msg = SaddData {
        key: data.set_key,
        id: data.id,
    };

    rds.send(msg).await.expect("msg").expect("msg");

    let msg = HsetData {
        key: data.hmap_key,
        id: data.id,
        opt_data: serde_json::to_string(&data.opt_data).expect("msg"),
    };

    rds.send(msg).await.expect("msg").expect("msg");
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
    let rds = REDIS_ADDR.get().expect("msg");

    let msg = SremData {
        key: data.set_key,
        value: data.id.clone(),
    };
    rds.send(msg).await.expect("msg").expect("msg");

    let msg = HdelData {
        id: data.id,
        key: data.hmap_key,
    };
    rds.send(msg).await.expect("msg").expect("msg");
}
