use tokio::fs::OpenOptions;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use reqwest::{ClientBuilder, Proxy};
use serde_json::{json, Value};
use random_string::generate;
use reqwest::header::{HeaderMap, HeaderValue};
use crate::DORTCAP_CONFIG;
use tokio::io::AsyncWriteExt;
use crate::captcha::arkose_funcaptcha::ArkoseSession;
use crate::captcha::arkose_funcaptcha::bda::templates::BDATemplate;
use crate::captcha::arkose_funcaptcha::structs::{FunCaptchaRequest};
use crate::commons::console::{created_account, solved};
use crate::commons::console::SolveType::INTERNAL;
use crate::commons::utils::get_proxy;

pub(crate) async fn register_roblox() {

    let username = generate(16, "abcdefghijklmnopqrstuvwxyz1234567890");
    let password = generate(20, "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!-_");
    let yy = fastrand::i64(1977..=2004);
    let mm = fastrand::i64(1..=13);
    let dd = fastrand::i64(1..=29);
    let data = json!({
        "username": username,
        "password": password,
        "birthday": format!(
            "{}-{}-{}T04:00:00.000Z",
            yy, mm, dd
        ),
        "gender": 1,
        "isTosAgreementBoxChecked": true,
        "agreementIds": [
            "adf95b84-cd26-4a2e-9960-68183ebd6393",
            "91b2d276-92ca-485f-b50d-c3952804cfd6",
        ],
    });
    // BEGIN AIDS
    let prxy = get_proxy().await;
    if prxy.is_none() {
        return;
    }
    let client_result = ClientBuilder::new().danger_accept_invalid_certs(true).proxy(Proxy::all(prxy.unwrap()).unwrap()).build();
    if client_result.is_err() {
        return;
    }
    let client = client_result.unwrap();
    let csrf_result = client.post("https://auth.roblox.com/v2/signup").json(&data.to_owned()).send().await;
    if csrf_result.is_err() {
        return;
    }
    let csrf_opt = csrf_result.unwrap();
    let csrf_opt_hv = csrf_opt.headers().get("x-csrf-token");
    if csrf_opt_hv.is_none() {
        return;
    }
    let csrf_to_str_result = csrf_opt_hv.unwrap().to_str();
    if csrf_to_str_result.is_err() {
        return;
    }
    let csrf = csrf_to_str_result.unwrap().to_owned();
    // END AIDS
    let mut headerss = HeaderMap::new();
    headerss.insert("x-csrf-token", HeaderValue::from_str(&csrf).unwrap());
    let blob_result = client.post("https://auth.roblox.com/v2/signup").headers(headerss).json(&data.to_owned()).send().await;
    if blob_result.is_err() {
        return;
    }
    let blob_resp = blob_result.unwrap();
    if let Some(challenge_header) = blob_resp.headers().get("rblx-challenge-metadata") {
        let header_value_str = challenge_header.to_str().unwrap();
        let decoded_bytes = BASE64_STANDARD.decode(header_value_str).unwrap();
        let json_str = String::from_utf8(decoded_bytes).unwrap();
        let parsed_json: Value = serde_json::from_str(&json_str).unwrap();
        let data_exchange_blob = parsed_json.get("dataExchangeBlob");
        if data_exchange_blob.is_none() {
            return;
        }
        let data_exchange_blob = data_exchange_blob.unwrap();
        let captcha_id = parsed_json.get("unifiedCaptchaId").unwrap();
        if let Some(challenge_id_value) = blob_resp.headers().get("rblx-challenge-id") {
            let challenge_id = challenge_id_value.to_str().unwrap().to_owned();
            let session = ArkoseSession::new(FunCaptchaRequest {
                site_url: String::from("https://www.roblox.com"),
                site_key: String::from("A2A14B1D-1AF3-C791-9BBC-EE33CC7A0A6F"),
                bda_template: BDATemplate {
                    document_referrer: Some(String::from("https://www.roblox.com/")),
                    window_ancestor_origins: Some(vec![
                        String::from("https://www.roblox.com/"),
                        String::from("https://www.roblox.com/")
                    ]),
                    window_tree_index: Some(vec![0, 0]),
                    window_tree_structure: Some(String::from("[[[]]]")),
                    window_location_href: None,
                    client_config_sitedata_location_href: Some(String::from("https://www.roblox.com/arkose/iframe")),
                    client_config_surl: Some(String::from("https://roblox-api.arkoselabs.com")),
                    client_config_language: None,
                },
                data: Some(String::from(data_exchange_blob.as_str().unwrap_or(""))),
                proxy: Some(DORTCAP_CONFIG.networking.proxy.replace("%SESSION_ID%", &*generate(23, "abcdef1234567890"))),
                arkose_api_url: String::from("https://roblox-api.arkoselabs.com"),
                audio: false,
            }).await;
            if session.is_err() {
                return;
            }
            let solve_result = session.unwrap().solve().await;
            if solve_result.is_err() {
                return;
            }
            let solve_result = solve_result.unwrap();
            if solve_result.error.is_some() {
                return;
            }
            solved(INTERNAL, solve_result.token.as_deref(), solve_result.variant.as_deref(), solve_result.waves.as_ref(), solve_result.solved.as_ref()).await;
            if solve_result.solved.unwrap() {
                let to_send = json!({
                    "unifiedCaptchaId": captcha_id,
                    "captchaToken": solve_result.token.to_owned(),
                    "actionType": "Signup"
                });
                let continue_req = json!({
                    "challengeId": challenge_id,
                    "challengeType": "captcha",
                    "challengeMetadata": to_send.to_string()
                });
                let mut headers = HeaderMap::new();
                headers.insert("X-Csrf-Token", HeaderValue::from_str(&csrf).unwrap());
                let continue_resp = client.post("https://apis.roblox.com/challenge/v1/continue").headers(headers).json(&continue_req).send().await;
                if continue_resp.is_err() {
                    return;
                }
                //challengeId
                let parsed_json = serde_json::from_str(&continue_resp.unwrap().text().await.unwrap());
                if parsed_json.is_err() {
                    return;
                }
                let parsed_json: Value = parsed_json.unwrap();
                if !parsed_json["challengeId"].is_string() {
                    return;
                }
                let challenge_id = parsed_json["challengeId"].as_str().unwrap();
                let encoded_b64 = BASE64_STANDARD.encode(&to_send.to_string());
                let mut headers = HeaderMap::new();
                headers.insert("X-Csrf-Token", HeaderValue::from_str(&csrf).unwrap());
                headers.insert("Rblx-Challenge-Id", HeaderValue::from_str(&challenge_id).unwrap());
                headers.insert("rblx-challenge-type", HeaderValue::from_static("captcha"));
                headers.insert("Rblx-Challenge-Metadata", HeaderValue::from_str(&encoded_b64).unwrap());
                headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36"));
                let resp_result = client.post("https://auth.roblox.com/v2/signup").headers(headers).json(&data.to_owned()).send().await;
                if resp_result.is_err() {
                    return;
                }
                let response = resp_result.unwrap().text().await.unwrap();
                let json_result = serde_json::from_str(&response);
                if json_result.is_err() {
                    return;
                }
                let json_result: Value = json_result.unwrap();
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open("logins.txt").await
                    .unwrap();
                if !json_result["userId"].is_null() {
                    created_account(Some(&*username));
                    let _ = file.write(format!("{username}:{password}\n").as_bytes()).await;
                }
            }
        }
    }
}