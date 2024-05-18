use lazy_static::lazy_static;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use rocket::async_trait;
use serde_json::json;
use warp::head;
use self::structs::EncDecAPIResponse;
use super::PoWChallenge;
use crate::commons::error::DortCapResult;

mod events;
mod constants;
mod structs;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}


pub struct HSW {
    fingerprint_key: String,
    version: String
}

impl HSW {
    async fn aes_gcm_encrypt(data: &str) -> DortCapResult<EncDecAPIResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("key", HeaderValue::from_str("itg939021i0t93r1i09").unwrap());
        Ok(CLIENT.post("http://83.143.112.20:8080/encrypt").headers(headers).json(&json!({"data": data})).send().await?.json().await?)
    }

    async fn aes_gcm_decrypt(data: &str) -> DortCapResult<EncDecAPIResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("key", HeaderValue::from_str("itg939021i0t93r1i09").unwrap());
        Ok(CLIENT.post("http://83.143.112.20:8080/decrypt").headers(headers).json(&json!({"data": data})).send().await?.json().await?)
    }
}

#[async_trait]
impl PoWChallenge for HSW {
    async fn get_proof(&self, jwt: &str) -> DortCapResult<String> {
        Ok(String::new())
    }
}