mod get_image_result;
mod get_search_result;
mod get_search_suggestions;
mod get_query_answer;

use axum::{routing::get, Router};
use database_helper::schema::{Entry, Image, Query, Site};
use mongodb::Collection;
use tower_http::cors::CorsLayer;

use get_image_result::get_image_result;
use get_search_result::get_search_result;
use get_search_suggestions::get_search_suggestions;
use get_query_answer::get_query_answer;

use crate::AppState;

pub fn create_routes(
    entries: Collection<Entry>,
    sites: Collection<Site>,
    images: Collection<Image>,
    queries: Collection<Query>,
    total_count: u64,
) -> Router {
    let cors = CorsLayer::permissive();

    let app_state = AppState {
        entries: entries,
        sites: sites,
        images: images,
        queries: queries,
        total_entry_count: total_count,
    };

    Router::new()
        .route("/", get(get_search_result))
        .route("/images/", get(get_image_result))
        .route("/suggestions/", get(get_search_suggestions))
        .route("/answer/", get(get_query_answer))
        .with_state(app_state)
        .layer(cors)
}
