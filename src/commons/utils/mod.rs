use reqwest::ClientBuilder;

pub async fn get_proxy() -> Option<String> {
    let client = ClientBuilder::new().cookie_store(true).build().ok()?;
    let response_text = client
        .get("http://46.4.64.139/proxy.php?Proxy=&new=&KeyAPI=a1208ec1288d7129d212c1212be6beff&protocol=ipv4")
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