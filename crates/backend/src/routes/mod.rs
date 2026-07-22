mod get_search_result;

use axum::{routing::get, Router};
use database_helper::schema::{Entry, Site};
use mongodb::Collection;
use tower_http::cors::CorsLayer;

use get_search_result::get_search_result;

use crate::AppState;

pub fn create_routes(
    entries: Collection<Entry>,
    sites: Collection<Site>,
    total_count: u64,
) -> Router {
    let cors = CorsLayer::permissive();

    let app_state = AppState {
        entries: entries,
        sites: sites,
        total_entry_count: total_count,
    };

    Router::new()
        .route("/", get(get_search_result))
        .with_state(app_state)
        .layer(cors)
}
