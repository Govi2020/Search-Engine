use axum::http::{HeaderMap, StatusCode};
use database_helper::schema::Image;
use futures::{StreamExt, TryStreamExt};
use mongodb::bson::doc;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

use database_helper::schema::Site;

pub async fn get_image_result(
    query_params: Query<SearchQueryParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let images = state.images;
    let sites = state.sites;

    let query = query_params.query.clone();
    let query_filtered = utils::remove_unneeded_words(&query);

    let language = utils::find_language(&query);

    let query_array: Vec<String> = utils::tokonize(&query_filtered, language);

    let start = Instant::now();

    // Rank all images that mention any query word in alt / title / file_name.
    let mut candidates: Vec<(Image, u32, usize)> = Vec::new();
    if !query_array.is_empty() {
        let regex = format!(
            "{}",
            regex::escape(&query_array.join("|"))
        );

        let filter = doc! {
            "$or": [
                { "alt": { "$regex": regex.clone(), "$options": "i" } },
                { "title": { "$regex": regex.clone(), "$options": "i" } },
                { "file_name": { "$regex": regex, "$options": "i" } },
            ]
        };

        let mut cursor = match images.find(filter).limit(100).await {
            Ok(c) => c,
            Err(e) => {
                println!("Image query error: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), Json(Vec::<ImagesResult>::new()));
            }
        };

        while let Some(Ok(image)) = cursor.next().await {
            let mut matched_words = 0;
            let mut total_score: u32 = 0;

            let alt = image.alt.to_lowercase();
            let title = image.title.to_lowercase();
            let file_name = image.file_name.to_lowercase();

            for word in &query_array {
                let w = word.to_lowercase();
                if alt.contains(&w) || title.contains(&w) || file_name.contains(&w) {
                    matched_words += 1;
                    total_score += 1;
                }
            }

            if matched_words > 0 {
                candidates.push((image, total_score, matched_words));
            }
        }
    }

    // Load the site for every candidate in one round-trip.
    let mut site_ids: Vec<mongodb::bson::oid::ObjectId> = Vec::new();
    for (image, _, _) in &candidates {
        if let Ok(id) = mongodb::bson::oid::ObjectId::parse_str(&image.site) {
            site_ids.push(id);
        }
    }
    site_ids.dedup();

    let mut site_hash_map: HashMap<String, Site> = HashMap::new();
    if !site_ids.is_empty() {
        let site_docs = match sites.find(doc! { "_id": { "$in": site_ids } }).await {
            Ok(c) => match c.try_collect::<Vec<Site>>().await {
                Ok(docs) => docs,
                Err(e) => {
                    println!("Site fetch error: {:?}", e);
                    Vec::new()
                }
            },
            Err(e) => {
                println!("Site query error: {:?}", e);
                Vec::new()
            }
        };

        for site in site_docs {
            if let Some(id) = site.id {
                site_hash_map.insert(id.to_string(), site);
            }
        }
    }

    // Sort: most matched words first, then total score, then page rank.
    candidates.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| {
                let pa = site_hash_map
                    .get(&a.0.site)
                    .map(|s| s.page_rank)
                    .unwrap_or(0.0);
                let pb = site_hash_map
                    .get(&b.0.site)
                    .map(|s| s.page_rank)
                    .unwrap_or(0.0);
                pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let mut result: Vec<ImagesResult> = Vec::with_capacity(candidates.len());
    for (image, _, _) in candidates.into_iter().take(50) {
        let url = image.url.clone();
        if let Some(site) = site_hash_map.get(&image.site) {
            result.push(ImagesResult {
                url,
                site: site.clone(),
            });
        }
    }

    let duration = start.elapsed();
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-search-time-ms",
        duration.subsec_millis().to_string().parse().unwrap(),
    );

    (StatusCode::OK, headers, Json(result))
}