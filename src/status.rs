use axum::{extract::Query, http::StatusCode, Json, response::IntoResponse};
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct StationQuery {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct StatusApiData {
    result: Vec<StationStatus>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StationStatus {
    name: String,
    coordinates: String,
    total_slots: u8,
    free_slots: u8,
    avl_bikes: u8,
    operative: bool,
    style: String,
    is_estation: bool,
}

pub async fn station_status(_query: Query<StationQuery>) -> Result<Json<StationStatus>, StatusError> {
    let stations = fetch_stations(crate::API_URL.parse().unwrap()).await?;
    Ok(Json(stations.result[0].clone()))
}

async fn fetch_stations(uri: http::Uri) -> Result<StatusApiData, StatusError> {
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let response = client.get(uri)
        .await
        .map_err(|_| { StatusError::ApiFailure })?;
    let body = hyper::body::to_bytes(response.into_body())
        .await
        .map_err(|_| { StatusError::ApiFailure })?;
    Ok(serde_json::de::from_slice(&body)
        .map_err(|_| { StatusError::Parse })?)
}

/// Errors that can occur when retrieving the status of a bike station
pub enum StatusError {
    ApiFailure,
    Parse,
}

impl IntoResponse for StatusError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            StatusError::ApiFailure => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching station status from the bike share API",
            ),
            StatusError::Parse => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error parsing the response from the bike share API",
            ),
        };
        (status, message).into_response()
    }
}
