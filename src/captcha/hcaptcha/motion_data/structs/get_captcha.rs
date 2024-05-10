use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;

use super::check_captcha::{Brand, TopLevel};

#[derive(Serialize, Deserialize)]
pub struct PreviousChallenge {
    pub escaped: bool,
    pub passed: bool,
    #[serde(rename = "expiredChallenge")]
    pub expired_challenge: bool,
    #[serde(rename = "expiredResponse")]
    pub expired_response: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UserAgentData {
    pub brands: Vec<Brand>,
    pub mobile: bool,
    pub platform: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetCaptchaMotionData {
    pub v: i64,
    #[serde(rename = "topLevel")]
    pub top_level: TopLevel,
    pub session: Vec<Value>,
    #[serde(rename = "widgetList")]
    pub widget_list: Vec<String>,
    #[serde(rename = "widgetId")]
    pub widget_id: String,
    pub href: String,
    pub prev: PreviousChallenge,
}