
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize};

use database_helper::database;

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


pub async fn get_search_suggestions(
    query_params: Query<SearchQueryParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Remove ",","." etc
    // remove "this", "is" etc
    //

    let queries = state.queries;

    let query = query_params.query.clone().trim().to_string();

    let start = Instant::now();

    let test = Vec::new();

    let mut result = database::get_suggestions(queries.clone(),&query,false,5,&test).await;

    // if result.len() < 3  {
    //     let mut new_result = database::get_suggestions(queries.clone(),&query,true,(5 - result.len()) as i64,&result).await;

    //     result.append(&mut new_result);
    // }


    let duration = start.elapsed();
    let mut headers = HeaderMap::new();

    headers.insert(
        "x-search-time-ms",
        duration.subsec_millis().to_string().parse().unwrap(),);
    return (StatusCode::OK, headers, Json(result));
}
