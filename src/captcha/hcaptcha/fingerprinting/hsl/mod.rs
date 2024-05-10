use hashcash::Stamp;
use rocket::async_trait;
use super::jwt::parse_jwt;
use crate::commons::error::DortCapResult;
use super::PoWChallenge;

pub struct HSL {}

#[async_trait]
impl PoWChallenge for HSL {
    async fn get_proof(&self, jwt: &str) -> DortCapResult<String> {
        let decoded = parse_jwt(jwt).await?;
        // Use mint_wasm as it has the so-called 'custom' date format hCaptcha uses.
        Ok(Stamp::mint_wasm(Some(&*decoded.data), Some(decoded.stamp_difficulty), None, None, None, false)?.to_string())
    }
}

impl Default for HSL {
    fn default() -> Self {
        Self {}
    }
}
