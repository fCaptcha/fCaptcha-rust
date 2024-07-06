use hashcash::Stamp;
use lazy_static::lazy_static;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use rocket::async_trait;
use rocket::time::macros::date;
use serde_json::json;
use warp::head;
use crate::captcha::hcaptcha::fingerprinting::hsw::events::generate_hsw_fp;
use crate::captcha::hcaptcha::fingerprinting::jwt::parse_jwt;
use self::structs::EncDecAPIResponse;
use super::PoWChallenge;
use crate::commons::error::FCaptchaResult;

pub mod events;
mod constants;
mod structs;
pub mod aes_encryption;
mod string_encoding;
mod checksum;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}


pub struct HSW {
    key: [u8; 32],
    version: String
}

impl HSW {
    async fn aes_gcm_encrypt(&self, data: &str) -> FCaptchaResult<String> {
        todo!()
    }

    async fn aes_gcm_decrypt(&self, data: &str) -> FCaptchaResult<String> {
        todo!()
    }
}

#[async_trait]
impl PoWChallenge for HSW {
    async fn get_proof(&self, jwt: &str) -> FCaptchaResult<String> {
        let jwt = parse_jwt(jwt)?;
        let stamp = Stamp::mint_wasm(Option::from(&*jwt.data), Option::from(jwt.stamp_difficulty), None, None, None, false)?;
        self.aes_gcm_encrypt(&*generate_hsw_fp(&*self.version).await?).await
    }
}