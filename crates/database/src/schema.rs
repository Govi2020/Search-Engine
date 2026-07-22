use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordInfo {
    pub count: i32,
    pub importance: i32,
    pub term_frequency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub text: String,
    pub map: HashMap<String, WordInfo>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            text: String::new(),
            map: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub url: String,
    pub title: String,
    pub description: String,
    pub links: Vec<String>,
    pub page_rank: f64,
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
}
