use std::{fs::File, sync::Arc};
use reqwest::{Client, Proxy};
use tokio::{sync::Mutex};
use std::io::{self, BufRead};

pub struct ProxyRotator {
    clients: Vec<Arc<Client>>,
    current: Mutex<usize>,
    count : usize
}

impl ProxyRotator {
    pub fn new(proxy_urls: Vec<&str>) -> Result<Self, reqwest::Error> {
        let mut clients = Vec::new();

        // Build a non-proxy one

        let client = Client::builder()
            .build()?;

        clients.push(Arc::new(client));

        if std::env::var("IS_PROXY").unwrap_or("1".to_string()) == "0" {
            return Ok(Self {
                clients,
                current: Mutex::new(0),
                count: 0
            });
        }

        // add proxy ones
        for proxy_url in proxy_urls {
            let proxy = Proxy::all(proxy_url)?;

            let client = Client::builder()
                .proxy(proxy)
                .build()?;

            clients.push(Arc::new(client));
        }

        Ok(Self {
            clients,
            current: Mutex::new(0),
            count: 0
        })
    }

    pub async fn next(&self) -> Arc<Client> {
        let mut current = self.current.lock().await;

        let client = Arc::clone(&self.clients[*current]);

        *current = (*current + 1) % self.clients.len();

        client
    }

    pub async fn current(&self) -> Arc<Client> {


        let max_req_with_proxy : usize = std::env::var("MAX_REQUESTS_WITH_EACH_PROXY").unwrap_or("20".to_string()).parse().unwrap();

        if self.count > max_req_with_proxy {
            return self.next().await;
        } else {
            let current = self.current.lock().await;
            let client = Arc::clone(&self.clients[*current]);
            return client;
        }
    }
}


pub fn configure_proxies() -> ProxyRotator {


    let seed_file = File::open("proxy_list.txt").unwrap();

    let reader = io::BufReader::new(seed_file);
    let mut lines = reader.lines();

    let mut proxy_list = Vec::new();


    loop {
        let url = match lines.next() {
            Some(result) => result.unwrap(),
            None => break, };


        proxy_list.push(url);
    };

    println!("Proxy List : {:?}",proxy_list);
    return ProxyRotator::new(proxy_list.iter().map(String::as_str).collect()).unwrap();

}
