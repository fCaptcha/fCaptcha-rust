mod user;

use std::collections::HashMap;
use std::str::FromStr;
use random_string::generate;
use redis::AsyncCommands;
use rocket::http::Status;
use rocket::{get, post};
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use serde_json::{json, Value};
use crate::REDIS_USERS_PPU;
use crate::commons::REDIS_RUNTIME;

#[get("/balance?<key>")]
pub async fn get_balance(key: &str) -> Custom<String> {
    let mut redis_users = REDIS_USERS_PPU.get().await.to_owned();
    let user = redis_users.hgetall::<String, HashMap<String, String>>(key.to_string()).await;
    if let Ok(user) = user {
        let balance = user.get("balance");
        if let Some(balance) = balance {
            let balance: f64 = f64::from_str(balance).unwrap();
            return Custom(Status::Ok, json!({"balance": balance}).to_string());
        }
    }
    return Custom(Status::Unauthorized, json!({"error": "KEY_NOT_FOUND"}).to_string());
}

fn create_key(secret: &str, data: Json<Value>) -> Option<String> {
    let rnd_key = generate(32, "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
    if secret.eq("6B3F04C500AF7D5B29232E93F77CD819767148C82918ECBC45DE25E3446DED5F") {
        let event = data.get("event")?;
        let event = event.as_str()?;
        if event.eq("product:dynamic") {
            let data = data.get("data")?;
            let total = data.get("total")?;
            let amount = total.as_f64()?;
            let nk = rnd_key.clone();
            REDIS_RUNTIME.spawn(async move {
                REDIS_USERS_PPU.get().await.clone().hset(nk, String::from("balance"), amount).await.ok()?;
                Some(())
            });
            return Some(format!("Your key is: {}", rnd_key));
        }
    };
    Some(String::from("Failed to create key. Please contact support."))
}

#[post("/LFNXUIhoHQPJFMJfDoivvTmwGaVcehJpLf_x-xLyE40cnn4g_X2kad42Uw55g7zSuJPbCeth1ZtvDVBBpBIr3A?<secret>", data = "<data>")]
pub async fn topup(secret: &str, data: Json<Value>) -> Result<Custom<String>, Custom<&'static str>> {
    Ok(Custom(Status::Ok, create_key(secret, data).ok_or(Custom(Status::Unauthorized, "Nice key generator dumbass.."))?))
}


pub async fn reduce_bal(key: &str, price: f64) -> Option<()> {
    let mut redis_users = REDIS_USERS_PPU.get().await.clone();
    let user: HashMap<String, String> = redis_users.hgetall(key.to_string()).await.ok()?;
    let balance = user.get("balance")?;
    let mut balance: f64 = f64::from_str(balance).ok()?;
    if balance > 0.0 {
        balance -= price;
        REDIS_USERS_PPU.get().await.to_owned().hset(key, String::from("balance"), balance).await.ok()?;
    }
    Some(())
}