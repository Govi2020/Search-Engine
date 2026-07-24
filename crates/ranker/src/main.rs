use std::{collections::HashMap, pin::Pin};

use database_helper::database;
use futures::future::join_all;
use pagerank_rs::Pagerank;
use std::future::Future;

#[tokio::main]
async fn main() {
    let (_, sites, _) = database::initialize_mongodb().await;

    let all_pages = database::get_all_sites(sites.clone()).await.unwrap();
    let size = all_pages.len();

    let mut graph = Pagerank::new(size);

    let mut page_hash_map: HashMap<String, usize> = HashMap::new();

    for (index, page) in all_pages.iter().enumerate() {
        page_hash_map.insert(page.url.clone(), index);
    }

    for (index, page) in all_pages.iter().enumerate() {
        for url in &page.links {
            let link_index_result = page_hash_map.get(url);

            if let Some(link_index) = link_index_result {
                graph.link(index, *link_index).unwrap();
            }
        }
    }

    let damping_factor = 0.9;
    let tolerance = 0.0001;
    let mut tasks: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();

    // TODO:: ADD BULK IN MongoDB WRIES TO REDUCE LATENCY

    graph.rank(damping_factor, tolerance, |node_index, rank| {
        let site_id = all_pages
            .get(node_index)
            .unwrap()
            .id
            .unwrap()
            .to_string()
            .clone();
        tasks.push(Box::pin(database::update_page_rank(
            sites.clone(),
            site_id,
            rank,
        )));
    });

    join_all(tasks).await;
}
