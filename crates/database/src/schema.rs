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
    #[serde(default)]
    pub map: HashMap<String, WordInfo>,
    #[serde(default)]
    pub images: HashMap<String, u32>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            text: String::new(),
            map: HashMap::new(),
            images: HashMap::new(),
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
    pub favicon: String,
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,

    pub url: String,
    pub site: String,
    pub file_name: String,
    pub alt: String,
    pub title: String,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub text : String,
    pub normalized: String,

    pub short_answer: String,
    pub long_answer : String,

    pub answer_type: String,
    pub frequency: u64,
}






impl Default for Query {
    fn default() -> Self {
        Self {
            short_answer: String::new(),
            long_answer: String::new(),
            answer_type: "unknown".to_string(),
            frequency: 0,
            text: String::new(),
            normalized: String::new()


        }
    }
}
