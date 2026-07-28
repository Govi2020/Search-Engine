use crate::schema::{Entry, Image, Query, Site};
use futures::StreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Document};

use regex;

use futures::stream::TryStreamExt;
use mongodb::options::{ClientOptions, FindOneAndUpdateOptions, IndexOptions, ReturnDocument};
use mongodb::{Client, Collection, IndexModel};
use std::collections::HashMap;

use dotenv::dotenv;

pub async fn initialize_mongodb() -> (Collection<Entry>, Collection<Site>, Collection<Image>,Collection<Query>) {
    dotenv().ok();

    let mongo_url = dotenv::var("MONGO_DB_URL").unwrap_or("mongodb://localhost:27017".to_string());
    println!("URL IS {:?}", mongo_url);

    let client_options = ClientOptions::parse(mongo_url)
        .await
        .expect("Failed to parse MongoDB connection string");

    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    // Init Database and Collection
    let db = client.database("SearchEngine2");
    let entries: Collection<Entry> = db.collection("entries");
    let sites: Collection<Site> = db.collection("sites");
    let images: Collection<Image> = db.collection("images");
    let queries: Collection<Query> = db.collection("queries");

    // Init Indexes
    create_indexes(&entries, "text",true,1).await;
    create_indexes(&sites, "url",true,1).await;
    create_indexes(&images, "url",true,1).await;
    create_indexes(&queries, "normalized",true,1).await;

    create_indexes(&queries, "frequency", false, -1).await;
        let index = IndexModel::builder()
            .keys(doc! {
                "normalized": 1,
                "frequency": -1
            })
            .build();

    queries.create_index(index).await;

    (entries, sites, images,queries)
}

async fn create_indexes<T>(collection: &Collection<T>, field: &str,is_unique: bool,sort: i32)
where
    T: Send + Sync,
{
    let index = IndexModel::builder()
        .keys(doc! {
            field: sort
        })
        .options(IndexOptions::builder().unique(is_unique).build())
        .build();

    collection.create_index(index).await.unwrap();
}

pub async fn get_total_no_of_documents(sites: Collection<Site>) -> u64 {
    let total_pages = sites.count_documents(doc! {}).await.unwrap_or(0);

    return total_pages;
}

pub async fn get_all_sites(sites: Collection<Site>) -> Result<Vec<Site>, mongodb::error::Error> {
    let all_sites: Vec<Site> = sites.find(doc! {}).await?.try_collect().await?;

    return Ok(all_sites);
}

pub async fn get_entry(entries: Collection<Entry>, text: &str) -> Option<Entry> {
    let filter = doc! {
        "text": text
    };

    let entry = entries.find_one(filter).await;

    entry.unwrap()
}

pub async fn get_suggestions(queries: Collection<Query>, text: &str,should_find_any: bool,limit: i64,ignore_list: &Vec<String>) -> Vec<String> {

    let mut symbol ="^";

    if should_find_any{
        symbol = "";

    }

    let filter = doc! {
        "text": {
            "$regex": format!("{}{}", symbol,regex::escape(text)),
            "$options": "i",
            "$nin": ignore_list
        }
    };


    let mut cursor = queries.find(filter).sort(doc! {"frequency" : -1}).limit(limit).await.unwrap();
    let mut result: Vec<String> = Vec::new();

    while cursor.has_next() {
        let text = cursor.next().await;
        if text.is_none() {
            break;
        }
        let text = text.unwrap();
        if text.is_err() {
            break;
        }
        let text = text.unwrap().text;
        result.push(text.to_string());
    }

    return result;
}


pub async fn get_query_answer(queries: Collection<Query>, text: &str) -> Query {
    let normalized = text.replace("?", "").trim().to_string();

    let filter = doc! {
        "normalized": normalized
    };

    let result = queries.find_one(filter).await.unwrap();

    if result.is_none() {
        return Query::default();
    }

    return result.unwrap();
}



pub async fn get_site(sites: Collection<Site>, site_id: String) -> Site {
    let site_id = ObjectId::parse_str(site_id).unwrap();

    let filter = doc! {
        "_id": site_id
    };

    sites.find_one(filter).await.unwrap().unwrap()
}

pub async fn get_image(images: Collection<Image>, image_id: String) -> Image {
    let image_id = ObjectId::parse_str(image_id).unwrap();

    let filter = doc! {
        "_id": image_id
    };

    images.find_one(filter).await.unwrap().unwrap()
}

pub async fn update_page_rank(sites: Collection<Site>, site_id: String, page_rank: f64) {
    let site_id = ObjectId::parse_str(site_id).unwrap();

    let filter = doc! {
        "_id": site_id
    };

    let options = FindOneAndUpdateOptions::builder()
        .upsert(true)
        .return_document(ReturnDocument::After)
        .build();

    let update = doc! {
        "$set": {
            "page_rank": page_rank
        }
    };

    match sites
        .find_one_and_update(filter, update)
        .with_options(options)
        .await
    {
        Ok(_result) => {}
        Err(e) => {
            println!("Error update Page Rank in site: {:?}", e);
        }
    }
}

pub async fn update_query(queries: Collection<Query>,query: &String) {
    let normalized = query.replace("?","").trim().to_string();

    let filter = doc! {
        "normalized" :normalized
    };

    let options = FindOneAndUpdateOptions::builder()
        .upsert(true)
        .return_document(ReturnDocument::After)
        .build();

    let update = doc! {
        "$inc": {
            "frequency": 1
        },
        "$setOnInsert": {
            "text": query.trim(),
            "normalized": query.replace("?","").trim(),
            "short_answer": "",
            "long_answer" : "",
            "answer_type": "unknown",
        }
    };

    match queries
        .find_one_and_update(filter, update)
        .with_options(options)
        .await
    {
        Ok(_result) => {}
        Err(e) => {
            println!("Error update Query frequency : {:?}", e);
        }
    }
}

pub async fn create_site(
    sites: Collection<Site>,
    url: &str,
    meta_data: HashMap<String, String>,
    link_list: Vec<String>,
) -> String {
    let title = meta_data.get("title").cloned().unwrap_or_default();
    let description = meta_data.get("description").cloned().unwrap_or_default();

    let site = Site {
        url: url.to_string(),
        title,
        description,
        links: link_list,
        id: None,
        page_rank: 0.0,
        favicon: meta_data.get("icon").cloned().unwrap_or("".to_string()),
    };

    let filter = doc! { "url": url };
    let update = doc! {
        "$set": {
            "url": &site.url,
            "title": &site.title,
            "description": &site.description,
            "links": &site.links,
            "page_rank": &site.page_rank,
            "favicon": &site.favicon
        }
    };

    let options = FindOneAndUpdateOptions::builder()
        .upsert(true)
        .return_document(ReturnDocument::After)
        .build();

    match sites
        .find_one_and_update(filter, update)
        .with_options(options)
        .await
    {
        Ok(result) => {
            if let Some(doc) = result {
                return doc.id.map(|id| id.to_hex()).unwrap_or_default();
            }
            String::new()
        }
        Err(e) => {
            println!("Error creating site: {:?}", e);
            String::new()
        }
    }
}

pub async fn does_site_exists(sites: &Collection<Site>, url: &str) -> bool {
    let filter = doc! { "url": url };
    let site = sites.find_one(filter).await.unwrap();
    if site.is_some() {
        return true;
    } else {
        return false;
    }
}

pub async fn create_image(
    images: Collection<Image>,
    image_url: String,
    image_info: (String, String, String),
    site_id: &str,
) -> String {
    let image = Image {
        id: None,
        url: image_url.to_string(),
        site: site_id.to_string(),
        title: image_info.2,
        alt: image_info.1,
        file_name: image_info.0,
    };

    let filter = doc! { "url": image_url };
    let update = doc! {
        "$set": {
            "url": &image.url,
            "title": &image.title,
            "alt": &image.alt,
            "site": &image.site,
            "file_name": &image.file_name,
        }
    };

    let options = FindOneAndUpdateOptions::builder()
        .upsert(true)
        .return_document(ReturnDocument::After)
        .build();

    match images
        .find_one_and_update(filter, update)
        .with_options(options)
        .await
    {
        Ok(result) => {
            if let Some(doc) = result {
                return doc.id.map(|id| id.to_hex()).unwrap_or_default();
            }
            String::new()
        }
        Err(e) => {
            println!("Error creating site: {:?}", e);
            String::new()
        }
    }
}

pub async fn create_entry(
    entries: &Collection<Entry>,
    word: &str,
    count: usize,
    total_no_of_words: usize,
    importance: i32,
    images_info: HashMap<String, u32>,
    _site_id: &str,
) {
    let filter = doc! { "text": word };

    let term_frequency = count as f64 / total_no_of_words as f64;

    let mut set_doc = Document::new();

    set_doc.insert("text", word);

    // Create the nested document
    let site_doc = doc! {
        "count": count as i32,
        "term_frequency": term_frequency,
        "importance": importance,
    };

    // Dynamic field name
    set_doc.insert(format!("map.{}", _site_id.to_string()), site_doc);
    if images_info.len() == 0 {
        set_doc.insert("images", doc! {});
    }

    for (image_id, score) in images_info {
        set_doc.insert(format!("images.{}", image_id), score);
    }
    let update = doc! {
        "$set": set_doc
    };

    let options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .upsert(true)
        .build();

    match entries
        .find_one_and_update(filter, update)
        .with_options(options)
        .await
    {
        Ok(_result) => {}
        Err(e) => {
            println!("Duplicate Key {:?} in Cache", e);
        }
    }
}

pub async fn create_entry_for_image(
    entries: &Collection<Entry>,
    word: &str,
    count: usize,
    total_no_of_words: usize,
    importance: i32,
    _site_id: &str,
) {
    let filter = doc! { "text": word };

    let term_frequency = count as f64 / total_no_of_words as f64;

    let update = doc! {
        "$set": {
            "text": word,
            format!("map.{}",_site_id)  : {
                "count": count as i32,
                "term_frequency": term_frequency,
                "importance": importance,
            }
        },
    };

    let options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .upsert(true)
        .build();

    match entries
        .find_one_and_update(filter, update)
        .with_options(options)
        .await
    {
        Ok(_result) => {}
        Err(e) => {
            println!("Duplicate Key {:?} in Cache", e);
        }
    }
}



pub async fn create_query(queries : Collection<Query>,query: Query) {
    let result = queries.insert_one(query).await;
}
