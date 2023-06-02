#![allow(unused_variables)]
mod api;
pub mod status;

use api::BikeshareApi;

const API_URL: &'static str = "https://vancouver-ca.smoove.pro/api-public/stations";

#[derive(Debug)]
pub struct ServerState {
    api: BikeshareApi,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            api: BikeshareApi::new(),
        }
    }
}
