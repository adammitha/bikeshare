use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::instrument;

use crate::ServerState;
use crate::{api::StationStatus, cache::CacheError};

#[derive(Deserialize, Debug)]
pub struct StationQuery {
    name: Option<String>,
}

/// Queries the 3rd-party bikeshare API, filters the result based on the user's search string,
/// and returns a JSON array of matching stations
#[instrument]
pub async fn station_status(
    State(state): State<Arc<ServerState>>,
    query: Query<StationQuery>,
) -> Result<Json<Vec<StationStatus>>, StatusError> {
    let stations = state
        .cache
        .lock()
        .await
        .lookup(query.name.as_deref())
        .await?
        .into_iter()
        .collect();
    Ok(Json(stations))
}

/// Errors that can occur when retrieving the status of a bike station
#[derive(Debug, thiserror::Error)]
pub enum StatusError {
    #[error("Error retrieving bikeshare data from the cache")]
    Cache(#[from] CacheError),
}

impl IntoResponse for StatusError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
