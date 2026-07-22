mod routes;

use database_helper::schema::{Entry, Site};

use mongodb::Collection;

#[derive(Clone)]
struct AppState {
    entries: Collection<Entry>,
    sites: Collection<Site>,
    total_entry_count: u64,
}

pub async fn run_server(entries: Collection<Entry>, sites: Collection<Site>) {
    let total_count = database_helper::database::get_total_no_of_documents(sites.clone()).await;
    let app = routes::create_routes(entries, sites, total_count);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
