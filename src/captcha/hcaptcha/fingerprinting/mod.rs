use rocket::async_trait;
use rocket::serde::{Deserialize, Serialize};

mod hsw;
mod hsl;
mod jwt;

#[async_trait]
pub trait PoWChallenge {
    async fn get_proof(&self, jwt: &str) -> String;
}
