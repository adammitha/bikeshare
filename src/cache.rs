use sqlx::{migrate, query, Result, SqlitePool};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::api::StatusApiData;

#[derive(Debug)]
pub struct Cache {
    db: SqlitePool,
}

impl Cache {
    pub async fn new() -> Result<Self> {
        let db = SqlitePool::connect(
            &std::env::var("DATABASE_URL").unwrap_or(String::from("sqlite:bikeshare.db")),
        )
        .await?;
        migrate!("./migrations").run(&db).await?;
        Ok(Self { db })
    }

    pub async fn update_cache(&self, data: &StatusApiData) -> Result<()> {
        let transaction = self.db.begin().await?;
        let timestamp = OffsetDateTime::now_utc().format(&Iso8601::DEFAULT).unwrap();
        for status in &data.result {
            let longitude = status.coordinates.unwrap().longitude;
            let latitude = status.coordinates.unwrap().latitude;
            let q = query!(
                r#"INSERT INTO cache
                (timestamp, name, longitude, latitude, total_slots, free_slots, avl_bikes, operative, style, is_estation)
                VALUES (?,?,?,?,?,?,?,?,?,?)"#,
                timestamp,
                status.name,
                longitude,
                latitude,
                status.total_slots,
                status.free_slots,
                status.avl_bikes,
                status.operative,
                status.style,
                status.is_estation,
            ).execute(&self.db).await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
