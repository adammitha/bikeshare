#![allow(dead_code)]
use std::ops::Sub;

use sublime_fuzzy::{FuzzySearch, Scoring};
use time::{ext::NumericalDuration, OffsetDateTime};

use crate::api::{StationStatus, StatusApiData};

#[derive(Debug)]
pub struct Cache {
    timestamp: OffsetDateTime,
    entries: Vec<StationStatus>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            timestamp: OffsetDateTime::UNIX_EPOCH,
            entries: Vec::new(),
        }
    }

    pub fn update_cache(&mut self, data: StatusApiData) {
        tracing::info!("Updating cache");
        self.timestamp = OffsetDateTime::now_utc();
        for status in data.result {
            self.entries.push(status);
        }
    }

    pub fn lookup(&self, name: Option<&str>) -> Result<Vec<&StationStatus>> {
        if self.is_expired() {
            return Err(CacheError::Expired);
        }
        Ok(self.entries
            .as_slice()
            .into_iter()
            .filter(|station| {
                if let Some(station_name) = name {
                    FuzzySearch::new(station_name, &station.name)
                    .case_insensitive()
                    .score_with(&Scoring::default())
                    .best_match()
                    .is_some()
                } else {
                    true
                }
            })
            .collect::<Vec<&StationStatus>>())
    }

    pub fn is_expired(&self) -> bool {
        let expiry_date = OffsetDateTime::now_utc()
            .sub(5.minutes());
        return expiry_date > self.timestamp;
    }

    fn clean_cache(&mut self) {
        if OffsetDateTime::now_utc() - self.timestamp > 5.minutes() {
            self.entries.clear()
        }
        todo!("Ship existing cache entries to DB and clear HashMap")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache entries have expired")]
    Expired,
}

pub type Result<T> = std::result::Result<T, CacheError>;
