use std::io::Write;

use sqlx::{postgres::PgPoolOptions, Result};
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
        let copy_data = Self::make_copy_data(api_data).unwrap();
        let mut conn = self.inner.acquire().await.unwrap();
        let mut copy_in = conn
            .copy_in_raw("COPY cache FROM STDIN (DELIMITER '|')")
            .await
            .unwrap();
        copy_in.send(copy_data).await.unwrap();
        copy_in.finish().await.unwrap();
        Ok(())
    }

    fn make_copy_data(api_data: &[StationStatus]) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        let timestamp = OffsetDateTime::now_utc();
        let lines = api_data.iter().map(|status| {
            BikeshareData::from_station_status(status, timestamp).to_text_row()
        });
        for line in lines {
            writeln!(&mut buf, "{}", line)?;
        }
        Ok(buf)
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

    fn to_text_row(&self) -> String {
        fn opt_f32_to_str(f: Option<f32>) -> String {
            match f {
                Some(n) => format!("{}", n),
                None => "".into(),
            }
        }
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.time,
            self.name,
            opt_f32_to_str(self.latitude),
            opt_f32_to_str(self.longitude),
            self.total_slots,
            self.free_slots,
            self.avl_bikes,
            self.operative,
            self.style,
            self.is_estation
        )
    }
}
