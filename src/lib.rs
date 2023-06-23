#![allow(dead_code)]
mod api;
mod cache;
mod db;
pub mod status;

use tokio::sync::Mutex;

use api::BikeshareApi;
use cache::{Cache, Stale};

const API_URL: &'static str = "https://vancouver-ca.smoove.pro/api-public/stations";

#[derive(Debug)]
pub struct ServerState {
    api: BikeshareApi,
    cache: Mutex<Cache<Stale>>,
}

impl ServerState {
    pub async fn new() -> Self {
        Self {
            api: BikeshareApi::new(),
            cache: Mutex::new(Cache::new()),
        }
    }
}
