pub mod game_struct;
pub mod session_structs;

use serde::{Deserialize, Serialize};
use crate::captcha::arkose_funcaptcha::bda::templates::BDATemplate;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolvedCaptchaResponse {
    #[serde(rename = "variant")]
    pub variant: Option<String>,
    #[serde(rename = "token")]
    pub token: Option<String>,
    #[serde(rename = "solved")]
    pub solved: Option<bool>,
    #[serde(rename = "waves")]
    pub waves: Option<i32>,
    #[serde(rename = "error")]
    pub error: Option<String>,
    #[serde(rename = "notes")]
    pub notes: Option<Vec<String>>,
    #[serde(rename = "ip")]
    pub ip_used: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptionKeyResponse {
    #[serde(rename = "error")]
    pub error: Option<String>,
    #[serde(rename = "decryption_key")]
    pub decryption_key: Option<String>
}

#[derive(Debug)]
pub struct FunCaptchaRequest {
    pub proxy: Option<String>,
    pub site_url: String,
    pub site_key: String,
    pub data: Option<String>,
    pub bda_template: BDATemplate,
    pub arkose_api_url: String,
    pub audio: bool
}