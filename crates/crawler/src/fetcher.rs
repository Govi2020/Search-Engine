use common::constants::CUSTOM_USER_AGENT;
use reqwest::{header::CONTENT_TYPE, header::USER_AGENT, Client};
use std::sync::Arc;

pub async fn get_html_from_url(url: &str, client: &Arc<Client>) -> Result<String, reqwest::Error> {
    let response = client
        .get(url)
        .header(USER_AGENT, CUSTOM_USER_AGENT)
        .send()
        .await?;

    if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
        let content_type = content_type.to_str().unwrap();

        if !(content_type.contains("text/html")
            || content_type.contains("application/xhtml+xml")
            || content_type.contains("text/plain"))
        {
            return Ok("".to_string());
        }
    }
    response.text().await
}
