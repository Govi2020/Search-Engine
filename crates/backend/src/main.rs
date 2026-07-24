use backend::run_server;
use database_helper::database;

#[tokio::main]
async fn main() {
    let (entries, sites,images) = database::initialize_mongodb().await;

    run_server(entries, sites,images).await;
}
