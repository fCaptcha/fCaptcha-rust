pub mod structs;

use std::time::Duration;
use reqwest::{Client, ClientBuilder, Proxy};
use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;
use DortCapError::DetailedInternalErr;
use crate::commons::error::DortCapError::CodeErr;
use crate::commons::error::{DortCapError, DortCapResult};
use crate::PROXIES;

pub struct HCaptchaSession {
    site_key: String,
    site_url: String,
    rq_data: Option<String>,
    client: Client
}

macro_rules! conv_option {
    ($x:expr) => {
        match ($x) {
            Some(t) => {
                Ok(t)
            },
            None => {
                Err(DetailedInternalErr("UNWRAP_FAILED"))
            }
        }
    };
}

impl HCaptchaSession {

    pub async fn new(site_key: &str, site_url: &str, rq_data: Option<&str>) -> DortCapResult<HCaptchaSession> {
        let proxies = PROXIES.read().await;
        let proxy = conv_option!(fastrand::choice(&*proxies))?;
        let client = ClientBuilder::new()
            .proxy(Proxy::all(proxy)?)
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .build()?;
        Ok(HCaptchaSession {
            site_key: String::from(site_key),
            site_url: String::from(site_url),
            rq_data: rq_data.map(|rq_data| {
                String::from(rq_data)
            }),
            client,
        })
    }

    pub async fn get_requester_question(&self) -> String {
        unimplemented!()
    }
    pub async fn get_language(&self) -> String {
        unimplemented!()
    }
}
