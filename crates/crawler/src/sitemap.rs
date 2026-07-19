use std::{io::Cursor, sync::Arc};

use async_recursion::async_recursion;
use common::constants::CUSTOM_USER_AGENT;
use dashmap::DashSet;
use futures::future;
use reqwest::{header::USER_AGENT, Client};
use sitemap::reader::{SiteMapEntity, SiteMapReader};

#[async_recursion]
pub async fn get_site_map(
    sitemap_urls: Vec<String>,
    result: Arc<DashSet<String>>,
    visited: Arc<DashSet<String>>,
    client: &Arc<Client>,
) {
    let mut child_maps = Vec::new();

    for sitemap in sitemap_urls {
        // Skip if we've already processed this sitemap
        if !visited.insert(sitemap.clone()) {
            continue;
        }

        let response = match client
            .get(sitemap)
            .header(USER_AGENT, CUSTOM_USER_AGENT)
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => continue,
        };

        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(_) => continue,
        };

        let reader = SiteMapReader::new(Cursor::new(bytes));

        for entity in reader {
            match entity {
                SiteMapEntity::Url(entry) => {
                    if let Some(url) = entry.loc.get_url() {
                        result.insert(url.to_string());
                    }
                }

                SiteMapEntity::SiteMap(entry) => {
                    if let Some(url) = entry.loc.get_url() {
                        // Avoid spawning duplicate work

                        let a = url.to_string();
                        if !visited.contains(&a) {
                            child_maps.push(a);
                        }
                    }
                }

                SiteMapEntity::Err(_) => {
                    // Ignore malformed XML entries
                }
            }
        }
    }

    let tasks = child_maps
        .into_iter()
        .map(|url| get_site_map(vec![url], result.clone(), visited.clone(), client));

    future::join_all(tasks).await;
}
