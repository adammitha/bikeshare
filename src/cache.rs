use std::ops::Sub;

use sublime_fuzzy::{FuzzySearch, Scoring};
use time::{ext::NumericalDuration, OffsetDateTime};

use crate::api::{BikeshareApi, StationStatus};

#[derive(Debug, Clone)]
pub struct Cache<S: CacheState + Clone> {
    timestamp: OffsetDateTime,
    entries: Vec<StationStatus>,
    marker: std::marker::PhantomData<S>,
}

impl Cache<Stale> {
    pub fn new() -> Self {
        Self {
            timestamp: OffsetDateTime::UNIX_EPOCH,
            entries: Vec::new(),
            marker: Default::default(),
        }
    }

    pub async fn refresh(&mut self, api: &BikeshareApi) -> Result<Cache<Fresh>, reqwest::Error> {
        let fresh_cache = if self.is_expired() {
            let mut entries = Vec::new();
            for status in api.fetch_data().await?.result {
                entries.push(status);
            }
            Cache::<Fresh>::new(entries)
        } else {
            Cache::<Fresh>::with_timestamp(self.timestamp, std::mem::take(&mut self.entries))
        };
        *self = From::from(fresh_cache.clone());
        Ok(fresh_cache)
    }

    fn is_expired(&self) -> bool {
        let expiry_date = OffsetDateTime::now_utc().sub(5.minutes());
        return expiry_date > self.timestamp;
    }
}

/// Stores the result of the bikeshare API call for 5 minutes.
impl Cache<Fresh> {
    fn new(entries: Vec<StationStatus>) -> Self {
        Self {
            timestamp: OffsetDateTime::now_utc(),
            entries,
            marker: Default::default(),
        }
    }

    fn with_timestamp(timestamp: OffsetDateTime, entries: Vec<StationStatus>) -> Self {
        Self {
            timestamp,
            entries,
            marker: Default::default(),
        }
    }

    pub fn lookup(self, name: Option<&str>) -> Vec<StationStatus> {
        self.entries
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
            .collect::<Vec<StationStatus>>()
    }
}

#[derive(Debug, Clone)]
pub enum Stale {}
#[derive(Debug, Clone)]
pub enum Fresh {}

pub trait CacheState: std::fmt::Debug + Clone {}
impl CacheState for Stale {}
impl CacheState for Fresh {}

impl From<Cache<Fresh>> for Cache<Stale> {
    fn from(value: Cache<Fresh>) -> Cache<Stale> {
        Cache::<Stale> {
            timestamp: value.timestamp,
            entries: value.entries,
            marker: Default::default(),
        }
    }
}
