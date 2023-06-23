use sqlx::{postgres::PgPoolOptions, query, Result};
use time::OffsetDateTime;

use crate::api::StationStatus;

#[derive(Debug, Clone)]
pub struct Db {
    inner: sqlx::PgPool,
}

impl Db {
    pub async fn new(url: &str) -> Result<Self> {
        Ok(Self {
            inner: PgPoolOptions::new().connect(url).await?,
        })
    }

    pub async fn archive_api_data(&self, api_data: &[StationStatus]) -> Result<()> {
        let transaction = self.inner.begin().await?;
        let timestamp = OffsetDateTime::now_utc();
        for row in api_data
            .iter()
            .map(|status| BikeshareData::from_station_status(status, timestamp))
        {
            query!(
                r#"INSERT INTO
        cache(time, name, longitude, latitude, total_slots,
        free_slots, avl_bikes, operative, style, is_estation)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10);"#,
                row.time,
                row.name,
                row.latitude,
                row.longitude,
                row.total_slots,
                row.free_slots,
                row.avl_bikes,
                row.operative,
                row.style,
                row.is_estation
            )
            .execute(&self.inner)
            .await?;
        }
        transaction.commit().await
    }
}

#[derive(sqlx::FromRow)]
pub struct BikeshareData {
    time: OffsetDateTime,
    name: String,
    latitude: Option<f32>,
    longitude: Option<f32>,
    total_slots: i32,
    free_slots: i32,
    avl_bikes: i32,
    operative: bool,
    style: String,
    is_estation: bool,
}

impl BikeshareData {
    pub fn from_station_status(status: &StationStatus, time: OffsetDateTime) -> Self {
        Self {
            time,
            name: status.name.clone(),
            latitude: status.coordinates.map(|c| c.latitude),
            longitude: status.coordinates.map(|c| c.longitude),
            total_slots: status.total_slots.into(),
            free_slots: status.free_slots.into(),
            avl_bikes: status.avl_bikes.into(),
            operative: status.operative,
            style: status.style.clone(),
            is_estation: status.is_estation,
        }
    }
}
