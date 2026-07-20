mod fetcher;
mod parser;
mod robots;
mod sitemap;

use common::utils;
use database_helper::database;
use database_helper::schema::Entry;
use database_helper::schema::Site;

use dashmap::DashMap;
use mongodb::Collection;
use reqwest::Client;
use robots::RobotsWrapper;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::sync::Arc;
use url::Url;
use tokio::sync::Semaphore;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let (entries, sites) = database::initialize_mongodb().await;

    let entries = Arc::new(entries);
    let sites = Arc::new(sites);
    let visited_urls: Arc<DashMap<String, bool>> = Arc::new(DashMap::new());
    let queue: Arc<DashMap<String, bool>> = Arc::new(DashMap::new());
    let semaphore = Arc::new(Semaphore::new(60));

    let robots_list: Arc<DashMap<String, RobotsWrapper>> = Arc::new(DashMap::new());

    // let path = std::env::current_dir().unwrap();

    // println!("Running from: {}", path.display());

    // Seed Url File
    let seed_file = File::open("seed.txt").unwrap();

    let reader = io::BufReader::new(seed_file);
    let mut lines = reader.lines();

    let mut tasks: Vec<_> = Vec::new();

    let client = Arc::new(Client::new());

    loop {
        let url = match lines.next() {
            Some(result) => result.unwrap(),
            None => break,
        };
        println!("{:?}",url);

        let entries = Arc::clone(&entries);
        let sites = Arc::clone(&sites);
        let visited_urls = Arc::clone(&visited_urls);
        let queue = Arc::clone(&queue);
        let semaphore = Arc::clone(&semaphore);
        let robots_list = Arc::clone(&robots_list);
        let client = Arc::clone(&client);

        tasks.push(tokio::spawn(async move {
            crawl_page(
                url.to_string(),
                entries,
                sites,
                visited_urls,
                queue,
                semaphore,
                robots_list,
                client,
            )
            .await
        }));
    }

    for task in tasks {
        let _ = task.await;
    }
}

struct QueueGuard {
    queue: Arc<DashMap<String, bool>>,
    key: String,
}

impl Drop for QueueGuard {
    fn drop(&mut self) {
        self.queue.remove(&self.key);
    }
}

#[async_recursion::async_recursion]
async fn crawl_page(
    url: String,
    entries: Arc<Collection<Entry>>,
    sites: Arc<Collection<Site>>,
    visited_urls: Arc<DashMap<String, bool>>,
    queue: Arc<DashMap<String, bool>>,
    semaphore: Arc<Semaphore>,
    robots_list: Arc<DashMap<String, RobotsWrapper>>,
    client: Arc<Client>,
) {


    if queue.contains_key(&url) {
        return;
    }




    // Using Queue Guard to make sure to remove the queue after return or error
    let _queue_guard = QueueGuard {
        queue: queue.clone(),
        key: url.clone(),
    };


    // Checking for Duplicate Scraping
    if visited_urls.contains_key(&url) {
        println!("Duplicate Key {:?} in Cache", url);
        queue.remove(&url);
        return;
    }

    if database::does_site_exists(&sites, &url).await {
        println!("Duplicate Key {:?} in DataBase", url);
        queue.remove(&url);
        return;
    }


    // Using the Permit to limit concorrent requests
    let permit = semaphore.acquire().await.unwrap();


    // Add to the Queue

    queue.insert(url.clone(), true);


    let origin = Url::parse(&url).unwrap().origin().ascii_serialization();

    // Checking The Robots.txt

    let site_map_urls: Vec<String>;

    if robots_list.contains_key(&origin) {
        let robots = robots_list.get(&origin).unwrap();

        if !robots.is_allowed(&url) {
            return;
        }

        site_map_urls = robots.sitemap.clone();

    } else {
        let robots: RobotsWrapper = robots::get_robots(&url, &client).await;

        if !robots.is_allowed(&url) {
            return;
        }

        // For i need to use the site_map once
        // TODO : Calculate the performance cost for this and decide
        // site_map_urls = robots.sitemap.clone();
        site_map_urls = Vec::new();

        robots_list.insert(origin, robots);

    }


    let mut sitemap_tasks: Vec<_> = Vec::new();

    // Going though each url in sitemap
    for url in site_map_urls {
        let entries = Arc::clone(&entries);
        let sites = Arc::clone(&sites);
        let visited_urls = Arc::clone(&visited_urls);
        let queue = Arc::clone(&queue);
        let client = Arc::clone(&client);
        let semaphore = Arc::clone(&semaphore);
        let robots_list = Arc::clone(&robots_list);
        let url = url.to_string();
        // If i want i can check in the Queue and visited before making the task
        // TODO : For now i just put it here


        if queue.contains_key(&url) {
            continue;
        }

        if visited_urls.contains_key(&url) {
            queue.remove(&url);
            continue;
        }


        sitemap_tasks.push(tokio::spawn(async move {
            crawl_page(
                url,
                entries,
                sites,
                visited_urls,
                queue,
                semaphore,
                robots_list,
                client,
            )
            .await
        }));
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // println!("Scraping URL : {}", url);

    let html = match fetcher::get_html_from_url(&url, &client).await {
        Ok(html) => html,
        Err(e) => {
            println!("Error fetching {}: {:?}", url, e.status());
            return;
        }
    };

    if html == "" {
        return;
    }

    let (meta_data, link_list, word_scores) = process_html(&html, &url);
    println!(
        "[{:?}] Fetched URL : {} ",
        meta_data.get("title").unwrap(),
        url
    );

    let site_id = database::create_site((*sites).clone(), &url, meta_data, link_list.clone()).await;

    for (word, count, importance) in &word_scores {
        database::create_entry(&entries, word, *count, *importance, &site_id).await;
    }

    visited_urls.insert(url.clone(), true);

    let mut tasks: Vec<_> = Vec::new();


    drop(permit);

    for url in link_list {
        let entries = Arc::clone(&entries);
        let sites = Arc::clone(&sites);
        let visited_urls = Arc::clone(&visited_urls);
        let queue = Arc::clone(&queue);
        let client = Arc::clone(&client);
        let semaphore = Arc::clone(&semaphore);
        let robots_list = Arc::clone(&robots_list);
        let url = url.to_string();

        tasks.push(tokio::spawn(async move {
            crawl_page(
                url,
                entries,
                sites,
                visited_urls,
                queue,
                semaphore,
                robots_list,
                client,
            )
            .await
        }));
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    for task in sitemap_tasks {
        let _ = task.await;
    }

    for task in tasks {
        let _ = task.await;
    }
}

fn process_html(
    html: &str,
    url: &str,
) -> (
    HashMap<String, String>,
    Vec<String>,
    Vec<(String, i32, i32)>,
) {
    let document = parser::get_html_parser(html);

    let meta_data: HashMap<String, String> = parser::get_meta_data(&document);
    let link_list: Vec<String> = parser::get_link_list(&document, url);

    let html_text_content: String = parser::get_text_only(&document);

    let clean_text: String = utils::remove_unneeded_words(&html_text_content);

    let word_tokens = utils::tokonize(&clean_text);
    let word_freq_count: HashMap<String, i32> = parser::arrange_count(word_tokens.clone());

    let word_scores: Vec<(String, i32, i32)> = word_tokens
        .iter()
        .map(|word| {
            let count = word_freq_count.get(word).copied().unwrap_or(0);
            let importance = parser::calculate_importance(&document, word);
            (word.clone(), count, importance)
        })
        .collect();

    return (meta_data, link_list, word_scores);
}
