use std::str::FromStr;
use ipgen::ip;
use random_string::generate;
use reqwest::header::{HeaderMap, HeaderValue};
use warp::head;
use warp::http::HeaderName;
use crate::captcha::arkose_funcaptcha::bda::firefox::ChromeHeaders;
use crate::commons::error::DortCapResult;
use crate::DORTCAP_CONFIG;
use super::bda::structs::ArkoseFingerprint;

pub async fn generate_headers(capi_version: &str, api_url: &str, headers_in: &ChromeHeaders) -> DortCapResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", HeaderValue::from_str("*/*")?);
    headers.insert("Accept-Encoding", HeaderValue::from_str("gzip, deflate, br, zstd")?);
    headers.insert("Accept-Language", HeaderValue::from_str(&*headers_in.accept_language)?);
    headers.insert("Content-Type", HeaderValue::from_str("application/x-www-form-urlencoded; charset=UTF-8")?);
    headers.insert("Origin", HeaderValue::from_str(&api_url)?);
    headers.insert("Referer", HeaderValue::try_from(format!("{0}/v2/{1}", api_url, capi_version))?);
    headers.insert("Sec-Ch-Ua", HeaderValue::try_from(&*headers_in.sec_ch_ua)?);
    headers.insert("Sec-Ch-Ua-Mobile", HeaderValue::try_from(&*headers_in.sec_ch_ua_mobile)?);
    headers.insert("Sec-Ch-Ua-Platform", HeaderValue::try_from(format!("\"{0}\"", &*headers_in.sec_ch_ua_platform))?);
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_str("empty")?);
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_str("cors")?);
    headers.insert("Sec-Fetch-Site", HeaderValue::from_str("same-origin")?);
    headers.insert("User-Agent", HeaderValue::try_from(&*headers_in.user_agent)?);
    Ok(headers)
}

pub async fn generate_headers_capi() -> DortCapResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", HeaderValue::from_str("*/*")?);
    headers.insert("Accept-Language", HeaderValue::from_str("en-US,en;q=0.9")?);
    headers.insert("Accept-Encoding", HeaderValue::from_str("gzip, deflate, br")?);
    headers.insert("Connection", HeaderValue::from_str("keep-alive")?);
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_str("empty")?);
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_str("cors")?);
    headers.insert("Sec-Fetch-Site", HeaderValue::from_str("same-origin")?);
    headers.insert("User-Agent", HeaderValue::try_from(format!("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:124.0) Gecko/20100101 Firefox/{}.0", fastrand::u16(0..14747)))?);
    Ok(headers)
}