use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::instrument;

use crate::ServerState;
use crate::{api::StationStatus, cache::CacheError};

#[derive(Deserialize, Debug)]
pub struct StationQuery {
    name: Option<String>,
    #[serde(default = "StationQuery::default_cached_value")]
    cached: bool,
}

impl StationQuery {
    fn default_cached_value() -> bool {
        true
    }
}

#[derive(Serialize, Debug)]
pub struct StationResponse {
    timestamp: OffsetDateTime,
    stations: Vec<StationStatus>,
}

/// Queries the 3rd-party bikeshare API, filters the result based on the user's search string,
/// and returns a JSON array of matching stations
#[instrument]
pub async fn station_status(
    State(state): State<Arc<ServerState>>,
    query: Query<StationQuery>,
) -> Result<Json<StationResponse>, StatusError> {
    let mut cache = state.cache.lock().await;
    if !query.cached {
        cache.invalidate();
    }
    let stations: Vec<StationStatus> = cache
        .lookup(query.name.as_deref())
        .await?
        .into_iter()
        .collect();
    let timestamp = cache.timestamp();
    Ok(Json(StationResponse {
        timestamp,
        stations,
    }))
}

/// Errors that can occur when retrieving the status of a bike station
#[derive(Debug, thiserror::Error)]
pub enum StatusError {
    #[error("Error retrieving bikeshare data from the cache: {0}")]
    Cache(#[from] CacheError),
}

impl IntoResponse for StatusError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
