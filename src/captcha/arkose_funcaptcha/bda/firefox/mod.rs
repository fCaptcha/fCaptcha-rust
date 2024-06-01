use std::time::UNIX_EPOCH;

use base64::{
    Engine,
    prelude::BASE64_STANDARD,
};
use base64::prelude::BASE64_URL_SAFE;
use random_string::generate;
use redis::AsyncCommands;
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
use crate::captcha::arkose_funcaptcha::encryption::cryptojs_decrypt;

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
    // base[3]["value"] = Value::from(format!("{0}|72627afbfd19a741c7da1732218301ac", generate(32, "abcdef1234567890")));
    let time = UNIX_EPOCH.elapsed()?.as_secs();
    bda_template.update(&mut base[4]["value"]);
    // niggerish way of doing this, but I do not give 2 fucks.
    base[4]["value"][38]["value"] = Value::from(format!(r#"{0}\u2062"#, generate(32, "abcdef1234567890")));
    base[4]["value"][56]["value"] = Value::from(format!(r#"{0}\u2062"#, time * 1000));
    base[4]["value"][76]["value"] = Value::from(&*uuid);
    let json_str = &*base.to_string().replace("\\\\u2062", "\\u2062");
    let time_range = time - time % 21600;
    let ua = &*base_t.headers.user_agent;
    let encrypted_bda = encrypt(json_str, &format!("{}{time_range}", ua))?;
    // println!("{}", String::from_utf8(cryptojs_decrypt(&encrypted_bda.to_string(), &format!("{}{time_range}", ua)).unwrap()).unwrap());
    let fingerprint = BASE64_URL_SAFE.encode(to_string(&encrypted_bda)?);
    Ok(ArkoseFingerprint {
        fingerprint_enc: fingerprint,
        user_agent: String::from(ua),
        headers: base_t.headers
    })
}