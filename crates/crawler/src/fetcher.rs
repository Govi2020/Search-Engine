use common::constants::CUSTOM_USER_AGENT;
use reqwest::{header::CONTENT_TYPE, header::USER_AGENT, Client};
use url::Url;
use std::sync::Arc;

pub async fn get_html_from_url(url: &str, client: &Arc<Client>) -> Result<String, reqwest::Error> {

    let url_info = Url::parse(url).unwrap();

    if (url_info.cannot_be_a_base() || url_info.scheme() != "http" || url_info.scheme() != "https") {
            return Ok("".to_string());
    }




    let response = client
        .get(url)
        .header(USER_AGENT, CUSTOM_USER_AGENT)
        .send()
        .await?;

    if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
        // TODO : Make a much better status code system
        if response.status() != 200 {
            println!("Status {}",response.status());
            return Ok("".to_string());
        }
        let content_type = content_type.to_str().unwrap();

        if !(content_type.contains("text/html")
            || content_type.contains("application/xhtml+xml")
            || content_type.contains("text/plain"))
        {
            println!("Contenet=-type {:?}",content_type);
            return Ok("".to_string());
        }
    }
    response.text().await
}
