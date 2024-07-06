use std::collections::HashMap;
use std::ops::Deref;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use crate::captcha::hcaptcha::fingerprinting::hsw::HSW;
use crate::commons::error::FCaptchaResult;
use crate::conv_option;
use crate::tools::generators::outlook::extract_value;

lazy_static! {
    pub static ref REGEX: Regex = Regex::new(r#"\[\["[^"]*","[^"]*","[^"]*","[^"]*"\],\["[^"]*","[^"]*","[^"]*","[^"]*"\]\]"#).expect("REGEX_ERROR");
    pub static ref FETCHER: Client = Client::default();
    pub static ref HSW_VERSION_EVENT_CACHE: HashMap<String, HashMap<i128, Option<HSWEventType>>> = HashMap::new();
}

// TODO: Add more stuff.
pub enum HSWEventType {
    EncryptedWebGL,
    UserAgentData,
    Timezone,
    WebGL,
    Time
}

pub async fn get_hsw_events(version: &str) -> FCaptchaResult<&HashMap<i128, Option<HSWEventType>>> {
    if HSW_VERSION_EVENT_CACHE.contains_key(version) {
        return conv_option!(HSW_VERSION_EVENT_CACHE.get(version));
    }
    let hsw_data = FETCHER.get(format!("https://newassets.hcaptcha.com/c/{}/hsw.js", version)).send().await?.text().await?;
    let wasm_base64 = conv_option!(extract_value(hsw_data.deref(), "(0,null,\"", "\""))?;
    let decoded_wasm_binary = BASE64_STANDARD.decode(wasm_base64)?;
    todo!()
}

pub async fn generate_hsw_fp(version: &str) -> FCaptchaResult<String> {
    let events = get_hsw_events(version).await?;
    todo!()
}
