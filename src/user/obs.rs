use std::fmt::format;

use crate::{
    response::{MyError, ResponseBody},
    REDIS_KEY,
};
use actix_web::{get, Responder};
use redis::AsyncCommands;
use reqwest::header;
use rs_service_util::redis_conn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AkSkRes {
    credential: Credential,
}

#[derive(Serialize, Deserialize)]
struct Credential {
    access: String,
    expires_at: String,
    secret: String,
    securitytoken: String,
}

#[utoipa::path(
  tag = "obs",
  responses( (status = 200) )
)]
#[get("/get_keys")]
pub async fn get_keys() -> Result<impl Responder, MyError> {
    let mut conn = redis_conn!().await;
    let key = format!("{}_obs_ak_sk", REDIS_KEY.to_string());
    let cache_token: Option<String> = conn.get(&key).await.expect("get cache_token error");
    if let Some(token) = cache_token {
        let res: AkSkRes = serde_json::from_str(&token).expect("token perse error");
        return Ok(ResponseBody::default(Some(res)));
    }

    // gen http header
    let mut headers = header::HeaderMap::new();
    // set auth token to http-header
    let content_type = header::HeaderValue::from_str("application/json;charset=utf8").unwrap();
    headers.insert("Content-Type", content_type);
    // build http client
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("build client faille");

    let body ="{\"auth\":{\"identity\":{\"methods\":[\"password\"],\"password\":{\"user\":{\"domain\":{\"name\":\"wyswill\"},\"name\":\"wyswill\",\"password\":\"wyswill4290\"}}},\"scope\":{\"domain\":{\"name\":\"wyswill\"},\"project\":{\"name\":\"cn-east-3\"}}}}";

    let domain = std::env::var("OBS_DOMAIN").expect("OBS_DOMAIN must be set");
    log::debug!("domain {domain}");
    let res = client
        .post(format!("{domain}/v3/auth/tokens"))
        .body(body)
        .send()
        .await
        .expect("msg");
    let token = res.headers().get("X-Subject-Token");

    match token {
        None => {
            return Err(MyError::ObsTokenError);
        }
        Some(token) => {
            let token = token.to_str().map_err(|_| MyError::ObsTokenError)?;
            let mut headers = header::HeaderMap::new();
            headers.insert(
                "X-Auth-Token",
                header::HeaderValue::from_str(token).unwrap(),
            );
            let body = "{\"auth\":{\"identity\":{\"policy\":{\"Version\":\"1.1\",\"Statement\":[{\"Action\":[\"obs:object:PutObject\"],\"Resource\":[\"obs:*:*:object:kaibai-admin/store/*\",\"obs:*:*:object:kaibai-admin/adm/*\"],\"Effect\":\"Allow\"}]},\"methods\":[\"token\"]}}}";

            let res = client
                .post(format!("{domain}/v3.0/OS-CREDENTIAL/securitytokens"))
                .headers(headers)
                .body(body)
                .send()
                .await
                .map_err(|_| MyError::ObsAkSkError)?;
            let json = res.text().await.map_err(|_| MyError::ObsAkSkError)?;
            let res: AkSkRes = serde_json::from_str(&json).expect("token perse error");

            let _ = conn
                .set_ex(key, json, 15 * 60)
                .await
                .map_err(|_| MyError::CacheObsAkSkError)?;

            return Ok(ResponseBody::default(Some(res)));
        }
    }
}
