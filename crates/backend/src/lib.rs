mod routes;

use database_helper::schema::{Entry, Site};

use mongodb::Collection;

#[derive(Clone)]
struct AppState {
    entries: Collection<Entry>,
    sites: Collection<Site>,
}

pub async fn run_server(entries: Collection<Entry>, sites: Collection<Site>) {
    let app = routes::create_routes(entries, sites);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
