use axum::{extract::Query, routing::get, Router};
use http::StatusCode;
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};

const API_URL: &'static str = "https://vancouver-ca.smoove.pro/api-public/stations";

#[tokio::main]
async fn main() {
    let app = Router::new().route("/status", get(station_status));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

}

async fn station_status(_query: Query<StationQuery>) -> Result<String, StatusCode> {
    // Fetch and parse data feed
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let uri: http::Uri = API_URL.parse().unwrap();
    let response = client.get(uri)
        .await
        .unwrap();
        // .map_err(|_err| { StatusCode::INTERNAL_SERVER_ERROR })?;

    // Find requested station
    let body = hyper::body::to_bytes(response.into_body())
        .await
        .unwrap();
        // .map_err(|_| { StatusCode::INTERNAL_SERVER_ERROR })?;
    
    let stations: ApiResult = serde_json::de::from_slice(&body)
        .unwrap();
        // .map_err(|_| { StatusCode::INTERNAL_SERVER_ERROR })?;

    // Return result
    return Ok(serde_json::ser::to_string_pretty(&stations.result[0]).unwrap());
}

#[derive(Deserialize)]
struct StationQuery {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct ApiResult {
    result: Vec<StationStatus>,
}

#[derive(Serialize, Deserialize)]
struct StationStatus {
    name: String,
    coordinates: String,
    total_slots: u8,
    free_slots: u8,
    avl_bikes: u8,
    operative: bool,
    style: String,
    is_estation: bool,
}
