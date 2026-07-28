use std::fs::File;
use std::io::{ BufRead, BufReader};
use std::path::Path;
use database_helper::{database, schema};
use serde::{Deserialize};


#[derive(Deserialize, Debug)]
struct QueryData {
    pub id : i64,
    pub question: Option<String>,
    pub short_answer: Option<String>,
    pub answer: Option<String>,
    pub answer_type: Option<String>,
    pub answer_url: Option<String>,
}

#[tokio::main]
async fn main() {
    // 1. Open the file path safely
    let path = Path::new("gogol.jsonl");
    let file = File::open(&path).unwrap();

    let (_,_,_,queries) = database::initialize_mongodb().await;

    // 2. Wrap the file handle in a buffered reader
    let reader = BufReader::new(file);

    // 3. Iterate over each line lazily

    for line_result in reader.lines() {
        // Handle I/O errors that might happen during reading
        let line = line_result.unwrap();

        let data : QueryData = serde_json::from_str(&line).unwrap();
        let answer_type = data.answer_type.unwrap_or("unknown".to_string());
        let mut question = data.question.unwrap_or("".to_string());
        question = question.trim().to_string();


        if question == "" {
            continue;
        }

        if question.starts_with("0800") {
            continue;
        }

        if answer_type.contains("_conv") {
            continue;

        }

        let normalized  = question.replace("?","");

        let short_answer = data.short_answer.unwrap_or("".to_string());
        let long_answer = data.answer.unwrap_or("".to_string());

        let query = schema::Query {
            text : question,
            normalized: normalized.to_string(),

            short_answer: short_answer,
            long_answer : long_answer,

            answer_type: answer_type,
            frequency: 0,
        };



        database::create_query(queries.clone(),query).await;
    }

}
