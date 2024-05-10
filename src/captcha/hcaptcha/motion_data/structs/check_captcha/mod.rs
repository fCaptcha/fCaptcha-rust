use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Brand {
    pub brand: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserAgentData {
    pub brands: Vec<Brand>,
    pub mobile: bool,
    pub platform: String,
}

#[derive(Serialize, Deserialize)]
pub struct EmptyStruct {}

#[derive(Serialize, Deserialize)]
pub struct NavigatorObject {
    #[serde(rename = "vendorSub")]
    pub vendor_sub: String,
    #[serde(rename = "productSub")]
    pub product_sub: String,
    pub vendor: String,
    #[serde(rename = "maxTouchPoints")]
    pub max_touch_points: i64,
    pub scheduling: EmptyStruct,
    #[serde(rename = "userActivation")]
    pub user_activation: EmptyStruct,
    #[serde(rename = "doNotTrack")]
    pub do_not_track: Option<u8>,
    pub geolocation: EmptyStruct,
    pub connection: EmptyStruct,
    #[serde(rename = "pdfViewerEnabled")]
    pub pdf_viewer_enabled: bool,
    #[serde(rename = "webkitTemporaryStorage")]
    pub webkit_temporary_storage: EmptyStruct,
    #[serde(rename = "windowControlsOverlay")]
    pub window_controls_overlay: EmptyStruct,
    #[serde(rename = "hardwareConcurrency")]
    pub hardware_concurrency: i64,
    #[serde(rename = "cookieEnabled")]
    pub cookie_enabled: bool,
    #[serde(rename = "appCodeName")]
    pub app_code_name: String,
    #[serde(rename = "appName")]
    pub app_name: String,
    #[serde(rename = "appVersion")]
    pub app_version: String,
    pub platform: String,
    pub product: String,
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    pub language: String,
    pub languages: Vec<String>,
    #[serde(rename = "onLine")]
    pub on_line: bool,
    pub webdriver: bool,
    #[serde(rename = "deprecatedRunAdAuctionEnforcesKAnonymity")]
    pub deprecated_run_ad_auction_enforces_kanonymity: bool,
    pub bluetooth: EmptyStruct,
    pub clipboard: EmptyStruct,
    pub credentials: EmptyStruct,
    pub keyboard: EmptyStruct,
    pub managed: EmptyStruct,
    #[serde(rename = "mediaDevices")]
    pub media_devices: EmptyStruct,
    pub storage: EmptyStruct,
    #[serde(rename = "serviceWorker")]
    pub service_worker: EmptyStruct,
    #[serde(rename = "virtualKeyboard")]
    pub virtual_keyboard: EmptyStruct,
    #[serde(rename = "wakeLock")]
    pub wake_lock: EmptyStruct,
    #[serde(rename = "deviceMemory")]
    pub device_memory: i64,
    #[serde(rename = "userAgentData")]
    pub user_agent_data: UserAgentData,
    pub login: EmptyStruct,
    pub ink: EmptyStruct,
    #[serde(rename = "mediaCapabilities")]
    pub media_capabilities: EmptyStruct,
    pub hid: EmptyStruct,
    pub locks: EmptyStruct,
    pub gpu: EmptyStruct,
    #[serde(rename = "mediaSession")]
    pub media_session: EmptyStruct,
    pub permissions: EmptyStruct,
    pub presentation: EmptyStruct,
    pub usb: EmptyStruct,
    pub xr: EmptyStruct,
    pub serial: EmptyStruct,
    pub plugins: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ScreenInfo {
    #[serde(rename = "availWidth")]
    pub avail_width: i64,
    #[serde(rename = "availHeight")]
    pub avail_height: i64,
    pub width: i64,
    pub height: i64,
    #[serde(rename = "colorDepth")]
    pub color_depth: i64,
    #[serde(rename = "pixelDepth")]
    pub pixel_depth: i64,
    #[serde(rename = "availLeft")]
    pub avail_left: i64,
    #[serde(rename = "availTop")]
    pub avail_top: i64,
    pub onchange: Option<Value>,
    #[serde(rename = "isExtended")]
    pub is_extended: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TopLevel {
    pub st: i64,
    pub sc: ScreenInfo,
    pub nv: NavigatorObject,
    pub dr: String,
    pub inv: bool,
    pub exec: bool,
    pub wn: Vec<i64>,
    #[serde(rename = "wn-mp")]
    pub wn_mp: i64,
    pub xy: Vec<Vec<i64>>,
    #[serde(rename = "xy-mp")]
    pub xy_mp: i64,
    pub mm: Vec<Vec<i64>>,
    #[serde(rename = "mm-mp")]
    pub mm_mp: f64,
}

#[derive(Serialize, Deserialize)]
pub struct CheckCaptchaMotionData {
    pub st: i64,
    pub dct: i64,
    pub mm: Vec<Vec<i64>>,
    #[serde(rename = "mm-mp")]
    pub mm_mp: f64,
    pub md: Vec<Vec<i64>>,
    #[serde(rename = "md-mp")]
    pub md_mp: f64,
    pub mu: Vec<Vec<i64>>,
    #[serde(rename = "mu-mp")]
    pub mu_mp: f64,
    #[serde(rename = "topLevel")]
    pub top_level: TopLevel,
    pub v: i64,
}