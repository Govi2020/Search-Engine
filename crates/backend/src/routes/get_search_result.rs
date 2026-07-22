use futures::future;
use serde::Deserialize;
use std::cmp::Ordering;
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
    let total_entry_count = state.total_entry_count;

    let query = query_params.query.clone();
    let query_filtered = utils::remove_unneeded_words(&query);
    let query_array: Vec<String> = utils::tokonize(&query_filtered);

    println!("HI");

    let query_entry_list: Vec<Entry> = Vec::new();

    let mut site_rarity_mapping: HashMap<String, Vec<f64>> = HashMap::new();
    let mut site_word_info_mapping: HashMap<String, Vec<WordInfo>> = HashMap::new();

    let futures = query_array
        .iter()
        .map(|word| database::get_entry(entries.clone(), word));

    let query_entry_list = futures::future::join_all(futures).await;

    for query_entry in query_entry_list {
        for (key, value) in &(query_entry.map) {
            let key = key.to_string();

            site_word_info_mapping
                .entry(key.clone())
                .or_default()
                .push(value.clone());

            site_rarity_mapping
                .entry(key.clone())
                .or_default()
                .push(calculate_rarity_of_word(
                    query_entry.map.len().try_into().unwrap(),
                    total_entry_count,
                ));
        }
    }

    let mut largest_len = 0;

    let mut word_freq: HashMap<usize, Vec<String>> = HashMap::new();

    for (key, value) in &site_word_info_mapping {
        let length = value.len();

        word_freq.entry(length).or_default().push(key.to_string());

        if largest_len < length {
            largest_len = length;
        }
    }

    let mut final_vector: Vec<String> = Vec::new();

    while largest_len > 0 {
        if let Some(site_vector) = word_freq.get(&largest_len) {
            let mut score_sheet: HashMap<String, f64> = HashMap::new();

            for site_id in site_vector {
                let score = calculate_score(
                    site_word_info_mapping.get(site_id).unwrap(),
                    site_rarity_mapping.get(site_id).unwrap(),
                );
                score_sheet.insert(site_id.to_string(), score);
            }

            let mut sorted: Vec<_> = score_sheet.iter().collect();

            sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(Ordering::Equal));

            println!("{:#?}", sorted);

            for item in sorted {
                final_vector.push(item.0.to_string());
            }
        }

        largest_len -= 1;
    }

    let futures = final_vector
        .iter()
        .map(|id| database::get_site(sites.clone(), id));

    let result = future::join_all(futures).await;

    return Json(result);
}

fn calculate_score(all_word_info: &Vec<WordInfo>, rarity_list: &Vec<f64>) -> f64 {
    let mut score: f64 = 0.0;

    for (index, word_info) in all_word_info.iter().enumerate() {
        // score += (*word_info).count * 2;
        // score += (*word_info).importance * 10;

        // let count = (word_info).count;
        // let importance = (word_info).importance;

        // TODO : USE ITF and IMPORTANCE AS WELL
        let itf = (word_info).term_frequency * 200.0;
        let importance = ((word_info).importance as f64 / 10.0) + 1.0;
        let idf = rarity_list.get(index).unwrap();

        score += itf * importance * idf;
    }

    return score;
}

fn calculate_rarity_of_word(current_entry_lenght: u64, total_lenght: u64) -> f64 {
    if current_entry_lenght == 0 {
        println!("Man it is 0");
        return 0.0;
    }

    let a: f64 = (total_lenght as f64 / current_entry_lenght as f64)
        .try_into()
        .unwrap();

    return a.log10();
}
