use rocket::async_trait;
use crate::captcha::hcaptcha::fingerprinting::PoWChallenge;

mod events;
mod constants;


pub struct HSW {
    fingerprint_key: String,
    version: String
}

impl HSW {

}

#[async_trait]
impl PoWChallenge for HSW {
    async fn get_proof(&self, jwt: &str) -> String {
        String::new()
    }
}