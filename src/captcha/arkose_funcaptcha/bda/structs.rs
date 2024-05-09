use crate::captcha::arkose_funcaptcha::bda::firefox::ChromeHeaders;
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn default_property_string() -> String {
    return String::from("false");
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FingerprintEntries {
    #[serde(rename = "DNT")]
    pub dnt: Option<String>,
    #[serde(rename = "L")]
    pub l: Option<String>,
    #[serde(rename = "D")]
    pub d: Option<String>,
    #[serde(rename = "PR")]
    pub pr: Option<String>,
    #[serde(rename = "S")]
    pub s: Option<String>,
    #[serde(rename = "AS")]
    pub r#as: Option<String>,
    #[serde(rename = "TO")]
    pub to: Option<String>,
    #[serde(rename = "SS")]
    pub ss: Option<String>,
    #[serde(rename = "LS")]
    pub ls: Option<String>,
    #[serde(rename = "IDB")]
    pub idb: Option<String>,
    #[serde(rename = "B")]
    pub b: Option<String>,
    #[serde(rename = "ODB")]
    pub odb: Option<String>,
    #[serde(rename = "CPUC")]
    pub cpuc: Option<String>,
    #[serde(rename = "PK")]
    pub pk: Option<String>,
    #[serde(rename = "CFP")]
    pub cfp: Option<String>,
    #[serde(rename = "FR", default = "default_property_string")]
    pub fr: String,
    #[serde(rename = "FOS", default = "default_property_string")]
    pub fos: String,
    #[serde(rename = "FB", default = "default_property_string")]
    pub fb: String,
    #[serde(rename = "JSF")]
    pub jsf: Option<String>,
    #[serde(rename = "P")]
    pub p: Option<String>,
    #[serde(rename = "T")]
    pub t: Option<String>,
    #[serde(rename = "H")]
    pub h: Option<String>,
    #[serde(rename = "SWF")]
    pub swf: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebGLEntries {
    pub webgl_extensions: Option<String>,
    pub webgl_extensions_hash: Option<String>,
    pub webgl_renderer: Option<String>,
    pub webgl_vendor: Option<String>,
    pub webgl_version: Option<String>,
    pub webgl_shading_language_version: Option<String>,
    pub webgl_aliased_line_width_range: Option<String>,
    pub webgl_aliased_point_size_range: Option<String>,
    pub webgl_antialiasing: Option<String>,
    pub webgl_bits: Option<String>,
    pub webgl_max_params: Option<String>,
    pub webgl_max_viewport_dims: Option<String>,
    pub webgl_unmasked_vendor: Option<String>,
    pub webgl_unmasked_renderer: Option<String>,
    pub webgl_vsf_params: Option<String>,
    pub webgl_vsi_params: Option<String>,
    pub webgl_fsf_params: Option<String>,
    pub webgl_fsi_params: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaDevice {
    #[serde(rename = "kind")]
    pub kind: Option<String>,
    #[serde(rename = "id")]
    pub id: Option<String>,
    #[serde(rename = "group")]
    pub group: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MediaDevices {
    #[serde(rename = "media_device_kinds")]
    pub media_device_kinds: Vec<String>,
    #[serde(rename = "media_devices")]
    pub media_devices: Vec<MediaDevice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkInfo {
    #[serde(rename = "downlink")]
    pub downlink: Option<f64>,
    #[serde(rename = "rtt")]
    pub rtt: Option<i32>,
    #[serde(rename = "save_data")]
    pub save_data: Option<bool>,
}

#[derive(Debug)]
pub struct ArkoseFingerprint {
    pub fingerprint_enc: String,
    pub user_agent: String,
    pub headers: ChromeHeaders,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fingerprint {
    pub webgl: WebGLEntries,
    pub fe: FingerprintEntries,
    #[serde(rename = "network")]
    pub network_info: NetworkInfo,
    #[serde(rename = "media")]
    pub media: MediaDevices,
    #[serde(rename = "headers")]
    pub headers: Value,
    #[serde(rename = "useragent")]
    pub useragent: String,
    #[serde(rename = "language")]
    pub language: Option<String>,
    #[serde(rename = "platform_header")]
    pub platform_header: Option<String>,
    #[serde(rename = "audio_fingerprint")]
    pub audio_fingerprint: Option<String>,
    #[serde(rename = "platform_key")]
    pub platform_key: Option<String>,
    #[serde(rename = "brands_is_mobile")]
    pub brands_is_mobile: Option<bool>,
    #[serde(rename = "brands_header")]
    pub brands_header: Option<String>,
    #[serde(rename = "brands_bda")]
    pub brands_bda: Option<String>,
    #[serde(rename = "languages_bda")]
    pub languages_bda: Option<String>,
    #[serde(rename = "languages_header")]
    pub languages_header: Option<String>,
}