mod constants;
mod database;
mod fetcher;
mod parser;
mod robots;
mod schema;

use dashmap::DashMap;
use mongodb::Collection;
use schema::{Entry, Site};
use std::collections::{HashMap} ;
use std::sync::Arc;
use std::fs::File;
use std::io::{self, BufRead};
use reqwest::Client;

#[tokio::main]
async fn main() {
    let (entries, sites) = database::initialize_mongodb().await;
    let entries = Arc::new(entries);
    let sites = Arc::new(sites);
    let visited_urls: Arc<DashMap<String, bool>> = Arc::new(DashMap::new());

    // Seed Url File
    let seed_file = File::open("seed.txt").unwrap();


    let reader = io::BufReader::new(seed_file);
    let mut lines = reader.lines();

    let mut tasks : Vec<_>= Vec::new();

    let client = Arc::new(Client::new());

    loop {

        let url = match lines.next() {
            Some(result) => result.unwrap(),
            None => break,
        };

        let entries = Arc::clone(&entries);
        let sites = Arc::clone(&sites);
        let visited_urls = Arc::clone(&visited_urls);
        let client = Arc::clone(&client);


        tasks.push(tokio::spawn(async move {
            crawl_page(
                url.to_string(),
                entries,
                sites,
                visited_urls,
                client
            ).await
        }));
    }


    for task in tasks {
        let _ = task.await;
    }
}


#[async_recursion::async_recursion]
async fn crawl_page(
    url: String,
    entries: Arc<Collection<Entry>>,
    sites: Arc<Collection<Site>>,
    visited_urls: Arc<DashMap<String, bool>>,
    client: Arc<Client>
) {
    if visited_urls.contains_key(&url) {
        println!("Duplicate Key {:?} in Cache", url);
        return;
    }


    if database::does_site_exists(&sites, &url).await {
        println!("Duplicate Key {:?} in DataBase", url);
        return;
    }

    println!("Scraping URL : {}", url);

    let html = match fetcher::get_html_from_url(&url,&client).await {
        Ok(html) => html,
        Err(e) => {
            println!("Error fetching {}: {:?}", url, e);
            return;
        }
    };

    let (meta_data, link_list, word_scores) = process_html(&html,&url);
    let site_id = database::create_site((*sites).clone(), &url, meta_data, link_list.clone())
        .await;


    for (word, count, importance) in &word_scores {
        database::create_entry(&entries, word, *count, *importance, &site_id).await;
    }

    visited_urls.insert(url.clone(), true);

    let mut tasks : Vec<_>= Vec::new();

    println!("The Link List is {:#?}",link_list);

    for url in link_list {
        let entries = Arc::clone(&entries);
        let sites = Arc::clone(&sites);
        let visited_urls = Arc::clone(&visited_urls);
        let client = Arc::clone(&client);
        let url = url.to_string();

        tasks.push(tokio::spawn(async move {
            crawl_page(
                url,
                entries,
                sites,
                visited_urls,
                client
            ).await
        }));
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }


    for task in tasks {
        let _ = task.await;
    }
}

fn process_html(html: &str,url: &str) -> (HashMap<String, String>, Vec<String>, Vec<(String, i32, i32)>) {
    let document = parser::get_html_parser(html);

    let meta_data : HashMap<String, String> = parser::get_meta_data(&document);
    let link_list : Vec<String> = parser::get_link_list(&document,url);

    let html_text_content: String = parser::get_text_only(&document);

    let clean_text : String = parser::remove_unneeded_words(&html_text_content);

    let word_tokens = parser::tokonize(&clean_text);
    let word_freq_count : HashMap<String, i32> = parser::arrange_count(word_tokens.clone());

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
