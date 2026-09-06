mod fetcher;
mod parser;
mod robots;
mod proxy_manager;
mod sitemap;

use common::utils;
use database_helper::database;
use database_helper::schema::Entry;
use database_helper::schema::Image;
use database_helper::schema::Site;

use dashmap::DashMap;
use mongodb::Collection;
use robots::RobotsWrapper;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::sync::Arc;
use tokio::sync::Semaphore;
use url::Url;
use dotenv::dotenv;

use crate::proxy_manager::ProxyRotator;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    dotenv().ok();

    let (entries, sites, images,_) = database::initialize_mongodb().await;

    let entries = Arc::new(entries);
    let sites = Arc::new(sites);
    let images = Arc::new(images);
    let visited_urls: Arc<DashMap<String, bool>> = Arc::new(DashMap::new());
    let queue: Arc<DashMap<String, bool>> = Arc::new(DashMap::new());

    let max_concurrency : usize = std::env::var("MAX_CONCURRENCY").unwrap_or("85".to_string()).parse().unwrap();


    let semaphore = Arc::new(Semaphore::new(max_concurrency));

    let robots_list: Arc<DashMap<String, RobotsWrapper>> = Arc::new(DashMap::new());

    let proxy_rotator = Arc::new(proxy_manager::configure_proxies());

    // let path = std::env::current_dir().unwrap();

    // println!("Running from: {}", path.display());

    // Seed Url File
    let seed_file = File::open("seed.txt").unwrap();

    let reader = io::BufReader::new(seed_file);
    let mut lines = reader.lines();

    let mut tasks: Vec<_> = Vec::new();

    loop {
        let url = match lines.next() {
            Some(result) => result.unwrap(),
            None => break,
        };

        let entries = Arc::clone(&entries);
        let sites = Arc::clone(&sites);
        let images = Arc::clone(&images);
        let visited_urls = Arc::clone(&visited_urls);
        let queue = Arc::clone(&queue);
        let semaphore = Arc::clone(&semaphore);
        let robots_list = Arc::clone(&robots_list);
        let proxy_rotator = Arc::clone(&proxy_rotator);

        tasks.push(tokio::spawn(async move {
            crawl_page(
                url.to_string(),
                entries,
                sites,
                images,
                visited_urls,
                queue,
                semaphore,
                robots_list,
                proxy_rotator,
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
    images: Arc<Collection<Image>>,
    visited_urls: Arc<DashMap<String, bool>>,
    queue: Arc<DashMap<String, bool>>,
    semaphore: Arc<Semaphore>,
    robots_list: Arc<DashMap<String, RobotsWrapper>>,
    proxy_rotator: Arc<ProxyRotator>
) {
    if queue.contains_key(&url) {
        return;
    }
    
    let url_info = Url::parse(&url).unwrap();
    let origin = url_info.origin().ascii_serialization();

    if !parser::is_valid_url(&url_info) {
        return;
    }

    // Add to the Queue
    queue.insert(url.clone(), true);

    // Using Queue Guard to make sure to remove the queue after return or error
    let _queue_guard = QueueGuard {
        queue: queue.clone(),
        key: url.clone(),
    };

    // Checking for Duplicate Scraping
    if visited_urls.contains_key(&url) {
        println!("Duplicate Key {:?} in Cache", url);
        return;
    }

    if database::does_site_exists(&sites, &url).await {
        println!("Duplicate Key {:?} in DataBase", url);
        return;
    }

    // Using the Permit to limit concorrent requests
    let permit = semaphore.acquire().await.unwrap();

    // Getting the Proxy

    let client = proxy_rotator.current().await;

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
        let images = Arc::clone(&images);
        let visited_urls = Arc::clone(&visited_urls);
        let queue = Arc::clone(&queue);
        let semaphore = Arc::clone(&semaphore);
        let robots_list = Arc::clone(&robots_list);
        let proxy_rotator = Arc::clone(&proxy_rotator);
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
                images,
                visited_urls,
                queue,
                semaphore,
                robots_list,
                proxy_rotator
            )
            .await
        }));
    }

    let html = match fetcher::get_html_from_url(&url, &client,&proxy_rotator,0).await {
        Ok(html) => html,
        Err(e) => {
            println!("Error fetching {}: {:?}", url,1);
            return;
        }
    };

    if html == "" {
        return;
    }

    // Extracting All of the data
    let (meta_data, link_list, total_no_of_words, word_scores, images_list) =
        process_html(&html, &url);

    println!(
        "[{:?}] Fetched URL : {} ",
        meta_data.get("title").unwrap_or(&" ".to_string()),
        url
    );

    let site_id = database::create_site((*sites).clone(), &url, meta_data, link_list.clone()).await;
    if site_id == "" {
        return;
    }

    let mut image_word_scores: HashMap<String, HashMap<String, u32>> = HashMap::new();

    let mut new_image_list: HashMap<String, (String, String, String)> = HashMap::new();

    for (a, b) in images_list {
        let image_id = database::create_image((*images).clone(), a, b.clone(), &site_id).await;
        new_image_list.insert(image_id, b);
    }

    // TODO: Right now Image and Site has difference entries , but the alt's in the images can be
    // treated a content in sites and so create_entry can actually create entry for both site and
    // image at the same time so do it

    for (id, info) in new_image_list {
        let file_name_formated = utils::format_file_name(info.0);

        let total_text = file_name_formated + " " + info.1.as_str() + " " + info.2.as_str();

        let language = utils::find_language(&total_text);
        let words = utils::tokonize(&total_text, language);

        let word_freq_count: HashMap<String, i32> = parser::arrange_count(words.clone());

        for (word, count) in word_freq_count {
            if !image_word_scores.contains_key(&word) {
                let mut new = HashMap::new();
                new.insert(id.clone(), count as u32);
                image_word_scores.insert(word, new);
            } else {
                image_word_scores
                    .get_mut(&word)
                    .unwrap()
                    .insert(id.clone(), count as u32);
            }
        }
    }

    for (word, count, importance) in &word_scores {
        let images_info: HashMap<String, u32> = image_word_scores
            .get(word)
            .unwrap_or(&HashMap::new())
            .clone();

        database::create_entry(
            &entries,
            word,
            *count,
            total_no_of_words,
            *importance,
            images_info,
            &site_id,
        )
        .await;
    }

    visited_urls.insert(url.clone(), true);

    let mut tasks: Vec<_> = Vec::new();

    drop(permit);

    for url in link_list {
        let entries = Arc::clone(&entries);
        let sites = Arc::clone(&sites);
        let images = Arc::clone(&images);
        let visited_urls = Arc::clone(&visited_urls);
        let queue = Arc::clone(&queue);
        let semaphore = Arc::clone(&semaphore);
        let robots_list = Arc::clone(&robots_list);
        let proxy_rotator = Arc::clone(&proxy_rotator);
        let url = url.to_string();

        tasks.push(tokio::spawn(async move {
            crawl_page(
                url,
                entries,
                sites,
                images,
                visited_urls,
                queue,
                semaphore,
                robots_list,
                proxy_rotator
            )
            .await
        }));
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
    usize,
    Vec<(String, usize, i32)>,
    HashMap<String, (String, String, String)>,
) {
    let document = parser::get_html_parser(html);

    let meta_data: HashMap<String, String> = parser::get_meta_data(&document, url);
    let link_list: Vec<String> = parser::get_link_list(&document, url);

    let images: HashMap<String, (String, String, String)> = parser::get_images(&document, url);

    let html_text_content: String = parser::get_text_only(&document);

    let clean_text: String = utils::remove_unneeded_words(&html_text_content);

    let language = utils::find_language(&clean_text);

    let word_tokens: Vec<String> = utils::tokonize(&clean_text, language);

    let total_no_of_words: usize = word_tokens.len();

    let word_freq_count: HashMap<String, i32> = parser::arrange_count(word_tokens.clone());

    let word_scores: Vec<(String, usize, i32)> = word_tokens
        .iter()
        .map(|word| {
            let count = word_freq_count.get(word).copied().unwrap_or(0);
            let importance = parser::calculate_importance(&document, word);
            (word.clone(), count as usize, importance)
        })
        .collect();

    return (meta_data, link_list, total_no_of_words, word_scores, images);
}
