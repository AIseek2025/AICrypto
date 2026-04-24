use anyhow::{Context, Result};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct FeatureDb {
    pool: PgPool,
}

impl FeatureDb {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn get_active_symbols(&self) -> Result<Vec<String>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT symbol FROM symbols WHERE status IN ('TRADING','trading') ORDER BY symbol LIMIT 100"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn ensure_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS feature_vectors (
                id BIGSERIAL PRIMARY KEY,
                symbol VARCHAR(32) NOT NULL,
                feature_set VARCHAR(64) NOT NULL,
                feature_version VARCHAR(16) NOT NULL,
                window VARCHAR(16) NOT NULL,
                ts_feature TIMESTAMPTZ NOT NULL,
                features JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_fv_symbol_ts ON feature_vectors(symbol, ts_feature DESC);
            CREATE INDEX IF NOT EXISTS idx_fv_symbol_set_ts ON feature_vectors(symbol, feature_set, ts_feature DESC);
            "#,
        )
        .execute(&self.pool)
        .await
        .context("failed to create feature_vectors table")?;
        Ok(())
    }

    pub async fn store_feature_vector(
        &self,
        symbol: &str,
        feature_set: &str,
        feature_version: &str,
        window: &str,
        ts_feature: i64,
        features: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<()> {
        let ts = chrono::DateTime::from_timestamp_millis(ts_feature)
            .unwrap_or_default();
        sqlx::query(
            r#"
            INSERT INTO feature_vectors (symbol, feature_set, feature_version, window, ts_feature, features)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(symbol)
        .bind(feature_set)
        .bind(feature_version)
        .bind(window)
        .bind(ts)
        .bind(serde_json::Value::Object(features.clone()))
        .execute(&self.pool)
        .await
        .context("failed to store feature vector")?;
        Ok(())
    }

    pub async fn load_klines_for_features(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<i64>,
    ) -> Result<Vec<crate::ohlcv::OhlcvCandle>> {
        let lim = limit.unwrap_or(200);
        let rows = sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>, String, String, String, String, String, Option<String>, Option<i32>)>(
            r#"SELECT time, open, high, low, close, volume, quote_volume, trades
               FROM klines WHERE symbol = $1 AND interval = $2
               ORDER BY time DESC LIMIT $3"#,
        )
        .bind(symbol)
        .bind(interval)
        .bind(lim as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut candles: Vec<crate::ohlcv::OhlcvCandle> = rows.into_iter().map(|(time, open, high, low, close, volume, qv, trades)| {
            crate::ohlcv::OhlcvCandle {
                time: time.timestamp_millis(),
                open: open.parse().unwrap_or(0.0),
                high: high.parse().unwrap_or(0.0),
                low: low.parse().unwrap_or(0.0),
                close: close.parse().unwrap_or(0.0),
                volume: volume.parse().unwrap_or(0.0),
                quote_volume: qv.unwrap_or_default().parse().unwrap_or(0.0),
                trades: trades.unwrap_or(0) as i64,
            }
        }).collect();

        candles.reverse();
        Ok(candles)
    }
}
