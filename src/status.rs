use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::instrument;

use crate::api::StationStatus;
use crate::ServerState;

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
    let mut cache = state.cache.lock().await;
    if cache.is_expired() {
        cache.update_cache(state.api.fetch_data().await?);
    }
    let stations = match cache.lookup(query.name.as_deref()) {
        Ok(res) => res.into_iter().cloned().collect(),
        Err(_) => todo!("Shouldn't call lookup on an expired cache"),
    };
    Ok(Json(stations))
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
                format!(
                    "Error fetching station status from the bike share API: {}",
                    inner
                ),
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
