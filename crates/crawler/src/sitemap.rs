use std::{io::Cursor, sync::Arc, time::Duration};

use async_recursion::async_recursion;
use common::constants::CUSTOM_USER_AGENT;
use dashmap::DashSet;
use futures::future;
use reqwest::{header::USER_AGENT, Client};
use sitemap::reader::{SiteMapEntity, SiteMapReader};
use tokio::time::Instant;

const SITEMAP_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(600);

pub async fn get_site_map(
    sitemap_urls: Vec<String>,
    result: Arc<DashSet<String>>,
    visited: Arc<DashSet<String>>,
    client: &Arc<Client>,
) {
    let deadline = Instant::now() + SITEMAP_DISCOVERY_TIMEOUT;
    get_site_map_until(sitemap_urls, result, visited, client, deadline).await;
}

#[async_recursion]
async fn get_site_map_until(
    sitemap_urls: Vec<String>,
    result: Arc<DashSet<String>>,
    visited: Arc<DashSet<String>>,
    client: &Arc<Client>,
    deadline: Instant,
) {
    if Instant::now() >= deadline {
        return;
    }

    let mut child_maps = Vec::new();

    for sitemap in sitemap_urls {
        if !visited.insert(sitemap.clone()) {
            continue;
        }

        if Instant::now() >= deadline {
            return;
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
            if Instant::now() >= deadline {
                return;
            }

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
        .map(|url| get_site_map_until(vec![url], result.clone(), visited.clone(), client, deadline));

    future::join_all(tasks).await;
}