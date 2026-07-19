use crate::schema::{Entry, Site};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;

use mongodb::options::{ClientOptions, FindOneAndUpdateOptions, IndexOptions, ReturnDocument};
use mongodb::{Client, Collection, IndexModel};
use std::collections::HashMap;

pub async fn initialize_mongodb() -> (Collection<Entry>, Collection<Site>) {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");

    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    // Init Database and Collection
    let db = client.database("SearchEngine");
    let entries: Collection<Entry> = db.collection("entries");
    let sites: Collection<Site> = db.collection("sites");

    // Init Indexes
    create_indexes(&entries).await;

    (entries, sites)
}


async fn create_indexes(collection: &Collection<Entry>) {
    let index = IndexModel::builder()
        .keys(doc! {
            "text": 1
        })
        .options(
            IndexOptions::builder()
                .unique(true)
                .build()
        )
        .build();

    collection
        .create_index(index)
        .await
        .unwrap();
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
    count: i32,
    importance: i32,
    _site_id: &str,
) {
    let filter = doc! { "text": word };
    let update = doc! {
        "$set": {
            "text": word,
            format!("map.{}",_site_id)  : {
                "count": count,
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
