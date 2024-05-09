use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use fmtools::obfstr;
use reqwest::{ClientBuilder, Proxy};
use serde_json::{json, Value};
use random_string::generate;
use reqwest::header::{HeaderMap, HeaderValue};
use crate::DORTCAP_CONFIG;
use crate::captcha::arkose_funcaptcha::ArkoseSession;
use crate::captcha::arkose_funcaptcha::bda::templates::BDATemplate;
use crate::captcha::arkose_funcaptcha::structs::{FunCaptchaRequest};
use crate::commons::console::solved;
use crate::commons::console::SolveType::INTERNAL;
use crate::commons::utils::get_proxy;
use crate::tools::generators::outlook::extract_value;

pub async fn login_roblox() -> Option<()> {
    let data = json!({
        "ctype": "Username",
        "cvalue": "123456",
        "password": "138749674913"
    });
    let prxy = get_proxy().await?;
    let client = ClientBuilder::new().danger_accept_invalid_certs(true).proxy(Proxy::all(prxy).ok()?).build().ok()?;
    let csrf_txt = client.get("https://www.roblox.com/").json(&data.to_owned()).send().await.ok()?.text().await.ok()?;
    let csrf = extract_value(&*csrf_txt, "=\"csrf-token\" data-token=\"", "\"")?;
    let mut headerss = HeaderMap::new();
    headerss.insert("x-csrf-token", HeaderValue::from_str(&*csrf).ok()?);
    let blob_resp = client.post("https://auth.roblox.com/v2/login").headers(headerss).json(&data.to_owned()).send().await.ok()?;
    let challenge_header = blob_resp.headers().get("rblx-challenge-metadata")?;
    let header_value_str = challenge_header.to_str().ok()?;
    let decoded_bytes = BASE64_STANDARD.decode(header_value_str).ok()?;
    let json_str = String::from_utf8(decoded_bytes).unwrap();
    let parsed_json: Value = serde_json::from_str(&json_str).ok()?;
    let data_exchange_blob = parsed_json.get("dataExchangeBlob")?;
    let session = ArkoseSession::new(FunCaptchaRequest {
        site_url: String::from("https://www.roblox.com"),
        site_key: String::from("476068BF-9607-4799-B53D-966BE98E2B81"),
        bda_template: BDATemplate {
            document_referrer: Some(String::from("https://www.roblox.com/")),
            window_ancestor_origins: Some(vec![
                String::from("https://www.roblox.com"),
                String::from("https://www.roblox.com")
            ]),
            window_tree_index: Some(vec![1, 0]),
            window_tree_structure: Some(String::from(obfstr!("[[],[[]]]"))),
            window_location_href: None,
            client_config_sitedata_location_href: Some(String::from("https://www.roblox.com/arkose/iframe")),
            client_config_surl: Some(String::from("https://roblox-api.arkoselabs.com")),
            client_config_language: None,
        },
        data: Some(String::from(data_exchange_blob.as_str().unwrap())),
        proxy: None,//(DORTCAP_CONFIG.networking.proxy.replace("%SESSION_ID%", &*generate(23, "abcdef1234567890"))),
        arkose_api_url: String::from("https://roblox-api.arkoselabs.com"),
        audio: false,
    }).await.ok()?;
    let solve_result = session.solve().await.ok()?;
    solved(INTERNAL, solve_result.token.as_deref(), solve_result.variant.as_deref(), solve_result.waves.as_ref(), solve_result.solved.as_ref()).await;
    if solve_result.solved.unwrap() {
    }
    Some(())
}