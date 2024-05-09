use std::time::Duration;
use lazy_static::lazy_static;
use reqwest::{Client, ClientBuilder, Proxy};
use crate::commons::utils::get_proxy;
use crate::DORTCAP_CONFIG;
use crate::tools::generators::outlook::extract_value;

pub async fn fetch_blob() -> Option<String> {
    let client = ClientBuilder::new().proxy(Proxy::all(get_proxy().await?).ok()?).danger_accept_invalid_certs(true).timeout(Duration::from_secs(25)).build().unwrap();
    let body = client.get("https://octocaptcha.com").send().await.ok()?.text().await.ok()?;
    extract_value(&*body, "data-data-exchange-payload=\"", "\"")
}