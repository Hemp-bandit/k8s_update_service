use crate::{user::OptionData, REDIS};
use redis::Commands;
#[derive(Debug)]
pub struct SyncOptData {
    pub set_key: String,
    pub hmap_key: String,
    pub opt_data: OptionData,
}
impl SyncOptData {
    pub fn default(set_key: &str, hmap_key: &str, opt_data: OptionData) -> Self {
        Self {
            set_key: set_key.to_string(),
            hmap_key: hmap_key.to_string(),
            opt_data,
        }
    }
}

pub async fn sync(data: SyncOptData) {
    let mut rds = REDIS.lock().unwrap();
    log::info!("data {data:?}");
    let _: () = rds
        .sadd(data.set_key, data.opt_data.id)
        .expect("set user_id to rds err");
    let _: () = rds
        .hset(
            data.hmap_key,
            data.opt_data.id,
            serde_json::to_string(&data.opt_data).expect("msg"),
        )
        .expect("hset user to rds err");
}
