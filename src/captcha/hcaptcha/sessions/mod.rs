pub mod structs;

use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct HCaptchaSession {
    pub site_key: String,
    pub site_url: String,
    pub rq_data: String
}

impl HCaptchaSession {
    pub async fn get_language(&self) -> String {
        String::from("")
    }
}
