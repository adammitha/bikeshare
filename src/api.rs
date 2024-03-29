use axum::{http::StatusCode, response::IntoResponse};
use reqwest::{Client, ClientBuilder};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

use crate::API_URL;

#[derive(Debug)]
pub struct BikeshareApi {
    client: Client,
}

impl BikeshareApi {
    pub fn new() -> Self {
        let timeout = std::env::var("BIKESHARE_TIMEOUT")
            .unwrap_or("5".into())
            .parse::<u64>()
            .unwrap_or(5);
        Self {
            client: ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(timeout))
                .build()
                .unwrap(),
        }
    }

    pub async fn fetch_data(&self) -> Result<StatusApiData, reqwest::Error> {
        Ok(self
            .client
            .get(API_URL)
            .send()
            .await?
            .json::<StatusApiData>()
            .await?)
    }
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

#[derive(Serialize, Deserialize)]
pub struct StatusApiData {
    pub result: Vec<StationStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StationStatus {
    pub name: String,
    #[serde(deserialize_with = "deserialize_coordinate")]
    pub coordinates: Option<Coordinate>,
    pub total_slots: u8,
    pub free_slots: u8,
    pub avl_bikes: u8,
    pub operative: bool,
    pub style: String,
    pub is_estation: bool,
}

#[derive(Serialize, Copy, Clone, Debug)]
pub struct Coordinate {
    pub latitude: f32,
    pub longitude: f32,
}

fn deserialize_coordinate<'de, D>(deserializer: D) -> Result<Option<Coordinate>, D::Error>
where
    D: Deserializer<'de>,
{
    struct CoordinateVisitor;

    impl<'de> Visitor<'de> for CoordinateVisitor {
        type Value = Option<Coordinate>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing coordinates")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut coords = v.split(',').map(|c| c.trim().parse::<f32>());

            match (coords.next(), coords.next()) {
                (Some(Ok(latitude)), Some(Ok(longitude))) => Ok(Some(Coordinate {
                    latitude,
                    longitude,
                })),
                _ => Ok(None),
            }
        }
    }
    deserializer.deserialize_any(CoordinateVisitor)
}
