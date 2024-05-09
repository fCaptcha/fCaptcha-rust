use fastrand::u16;
use reqwest::{Client, ClientBuilder, Proxy, Version};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use self::cipher::encrypt_pw;
use reqwest::header::{HeaderMap, HeaderValue};
use rand::Rng;
use random_string::generate;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use crate::{ARGUMENTS, DORTCAP_CONFIG};
use crate::captcha::arkose_funcaptcha::ArkoseSession;
use crate::captcha::arkose_funcaptcha::bda::templates::BDATemplate;
use crate::captcha::arkose_funcaptcha::structs::{FunCaptchaRequest};
use crate::commons::console::{solved};
use crate::commons::console::SolveType::INTERNAL;
use crate::commons::error::DortCapResult;
use crate::commons::error::DortCapError::InternalErr;

mod cipher;
mod date;

#[derive(Serialize, Deserialize, Debug)]
pub struct OutlookError {
    #[serde(rename = "code")]
    pub code: Option<String>,

    #[serde(rename = "data")]
    pub data: Option<String>,

    #[serde(rename = "field")]
    pub field: Option<String>,

    #[serde(rename = "stackTrace")]
    pub stack_trace: Option<String>,

    #[serde(rename = "telemetryContext")]
    pub telemetry_context: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutlookResponse {
    #[serde(rename = "error")]
    pub error: Option<OutlookError>,
}

#[derive(Debug, Clone)]
pub(crate) struct OutlookData {
    uaid: String,
    tcxt: String,
    canary: String,
    random_num: String,
    key: String,
    ski: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorData {
    #[serde(rename = "encAttemptToken")]
    pub enc_attempt_token: Option<String>,

    #[serde(rename = "dfpRequestId")]
    pub dfp_request_id: Option<String>,

    #[serde(rename = "arkoseBlob")]
    pub arkose_blob: Option<String>,
}

pub struct OutlookCreator {
    client: Client
}


async fn get_proxy() -> Option<String> {
    Some(DORTCAP_CONFIG.networking.proxy.replace("%SESSION_ID%", &*generate(23, "abcdef1234567890")).replace("%RND_PORT%", &*u16::to_string(&u16(10000..20000))))
}

pub fn extract_value(body: &str, start_pattern: &str, end_pattern: &str) -> Option<String> {
    return if let Some(start_index) = body.find(start_pattern) {
        if let Some(end_index) = body[start_index + start_pattern.len()..].find(end_pattern) {
            let value = &body[start_index + start_pattern.len()..start_index + start_pattern.len() + end_index];
            Some(value.to_string())
        } else {
            None
        }
    } else {
        None
    }
}

fn fix_text(text: &str) -> String {
    return text.replace("\\u002f", "/")
        .replace("\\u003a", ":")
        .replace("\\u0026", "&")
        .replace("\\u003d", "=")
        .replace("\\u002b", "+");
}

fn register_headers(canary: &str, tcxt: &str, uaid: &str, agent: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_static("application/json"));
    headers.insert("accept-encoding", HeaderValue::from_static("gzip, deflate, br"));
    headers.insert("accept-language", HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert("cache-control", HeaderValue::from_static("no-cache"));
    headers.insert("canary", HeaderValue::from_str(canary).unwrap());
    headers.insert("content-type", HeaderValue::from_static("application/json"));
    headers.insert("dnt", HeaderValue::from_static("1"));
    headers.insert("hpgid", HeaderValue::from_str(&format!("2006{}", rand::thread_rng().gen_range(10..99))).unwrap());
    headers.insert("origin", HeaderValue::from_static("https://signup.live.com"));
    headers.insert("pragma", HeaderValue::from_static("no-cache"));
    headers.insert("scid", HeaderValue::from_static("100118"));
    headers.insert("sec-ch-ua", HeaderValue::from_static("\" Not A;Brand\";v=\"107\", \"Chromium\";v=\"96\", \"Google Chrome\";v=\"96\""));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("tcxt", HeaderValue::from_str(tcxt).unwrap());
    headers.insert("uaid", HeaderValue::from_str(uaid).unwrap());
    headers.insert("uiflvr", HeaderValue::from_static("1001"));
    headers.insert("User-Agent", HeaderValue::from_str(agent).unwrap());
    headers.insert("x-ms-apitransport", HeaderValue::from_static("xhr"));
    headers.insert("x-ms-apiversion", HeaderValue::from_static("2"));
    headers.insert("referer", HeaderValue::from_static("https://signup.live.com/?lic=1"));
    return headers;
}
impl OutlookCreator {
    // HOLY FUCK WHY DID I DO THIS LOL
    pub(crate) async fn new() -> Option<Self> {
        let client_builder = ClientBuilder::new().danger_accept_invalid_certs(true)
            .cookie_store(true)
            .proxy(Proxy::all(DORTCAP_CONFIG.networking.proxy.replace("%SESSION_ID%", &*generate(23, "abcdef1234567890")).replace("%RND_PORT%", &*u16::to_string(&u16(10000..20000)))).ok()?);
        let client = client_builder.build().ok()?;
        return Some(OutlookCreator { client });
    }

    async fn fetch_params(&self) -> Option<OutlookData> {
        let resp = self.client.get("https://signup.live.com/?lic=1&mkt=en-US").send().await.ok()?;
        let body = resp.text().await.ok()?;
        let uaid = extract_value(&body, "\"uaid\":\"", "\"")?;
        let tcxt = fix_text(&*extract_value(&body, "\"tcxt\":\"", "\"")?);
        let canary = fix_text(&*extract_value(&body, "\"apiCanary\":\"", "\"")?);
        let random_num = extract_value(&body, "var randomNum=\"", "\"")?;
        let key = extract_value(&body, "var Key=\"", "\"")?;
        let ski = extract_value(&body, "var SKI=\"", "\"")?;
        Some(OutlookData {
            uaid,
            tcxt,
            canary,
            random_num,
            key,
            ski
        })
    }


    pub(crate) async fn create_account(&self) -> DortCapResult<String> {
        let data = self.fetch_params().await.ok_or(InternalErr)?;
        let pw2 = &generate(11, "ABCDEFabcdef0123456890!");
        let pw = encrypt_pw(pw2, &data.random_num, &data.key);
        let xd = format!("{}{}", &ARGUMENTS.name_prefix, generate(13, "abcdef0123456890"));
        let xd2 = &generate(32, "abcdef0123456890");
        let mut body = json!({
            "RequestTimeStamp": date::get_date(),
            "MemberName": format!("{xd}@outlook.com"),
            "CheckAvailStateMap": [
                format!("{xd}@outlook.com")
            ],
            "EvictionWarningShown": [],
            "UpgradeFlowToken": {},
            "FirstName": "sirington",
            "LastName": "fardmann",
            "MemberNameChangeCount": 1,
            "MemberNameAvailableCount": 1,
            "MemberNameUnavailableCount": 0,
            "CipherValue": pw,
            "SKI": &data.ski,
            "BirthDate": "04:04:1943",
            "Country": "US",
            "AltEmail": null,
            "IsOptOutEmailDefault": true,
            "IsOptOutEmailShown": true,
            "IsOptOutEmail": true,
            "LW": true,
            "SiteId": "68692",
            "IsRDM": 0,
            "WReply": null,
            "ReturnUrl": null,
            "SignupReturnUrl": null,
            "uiflvr": 1001,
            "uaid": &data.uaid,
            "SuggestedAccountType": "OUTLOOK",
            "SuggestionType": "Locked",
            "HFId": &xd2,
            "encAttemptToken": "",
            "dfpRequestId": "",
            "scid": 100118,
            "hpgid": 201040,
        });
        let response = self.client.post("https://signup.live.com/API/CreateAccount?lic=1")
            .json(&body)
            .headers(register_headers(&data.canary, &data.tcxt, &data.uaid, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36"))
            .send()
            .await?;
        let json: OutlookResponse = response.json().await?;
        let error_json = json.error.ok_or(InternalErr)?;
        let error: ErrorData = from_str(&error_json.data.ok_or(InternalErr)?)?;
        body["encAttemptToken"] = json!(error.enc_attempt_token);
        body["dfpRequestId"] = json!(error.dfp_request_id);
        body["HType"] = json!("enforcement");
        let session: ArkoseSession = ArkoseSession::new(FunCaptchaRequest {
            site_url: String::from("https://iframe.arkoselabs.com"),
            site_key: String::from("B7D8911C-5CC8-A9A3-35B0-554ACEE604DA"),
            bda_template: BDATemplate {
                document_referrer: Some(String::from("https://iframe.arkoselabs.com/")),
                window_ancestor_origins: Some(vec![
                    String::from("https://iframe.arkoselabs.com"),
                    String::from("https://signup.live.com")
                ]),
                window_tree_index: Some(vec![
                    1,
                    0
                ]),
                window_tree_structure: Some(String::from("[[[]],[[]]]")),
                window_location_href: None,
                client_config_sitedata_location_href: Some(String::from("https://iframe.arkoselabs.com/B7D8911C-5CC8-A9A3-35B0-554ACEE604DA/index.html")),
                client_config_surl: Some(String::from("https://client-api.arkoselabs.com")),
                client_config_language: Some(String::from("en"))
            },
            audio: false,
            data: error.arkose_blob,
            proxy: None,
            arkose_api_url: String::from("https://client-api.arkoselabs.com"),
        }).await?;
        let solve_result = session.solve().await?;
        solved(INTERNAL, solve_result.token.as_deref(), solve_result.variant.as_deref(), solve_result.waves.as_ref(), solve_result.solved.as_ref()).await;
        if solve_result.solved.is_none() || !solve_result.solved.unwrap() {
            return Err(InternalErr);
        }
        body["HPId"] = json!("B7D8911C-5CC8-A9A3-35B0-554ACEE604DA");
        body["HSol"] = json!(solve_result.token);
        let response = self.client.post("https://signup.live.com/API/CreateAccount?lic=1")
            .json(&body)
            .headers(register_headers(&data.canary, &data.tcxt, &data.uaid, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36"))
            .version(Version::HTTP_11)
            .send()
            .await?;
        let t = response.text().await?;
        let json: OutlookResponse = from_str(&*t)?;
        if json.error.is_none() {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open("logins_outlook.txt").await?;
            file.write(format!("{xd}@outlook.com:{pw2}\n").as_bytes()).await?;
            return Ok(format!("{xd}@outlook.com:{pw2}"));
        }
        Err(InternalErr)
    }
}
