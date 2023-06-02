use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use tracing::instrument;

use crate::api::{BikeshareApi, StationStatus};

#[derive(Deserialize, Debug)]
pub struct StationQuery {
    name: Option<String>,
}

/// Queries the 3rd-party bikeshare API, filters the result based on the user's search string,
/// and returns a JSON array of matching stations
#[instrument]
pub async fn station_status(
    query: Query<StationQuery>,
) -> Result<Json<Vec<StationStatus>>, StatusError> {
    let response = BikeshareApi::new().fetch_data().await?;
    let stations = match &query.name {
        Some(name) => response.filter_stations(&name),
        None => response.result,
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
