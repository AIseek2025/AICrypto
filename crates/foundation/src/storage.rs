use anyhow::Result;

pub struct StoragePool {
    database_url: String,
}

impl StoragePool {
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
        }
    }

    pub async fn connect(&self) -> Result<sqlx::PgPool> {
        let pool = sqlx::PgPool::connect(&self.database_url).await?;
        Ok(pool)
    }
}
