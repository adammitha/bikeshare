use std::ops::Sub;

use sublime_fuzzy::{FuzzySearch, Scoring};
use time::{ext::NumericalDuration, OffsetDateTime};

use crate::api::{BikeshareApi, StationStatus};

#[derive(Debug)]
pub struct Cache {
    timestamp: OffsetDateTime,
    entries: Vec<StationStatus>,
    api: BikeshareApi,
}

impl Cache {
    pub fn new(api: BikeshareApi) -> Self {
        Self {
            timestamp: OffsetDateTime::UNIX_EPOCH,
            entries: Vec::new(),
            api,
        }
    }

    pub async fn refresh(&mut self) -> Result<(), reqwest::Error> {
        if self.is_expired() {
            self.entries = self.api.fetch_data().await?.result;
            self.timestamp = OffsetDateTime::now_utc();
        }
        Ok(())
    }

    fn is_expired(&self) -> bool {
        let expiry_date = OffsetDateTime::now_utc().sub(5.minutes());
        return expiry_date > self.timestamp;
    }

    pub async fn lookup(
        &mut self,
        name: Option<&str>,
    ) -> Result<Vec<StationStatus>, reqwest::Error> {
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
}
