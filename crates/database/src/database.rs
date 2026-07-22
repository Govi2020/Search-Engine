use crate::schema::{Entry, Site};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;

use futures::stream::TryStreamExt;
use mongodb::options::{ClientOptions, FindOneAndUpdateOptions, IndexOptions, ReturnDocument};
use mongodb::{Client, Collection, IndexModel};
use std::collections::HashMap;

pub async fn initialize_mongodb() -> (Collection<Entry>, Collection<Site>) {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");

    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    // Init Database and Collection
    let db = client.database("SearchEngine2");
    let entries: Collection<Entry> = db.collection("entries");
    let sites: Collection<Site> = db.collection("sites");

    // Init Indexes
    create_indexes(&entries, "text").await;
    create_indexes(&sites, "url").await;

    (entries, sites)
}

async fn create_indexes<T>(collection: &Collection<T>, field: &str)
where
    T: Send + Sync,
{
    let index = IndexModel::builder()
        .keys(doc! {
            field: 1
        })
        .options(IndexOptions::builder().unique(true).build())
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

pub async fn get_entry(entries: Collection<Entry>, text: &str) -> Entry {
    let filter = doc! {
        "text": text
    };

    entries.find_one(filter).await.unwrap().unwrap()
}

pub async fn get_site(sites: Collection<Site>, site_id: &str) -> Site {
    let site_id = ObjectId::parse_str(site_id).unwrap();

    let filter = doc! {
        "_id": site_id
    };

    sites.find_one(filter).await.unwrap().unwrap()
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
        Ok(result) => {}
        Err(e) => {
            println!("Error update Page Rank in site: {:?}", e);
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
    };

    let filter = doc! { "url": url };
    let update = doc! {
        "$set": {
            "url": &site.url,
            "title": &site.title,
            "description": &site.description,
            "links": &site.links,
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

pub async fn create_entry(
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
