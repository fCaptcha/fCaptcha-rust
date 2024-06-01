use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use rocket::serde::{Deserialize, Serialize};
use crate::commons::error::DortCapResult;
use crate::conv_option;

#[derive(Serialize, Deserialize)]
pub struct DecodedJWT {
    #[serde(rename = "s")]
    pub stamp_difficulty: u32,
    #[serde(rename = "t")]
    pub r#type: String,
    #[serde(rename = "d")]
    pub data: String,
    #[serde(rename = "l")]
    pub hsw_location: String,
    #[serde(rename = "i")]
    pub integrity: String,
    #[serde(rename = "n")]
    pub full_type: String
}

pub fn parse_jwt(jwt: &str) -> DortCapResult<DecodedJWT> {
    let mut split = jwt.split(".");
    split.next();
    let extracted = conv_option!(split.next())?;
    let decoded = String::from_utf8(BASE64_STANDARD_NO_PAD.decode(extracted)?)?;
    Ok(serde_json::from_str(&*decoded)?)
}