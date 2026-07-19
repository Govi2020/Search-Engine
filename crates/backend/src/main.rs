use backend::run_server;
use database_helper::database;

#[tokio::main]
async fn main() {
    let (entries, sites) = database::initialize_mongodb().await;

    run_server(entries, sites).await;
}
