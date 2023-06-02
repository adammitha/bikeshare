use axum::{http::StatusCode, response::IntoResponse};
use reqwest::{Client, ClientBuilder};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use sublime_fuzzy::{FuzzySearch, Scoring};

use crate::API_URL;

#[derive(Debug)]
pub struct BikeshareApi {
    client: Client,
}

impl BikeshareApi {
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(5))
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

impl StatusApiData {
    pub fn filter_stations(&self, name: &str) -> Vec<StationStatus> {
        self.result
            .as_slice()
            .into_iter()
            .filter(|station| {
                FuzzySearch::new(name, &station.name)
                    .case_insensitive()
                    .score_with(&Scoring::default())
                    .best_match()
                    .is_some()
            })
            .cloned()
            .collect::<Vec<StationStatus>>()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StationStatus {
    name: String,
    #[serde(deserialize_with = "deserialize_coordinate")]
    coordinates: Option<Coordinate>,
    total_slots: u8,
    free_slots: u8,
    avl_bikes: u8,
    operative: bool,
    style: String,
    is_estation: bool,
}

#[derive(Serialize, Clone)]
struct Coordinate {
    latitude: f64,
    longitude: f64,
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
            let mut coords = v.split(',').map(|c| c.trim().parse::<f64>());

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
