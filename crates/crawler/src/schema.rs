use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordInfo {
    count: i32,
    importance: i32,
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
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
}
