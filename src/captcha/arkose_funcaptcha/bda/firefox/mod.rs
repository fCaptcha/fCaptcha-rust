use std::time::{SystemTime, UNIX_EPOCH};
use base64::{
    Engine,
    prelude::BASE64_STANDARD,
};
use chrono::{DateTime, Local, Utc};
use redis::AsyncCommands;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string, Value};
use uuid::Uuid;
use crate::{
    captcha::arkose_funcaptcha::{
        bda::{
            structs::ArkoseFingerprint,
            templates::BDATemplate
        },
        encryption::encrypt,
    },
    commons::error::DortCapResult,
    FINGERPRINTS,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChromeHeaders {
    #[serde(rename = "Sec-Ch-Ua")]
    pub sec_ch_ua: String,
    #[serde(rename = "Sec-Ch-Ua-Platform")]
    pub sec_ch_ua_platform: String,
    #[serde(rename = "Sec-Ch-Ua-Mobile")]
    pub sec_ch_ua_mobile: String,
    #[serde(rename = "User-Agent")]
    pub user_agent: String,
    #[serde(rename = "Accept-Language")]
    pub accept_language: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BDATemp {
    data: Value,
    headers: ChromeHeaders
}

pub async fn get_encrypted_firefox_bda(version_url: &str, bda_template: &mut BDATemplate) -> DortCapResult<ArkoseFingerprint> {
    let time = BASE64_STANDARD.encode(UNIX_EPOCH.elapsed()?.as_secs().to_string());
    let mut fps = FINGERPRINTS.get().await.clone();
    let cmd: String = redis::cmd("RANDOMKEY").query_async(&mut fps).await?;
    let data: String = fps.get(cmd).await?;
    let mut base_t: BDATemp = from_str(&*data)?;
    let mut base = &mut base_t.data;
    let uuid = Uuid::new_v4().to_string();
    if bda_template.window_location_href.is_none() {
        bda_template.window_location_href = Some(String::from(version_url));
    }
    base[2]["value"] = Value::from(time);
    let time = UNIX_EPOCH.elapsed()?.as_secs();
    bda_template.update(&mut base[4]["value"]);
    base[4]["value"][76]["value"] = Value::from(&*uuid);
    base[4]["value"][56]["value"] = Value::from(format!("{0}\\u0263", time / 1000));
    let time_range = time - time % 21600;
    let ua = &*base_t.headers.user_agent;
    let encrypted_bda = encrypt(&*base.to_string(), &format!("{}{time_range}", ua))?;
    let fingerprint = BASE64_STANDARD.encode(to_string(&encrypted_bda)?);
    Ok(ArkoseFingerprint {
        fingerprint_enc: fingerprint,
        user_agent: String::from(ua),
        headers: base_t.headers
    })
}