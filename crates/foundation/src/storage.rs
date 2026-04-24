use anyhow::Result;

pub struct StoragePool {
    database_url: String,
    max_connections: u32,
}

impl StoragePool {
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            max_connections: 10,
        }
    }

    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub async fn connect(&self) -> Result<sqlx::PgPool> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(self.max_connections)
            .connect(&self.database_url)
            .await?;
        Ok(pool)
    }
}
