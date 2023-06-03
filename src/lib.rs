#![allow(unused_variables)]
mod api;
mod cache;
pub mod status;

use api::BikeshareApi;
use cache::Cache;

const API_URL: &'static str = "https://vancouver-ca.smoove.pro/api-public/stations";

#[derive(Debug)]
pub struct ServerState {
    api: BikeshareApi,
    cache: Cache,
}

impl ServerState {
    pub async fn new() -> Self {
        Self {
            api: BikeshareApi::new(),
            cache: Cache::new().await.unwrap(),
        }
    }
}
