// Still Not Implemented
// TODO: Implement it

use crate::sitemap;
use dashmap::DashSet;
use fast_robots::RobotsTxt;
use reqwest::{header::USER_AGENT, Client};
use std::sync::Arc;
use url::Url;

use common::constants::CUSTOM_USER_AGENT;

pub struct RobotsWrapper {
    content: String,
    pub sitemap: Vec<String>,
}

impl RobotsWrapper {
    pub fn is_allowed(&self, url: &str) -> bool {
        let robots = RobotsTxt::parse(&self.content);

        let parsed = Url::parse(url).unwrap();
        let path = parsed.path();

        robots.is_allowed("GBot", path)
    }

    pub fn get_sitemap_urls(&self) -> Vec<&str> {
        let robots = RobotsTxt::parse(&self.content);
        return robots.extensions.sitemaps;
    }
}

pub async fn get_robots(url: &str, client: &Arc<Client>) -> RobotsWrapper {
    let url = Url::parse(url).unwrap();
    let robots_url = url.join("/robots.txt").unwrap();

    let response = client
        .get(robots_url)
        .header(USER_AGENT, CUSTOM_USER_AGENT)
        .send()
        .await;

    if let Ok(content) = response {
        let content = content.text().await.unwrap_or("".to_string()).to_string();

        // The Logic is kinda sloppy here but it's ok for now
        let robots = RobotsTxt::parse(&content);

        // sitemap::get_site_map(robots.extensions.sitemaps, &mut site_map_urls,client);

        let result = Arc::new(DashSet::new());
        let visited = Arc::new(DashSet::new());

        let sitemap_urls: Vec<String> = robots
            .extensions
            .sitemaps
            .iter()
            .map(|url| url.to_string())
            .collect();


        sitemap::get_site_map(sitemap_urls, result.clone(), visited, client).await;


        // for url in result.iter() {
        //     println!("Item {}", url.key());
        // }

        return RobotsWrapper {
            content: content,
            sitemap: (*result)
                .iter()
                .map(|item| item.clone())
                .collect::<Vec<String>>(),
        };
    } else {
        let content = "".to_string();
        return RobotsWrapper {
            content: content,
            sitemap: vec![],
        };
    }
}
