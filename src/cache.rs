use std::ops::Sub;

use sublime_fuzzy::{FuzzySearch, Scoring};
use time::{ext::NumericalDuration, OffsetDateTime};

use crate::{
    api::{BikeshareApi, StationStatus},
    db::Db,
};

#[derive(Debug)]
pub struct Cache {
    timestamp: OffsetDateTime,
    entries: Vec<StationStatus>,
    api: BikeshareApi,
    db: Option<Db>,
}

impl Cache {
    pub fn new(api: BikeshareApi, db: Option<Db>) -> Self {
        Self {
            timestamp: OffsetDateTime::UNIX_EPOCH,
            entries: Vec::new(),
            api,
            db,
        }
    }

    pub async fn refresh(&mut self) -> Result<(), CacheError> {
        if self.is_expired() {
            self.entries = self.api.fetch_data().await?.result;
            if let Some(db) = &self.db {
                db.archive_api_data(&self.entries).await?;
            }
            self.timestamp = OffsetDateTime::now_utc();
        }
        Ok(())
    }

    fn is_expired(&self) -> bool {
        let expiry_date = OffsetDateTime::now_utc().sub(5.minutes());
        return expiry_date > self.timestamp;
    }

    pub async fn lookup(&mut self, name: Option<&str>) -> Result<Vec<StationStatus>, CacheError> {
        self.refresh().await?;
        Ok(self
            .entries
            .iter()
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
            .cloned()
            .collect::<Vec<StationStatus>>())
    }

    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
