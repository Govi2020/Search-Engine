use crate::constants::{CUSTOM_USER_AGENT};
use reqwest::{Client, header::USER_AGENT};
use std::sync::Arc;

pub async fn get_html_from_url(
    url: &str,
    client: &Arc<Client>
) -> Result<String, reqwest::Error> {

    let response = client
        .get(url)
        .header(USER_AGENT, CUSTOM_USER_AGENT)
        .send()
        .await?;

    response.text().await
}
