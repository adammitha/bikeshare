#![allow(dead_code)]
mod api;
mod cache;
mod db;
pub mod status;

use db::Db;
use tokio::sync::Mutex;

use api::BikeshareApi;
use cache::Cache;

const API_URL: &'static str = "https://vancouver-ca.smoove.pro/api-public/stations";

#[derive(Debug)]
pub struct ServerState {
    cache: Mutex<Cache>,
}

impl ServerState {
    pub async fn new(db_url: Option<String>) -> Self {
        let db = match db_url {
            Some(url) => Db::new(&url).await.ok(),
            None => None,
        };
        Self {
            cache: Mutex::new(Cache::new(BikeshareApi::new(), db)),
        }
    }
}
