use axum::http::{HeaderMap, StatusCode};
use database_helper::schema::Image;
use futures::future;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

use database_helper::{database, schema::Site};

use axum::response::IntoResponse;
use axum::{
    extract::{Query, State},
    Json,
};
use common::utils;
use std::time::Instant;

use crate::AppState;

#[derive(Deserialize)]
pub struct SearchQueryParams {
    query: String,
}

#[derive(Serialize)]
struct ImagesResult {
    url: String,
    site: Site,
}

pub async fn get_image_result(
    query_params: Query<SearchQueryParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Remove ",","." etc
    // remove "this", "is" etc
    //

    let entries = state.entries;
    let sites = state.sites;
    let images = state.images;

    let query = query_params.query.clone();
    let query_filtered = utils::remove_unneeded_words(&query);

    let language = utils::find_language(&query);

    let query_array: Vec<String> = utils::tokonize(&query_filtered, language);

    let mut image_score_mapping: HashMap<String, Vec<u32>> = HashMap::new();

    let start = Instant::now();

    let futures = query_array
        .iter()
        .map(|word| database::get_entry(entries.clone(), word));

    let query_entry_list = futures::future::join_all(futures).await;
    let mut temp_hash_map: HashMap<String, ()> = HashMap::new();

    for query_entry in query_entry_list {
        for (key, value) in &(query_entry.images) {
            let key = key.to_string();

            temp_hash_map.entry(key.clone()).or_insert(());

            image_score_mapping
                .entry(key.clone())
                .or_default()
                .push(value.clone());
        }
    }

    let mut image_tasks = Vec::new();

    for (image_id, _) in temp_hash_map {
        let images = images.clone();

        image_tasks.push(async move {
            let image = database::get_image(images, image_id).await;

            let id = image.id.unwrap().to_string();

            (id, image)
        });
    }

    let image_results = future::join_all(image_tasks).await;

    let mut image_hash_map: HashMap<String, Image> = HashMap::new();

    for (id, image) in image_results {
        image_hash_map.insert(id, image);
    }

    let mut site_tasks = Vec::new();

    for (_, image) in &image_hash_map {
        let sites = sites.clone();
        let site_id = image.site.to_string();

        site_tasks.push(async move {
            let site = database::get_site(sites, site_id).await;

            let id = site.id.unwrap().to_string();

            (id, site)
        });
    }

    let site_results = future::join_all(site_tasks).await;

    let mut site_hash_map: HashMap<String, Site> = HashMap::new();

    for (id, site) in site_results {
        site_hash_map.insert(id, site);
    }

    let mut largest_len = 0;

    let mut word_freq: HashMap<usize, Vec<String>> = HashMap::new();

    for (key, value) in &image_score_mapping {
        let length = value.len();

        word_freq.entry(length).or_default().push(key.to_string());

        if largest_len < length {
            largest_len = length;
        }
    }

    let mut final_vector: Vec<String> = Vec::new();

    while largest_len > 0 {
        if let Some(image_vector) = word_freq.get(&largest_len) {
            let mut score_sheet: HashMap<String, f64> = HashMap::new();

            for image_id in image_vector {
                let mut final_score = 0.0;
                let mut page_rank = site_hash_map
                    .get(&image_hash_map.get(image_id).unwrap().site)
                    .unwrap()
                    .page_rank;

                if page_rank == 0.0 {
                    page_rank = 0.001;
                }

                for score in image_score_mapping.get(image_id).unwrap() {
                    final_score += *score as f64;
                }
                score_sheet.insert(image_id.to_string(), final_score * page_rank);
            }

            let mut sorted: Vec<_> = score_sheet.iter().collect();

            sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(Ordering::Equal));

            // println!("{:#?}", sorted);

            for item in sorted {
                final_vector.push(item.0.to_string());
            }
        }

        largest_len -= 1;
    }

    let futures = final_vector
        .iter()
        .map(|id| image_hash_map.get(id).unwrap().clone());

    let image_list = futures.collect::<Vec<Image>>();

    let mut result: Vec<ImagesResult> = Vec::new();

    println!("{:?}", image_list);

    for image in image_list {
        let site = site_hash_map.get(&image.site).unwrap();
        let url = image.url;

        let image_result = ImagesResult {
            url: url,
            site: site.clone(),
        };
        result.push(image_result);
    }
    let duration = start.elapsed();
    let mut headers = HeaderMap::new();

    headers.insert(
        "x-search-time-ms",
        duration.subsec_millis().to_string().parse().unwrap(),
    );
    return (StatusCode::OK, headers, Json(result));
}
