use axum::{extract::Query, http::StatusCode, Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::API_URL;

#[derive(Deserialize, Debug)]
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

#[instrument]
pub async fn station_status(_query: Query<StationQuery>) -> Result<Json<StationStatus>, StatusError> {
    let stations = fetch_stations(API_URL).await?;
    Ok(Json(stations.result[0].clone()))
}

async fn fetch_stations(uri: &str) -> Result<StatusApiData, StatusError> {
    Ok(reqwest::get(uri)
        .await?
        .json::<StatusApiData>()
        .await?)
}

/// Errors that can occur when retrieving the status of a bike station
pub enum StatusError {
    ApiFailure(reqwest::Error),
}

impl IntoResponse for StatusError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            StatusError::ApiFailure(inner) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error fetching station status from the bike share API: {}", inner),
            ),
        };
        (status, message).into_response()
    }
}

impl From<reqwest::Error> for StatusError {
    fn from(err: reqwest::Error) -> Self {
        StatusError::ApiFailure(err)
    }
}