// Still Not Implemented
// TODO: Implement it

use reqwest::{Client, header::USER_AGENT,header::CONTENT_TYPE};
use std::sync::Arc;
use url::Url;
use fast_robots::RobotsTxt;

use common::constants::{CUSTOM_USER_AGENT};

pub struct RobotsWrapper<> {
    content: String,
}

impl RobotsWrapper {

    pub fn is_allowed(&self,url: &str) -> bool {
        let robots = RobotsTxt::parse(&self.content);

        let parsed = Url::parse(url).unwrap();
        let path = parsed.path();

        robots.is_allowed("GBot", path)
    }
}



pub async fn get_robots(url : &str,client: &Arc<Client>) -> RobotsWrapper {

    let url = Url::parse(url).unwrap();
    let robots_url = url.join("/robots.txt").unwrap();


    let response = client
        .get(robots_url)
        .header(USER_AGENT, CUSTOM_USER_AGENT)
        .send()
        .await;

    if let Ok(content) = response {
        let content =  content.text().await.unwrap_or("".to_string()).to_string();
        println!("The Response is Ok : {:?}",content);
        return RobotsWrapper{ content: content}
    } else {
        let content =  "".to_string();
        return RobotsWrapper{ content: content}
    }
}
