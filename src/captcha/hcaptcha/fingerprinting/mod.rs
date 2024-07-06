use rocket::async_trait;
use rocket::serde::{Deserialize, Serialize};
use crate::commons::error::FCaptchaResult;

pub mod hsw;
pub mod hsl;
pub mod jwt;

#[async_trait]
pub trait PoWChallenge {
    async fn get_proof(&self, jwt: &str) -> FCaptchaResult<String>;
}
