use std::time::Duration;
use base64::{
    Engine,
    prelude::BASE64_URL_SAFE_NO_PAD
};
use fastrand::u16;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use random_string::generate;
use regex::Regex;
use reqwest::{
    Client,
    ClientBuilder,
    Proxy,
    StatusCode,
};
use crate::{
    commons::error::DortCapError::CodeErr,
    commons::error::FCaptchaResult,
    PROXIES,
    tools::generators::outlook::extract_value
};

lazy_static! {
    static ref ANCHOR_TYPE_REGEX: Regex = Regex::new("([api2|enterprise]+)\\/anchor\\?(.*)").expect("Regex instance creation failed");
}

pub struct ReCaptchaV3<'a> {
    site_key: &'a str,
    site_url: &'a str,
    client: Client
}

impl<'a> ReCaptchaV3<'a> {

    pub async fn new(site_key: &'a str, site_url: &'a str) -> FCaptchaResult<ReCaptchaV3<'a>> {
        let mut client;
        loop {
            let proxies = PROXIES.read().await;
            let proxy = fastrand::choice(&*proxies).ok_or(CodeErr(0x01, "PROXIES"))?.clone();
            client = ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .timeout(Duration::from_secs(10))
                .proxy(Proxy::all(proxy.replace("%SESSION_ID%", &*generate(23, "abcdef1234567890")).replace("%RND_PORT%", &*u16::to_string(&u16(10000..20000))))?)
                .build()?;
            drop(proxies);
            let check = client.get("https://ipinfo.io/ip").send().await;
            if let Err(check) = check {
                let err_code = check.status().unwrap_or(StatusCode::from_u16(200).unwrap());
                if err_code == 407 || err_code == 402 {
                    PROXIES.write().await.retain_mut(|a| !a.eq_ignore_ascii_case(&*proxy));
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(ReCaptchaV3 {
            site_key,
            site_url,
            client,
        })
    }

    pub async fn request_anchor(&self) -> FCaptchaResult<String> {
        let mut params: IndexMap<&str, &str> = IndexMap::new();
        let url = &*format!("{}.", BASE64_URL_SAFE_NO_PAD.encode(self.site_url));
        params.insert("ar", "1");
        params.insert("k", self.site_key);
        params.insert("co", url);
        params.insert("hl", "en-US");
        params.insert("v", "rz4DvU-cY2JYCwHSTck0_qm-");
        params.insert("size", "invisible");
        params.insert("cb", "fpe22z3udfoy");
        let resp = self.client.get("https://www.google.com/recaptcha/api2/anchor")
            .query(&params)
            .send().await?
            .text().await?;
        let base_url = extract_value(&*resp, "window['__recaptcha_api'] = '", "';").ok_or(CodeErr(0x01, "RECAP_ANCHOR"))?;
        let token = extract_value(&*resp, "id=\"recaptcha-token\" value=\"", "\"").ok_or(CodeErr(0x02, "RECAP_ANCHOR"))?;
        let mut params = IndexMap::new();
        params.insert("v", "rz4DvU-cY2JYCwHSTck0_qm-");
        params.insert("c", &*token);
        params.insert("reason", "q");
        params.insert("k", self.site_key);
        params.insert("co", url);
        let resp = self.client.post(format!("{}reload", base_url))
            .query(&params)
            .send().await?
            .text().await?;
        let token = extract_value(&*resp, "\"rresp\",\"", "\"").ok_or(CodeErr(0x03, "SOLVE_FAILED"))?;
        let score = self.client.get(format!("https://recaptcha-demo.appspot.com/recaptcha-v3-verify.php?action=examples/v3scores&token={token}")).send().await?.text().await?;
        println!("{}", score);
        Ok(token)
    }

}