use random_string::generate;
use reqwest::ClientBuilder;

pub async fn get_proxy() -> Option<String> {
    // let client = ClientBuilder::new().cookie_store(true).build().ok()?;
    // let response_text = client
    //     .get("http://46.4.64.139/proxy.php?Proxy=&new=&KeyAPI=a1208ec1288d7129d212c1212be6beff&protocol=ipv4")
    //     .send().await.ok()?.text().await.ok()?;
    // let temp = response_text.replace("OK;HTTPS:", "");
    // let mut proxy_split = temp.split("@");
    // let mut ip_port_split = proxy_split.next()?.split(":");
    // let ip = ip_port_split.next()?;
    // let port = ip_port_split.next()?;
    // let mut user_pass_split = proxy_split.next()?.split(":");
    // let user = user_pass_split.next()?;
    // let password = user_pass_split.next()?;
    // Some(format!("http://{user}:{password}@{ip}:{port}"))
    Some(format!("http://Gym42JbGESQT5hEwGz6s-zone-adam-region-north_america-session-{}:Gym42JbGESQT5hEwGz6s0upc@pybpm-ins-eql5m9kg.pyproxy.io:2510", generate(16, "abcdef1234567890")))
}

pub async fn get_proxy_ipv6() -> Option<String> {
    let client = ClientBuilder::new().cookie_store(true).build().ok()?;
    let response_text = client
        .get("http://46.4.64.139/proxy.php?Proxy=&new=&KeyAPI=a1208ec1288d7129d212c1212be6beff&protocol=ipv6")
        .send().await.ok()?.text().await.ok()?;
    let temp = response_text.replace("OK;HTTPS:", "");
    let mut proxy_split = temp.split("@");
    let mut ip_port_split = proxy_split.next()?.split(":");
    let ip = ip_port_split.next()?;
    let port = ip_port_split.next()?;
    let mut user_pass_split = proxy_split.next()?.split(":");
    let user = user_pass_split.next()?;
    let password = user_pass_split.next()?;
    Some(format!("http://{user}:{password}@{ip}:{port}"))
}

#[macro_export] macro_rules! captcha_debug {
    ($x: expr) => {
        #[cfg(debug_assertions)]
        println!("DEBUG: {0}", $x);
    };
}

#[macro_export] macro_rules! conv_option {
    ($x:expr) => {
        match ($x) {
            Some(t) => {
                Ok(t)
            },
            None => {
                Err(crate::commons::error::DortCapError::DetailedInternalErr("UNWRAP_FAILED"))
            }
        }
    };
}