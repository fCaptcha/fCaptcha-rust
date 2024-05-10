use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use rocket::serde::{Deserialize, Serialize};
use crate::commons::error::DortCapError::CodeErr;
use crate::commons::error::DortCapResult;

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

pub async fn parse_jwt(jwt: &str) -> DortCapResult<DecodedJWT> {
    let mut split = jwt.split(".");
    split.next();
    let extracted = split.next().ok_or(CodeErr(0x01, "PARSE_PROOF_JWT"))?;
    let decoded = String::from_utf8(BASE64_STANDARD_NO_PAD.decode(extracted)?)?;
    Ok(serde_json::from_str(&*decoded)?)
}