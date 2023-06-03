use sqlx::{migrate, Result, SqlitePool};

#[derive(Debug)]
pub struct Cache {
    db: SqlitePool,
}

impl Cache {
    pub async fn new() -> Result<Self> {
        let db = SqlitePool::connect("sqlite:bikeshare.db").await?;
        migrate!("./migrations").run(&db).await?;
        Ok(Self { db })
    }
}
