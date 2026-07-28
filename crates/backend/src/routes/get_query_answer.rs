

use axum::http::{HeaderMap, StatusCode};
use database_helper::schema::Image;
use futures::future;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

use database_helper::{database, schema::Site,schema::Entry};

use axum::response::IntoResponse;
use axum::{
    extract::{Query, State},
    Json,
};
use std::time::Instant;

use crate::AppState;

#[derive(Deserialize)]
pub struct SearchQueryParams {
    query: String,
}


pub async fn get_query_answer(
    query_params: Query<SearchQueryParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {

    let queries = state.queries;

    let query = query_params.query.clone().trim().to_string();

    let start = Instant::now();


    let result = database::get_query_answer(queries,&query).await;

    let duration = start.elapsed();
    let mut headers = HeaderMap::new();

    headers.insert(
        "x-search-time-ms",
        duration.subsec_millis().to_string().parse().unwrap(),
    );

    return (StatusCode::OK, headers, Json(result));
}
