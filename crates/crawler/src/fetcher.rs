use common::constants::CUSTOM_USER_AGENT;
use common::constants::RETRY_AFTER_TIME_CODES;
use common::constants::RETRY_ANOTHER_PROXY_CODES;

use reqwest::{Client, Proxy, header::{CONTENT_TYPE, USER_AGENT}};
use serde::de::Error;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::proxy_manager::ProxyRotator;


pub fn get_client() -> Arc<Client> {
    let is_proxy = std::env::var("IS_PROXY").unwrap_or("0".to_string());
    let client: Arc<Client>;

    if is_proxy == "0" {
        client = Arc::new(Client::new());
    }else {

        let proxy_uri = std::env::var("PROXY_URL").unwrap_or_default();
        let proxy = Proxy::all(proxy_uri).unwrap();

        client = Arc::new(Client::builder().proxy(proxy).build().unwrap_or(Client::new()));
    }

    return client;

}

#[async_recursion::async_recursion]
pub async fn get_html_from_url(
    url: &str,
    client: &Arc<Client>,
    proxy_rotator: &Arc<ProxyRotator>,
    count: u32,
) -> Result<String, Box<dyn std::error::Error>> {

    const MAX_RETRIES: u32 = 2;

    if count >= MAX_RETRIES {
        return Err("Maximum retry count exceeded".into());
    }

    let response = client
        .get(url)
        .header(USER_AGENT, CUSTOM_USER_AGENT)
        .send()
        .await?;

    let status = response.status();
    let status_code : i32 = status.as_u16().try_into().unwrap();

    for retry_status in RETRY_AFTER_TIME_CODES {
        if status_code == *retry_status {

            println!("Status {} for {}",status_code, url);

            let retry_after = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok())
                .map(|seconds| seconds + 5)
                .unwrap_or(65);

            println!("Retrying after {} seconds...", retry_after);

            sleep(Duration::from_secs(retry_after)).await;
            let client = &proxy_rotator.next().await;


            return get_html_from_url(url, client,proxy_rotator, count + 1).await;
        }
    }

    for retry_status in RETRY_ANOTHER_PROXY_CODES {

        if status_code == *retry_status {

            println!("Status {} for {}",status_code, url);

            println!("Retrying with new proxy ...");

            let client = &proxy_rotator.next().await;

            return get_html_from_url(url, client,proxy_rotator, count + 1).await;
        }
    }



    if !status.is_success() {
        println!("Status {} for {}", status, url);
        return Ok(String::new());
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if !(content_type.contains("text/html")
        || content_type.contains("application/xhtml+xml")
        || content_type.contains("text/plain"))
    {
        println!("Content-Type {:?}", content_type);
        return Ok(String::new());
    }

    let html = response.text().await?;

    Ok(html)
}
