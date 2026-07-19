use futures::future;
use serde::Deserialize;
use std::collections::HashMap;

use database_helper::{
    database,
    schema::{Entry, Site, WordInfo},
};

use axum::{
    extract::{Query, State},
    Json,
};
use common::utils;

use crate::AppState;

#[derive(Deserialize)]
pub struct SearchQueryParams {
    query: String,
}

pub async fn get_search_result(
    query_params: Query<SearchQueryParams>,
    State(state): State<AppState>,
) -> Json<Vec<Site>> {
    // Remove ",","." etc
    // remove "this", "is" etc
    //

    let entries = state.entries;
    let sites = state.sites;

    let query = query_params.query.clone();
    let query_filtered = utils::remove_unneeded_words(&query);
    let query_array: Vec<String> = utils::tokonize(&query_filtered);

    println!("HI");

    let query_entry_list: Vec<Entry> = Vec::new();

    let mut site_word_mapping: HashMap<String, Vec<WordInfo>> = HashMap::new();

    let futures = query_array
        .iter()
        .map(|word| database::get_entry(entries.clone(), word));

    let query_entry_list = futures::future::join_all(futures).await;

    print!("Got Line");

    for query_entry in query_entry_list {
        for (key, value) in &(query_entry.map) {
            let key = key.to_string();

            site_word_mapping
                .entry(key.clone())
                .or_default()
                .push(value.clone());
        }
    }

    let mut largest_len = 0;

    let mut word_freq: HashMap<usize, Vec<String>> = HashMap::new();

    for (key, value) in &site_word_mapping {
        let length = value.len();

        word_freq.entry(length).or_default().push(key.to_string());

        if largest_len < length {
            largest_len = length;
        }
    }

    let mut final_vector: Vec<String> = Vec::new();

    while largest_len > 0 {
        if let Some(site_vector) = word_freq.get(&largest_len) {
            let mut score_sheet: HashMap<String, i32> = HashMap::new();

            for site_id in site_vector {
                let score = calculate_score(site_word_mapping.get(site_id).unwrap());

                score_sheet.insert(site_id.to_string(), score);
            }

            let mut sorted: Vec<_> = score_sheet.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));

            println!("{:#?}", sorted);

            for item in sorted {
                final_vector.push(item.0.to_string());
            }
        }

        largest_len -= 1;
    }

    let mut result: Vec<Site> = Vec::new();

    let futures = final_vector
        .iter()
        .map(|id| database::get_site(sites.clone(), id));

    let result = future::join_all(futures).await;

    return Json(result);
}

fn calculate_score(all_word_info: &Vec<WordInfo>) -> i32 {
    let mut score = 0;

    for word_info in all_word_info {
        score += (*word_info).count * 2;
        score += (*word_info).importance * 10;
    }

    return score;
}
