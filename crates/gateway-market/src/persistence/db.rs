use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing;

#[derive(Debug, Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .context("failed to connect to database")?;
        tracing::info!("database connected");
        Ok(Self { pool })
    }

    pub async fn run_schema(&self) -> Result<()> {
        let schema = include_str!("../../../../scripts/bootstrap/001_init_schema.sql");
        for statement in schema.split(';') {
            let stmt = statement.trim();
            if stmt.is_empty() || stmt.starts_with("--") {
                continue;
            }
            if let Err(e) = sqlx::query(stmt).execute(&self.pool).await {
                let err_str = e.to_string();
                if err_str.contains("already exists") {
                    tracing::debug!("schema object already exists, skipping");
                } else {
                    tracing::warn!(error = %e, "schema exec warning");
                }
            }
        }
        tracing::info!("database schema initialized");
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn upsert_symbol(
        &self,
        symbol: &str,
        contract_type: Option<&str>,
        underlying: Option<&str>,
        quote_asset: Option<&str>,
        margin_asset: Option<&str>,
        status: &str,
        onboard_date: Option<chrono::DateTime<chrono::Utc>>,
        price_precision: i32,
        quantity_precision: i32,
        tick_size: &str,
        step_size: &str,
        min_qty: Option<&str>,
        min_notional: Option<&str>,
        max_leverage: Option<i32>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO symbols (symbol, contract_type, underlying, quote_asset, margin_asset, status,
                onboard_date, price_precision, quantity_precision, tick_size, step_size,
                min_qty, min_notional, max_leverage, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::DECIMAL, $11::DECIMAL,
                $12::DECIMAL, $13::DECIMAL, $14, NOW())
            ON CONFLICT (symbol) DO UPDATE SET
                contract_type = EXCLUDED.contract_type,
                underlying = EXCLUDED.underlying,
                quote_asset = EXCLUDED.quote_asset,
                margin_asset = EXCLUDED.margin_asset,
                status = EXCLUDED.status,
                onboard_date = EXCLUDED.onboard_date,
                price_precision = EXCLUDED.price_precision,
                quantity_precision = EXCLUDED.quantity_precision,
                tick_size = EXCLUDED.tick_size,
                step_size = EXCLUDED.step_size,
                min_qty = EXCLUDED.min_qty,
                min_notional = EXCLUDED.min_notional,
                max_leverage = EXCLUDED.max_leverage,
                updated_at = NOW()
            "#,
        )
        .bind(symbol)
        .bind(contract_type)
        .bind(underlying)
        .bind(quote_asset)
        .bind(margin_asset)
        .bind(status)
        .bind(onboard_date)
        .bind(price_precision)
        .bind(quantity_precision)
        .bind(tick_size)
        .bind(step_size)
        .bind(min_qty)
        .bind(min_notional)
        .bind(max_leverage)
        .execute(&self.pool)
        .await
        .context("failed to upsert symbol")?;
        Ok(())
    }

    pub async fn insert_kline(
        &self,
        symbol: &str,
        interval: &str,
        open_time: i64,
        open: &str,
        high: &str,
        low: &str,
        close: &str,
        volume: &str,
        close_time: i64,
        quote_volume: &str,
        trades: i64,
        taker_buy_volume: &str,
    ) -> Result<()> {
        let open_time_dt = chrono::DateTime::from_timestamp_millis(open_time)
            .unwrap_or_default();
        let close_time_dt = chrono::DateTime::from_timestamp_millis(close_time);

        sqlx::query(
            r#"
            INSERT INTO klines (time, symbol, interval, open, high, low, close, volume,
                close_time, quote_volume, trades, taker_buy_volume)
            VALUES ($1, $2, $3, $4::DECIMAL, $5::DECIMAL, $6::DECIMAL, $7::DECIMAL, $8::DECIMAL,
                $9, $10::DECIMAL, $11, $12::DECIMAL)
            ON CONFLICT (symbol, interval, time) DO UPDATE SET
                open = EXCLUDED.open, high = EXCLUDED.high, low = EXCLUDED.low,
                close = EXCLUDED.close, volume = EXCLUDED.volume,
                close_time = EXCLUDED.close_time, quote_volume = EXCLUDED.quote_volume,
                trades = EXCLUDED.trades, taker_buy_volume = EXCLUDED.taker_buy_volume
            "#,
        )
        .bind(open_time_dt)
        .bind(symbol)
        .bind(interval)
        .bind(open)
        .bind(high)
        .bind(low)
        .bind(close)
        .bind(volume)
        .bind(close_time_dt)
        .bind(quote_volume)
        .bind(trades as i32)
        .bind(taker_buy_volume)
        .execute(&self.pool)
        .await
        .context("failed to insert kline")?;
        Ok(())
    }

    pub async fn insert_klines_batch(
        &self,
        symbol: &str,
        interval: &str,
        klines: &[(i64, &str, &str, &str, &str, &str, i64, &str, i64, &str)],
    ) -> Result<u64> {
        let mut count = 0u64;
        for k in klines {
            self.insert_kline(
                symbol, interval,
                k.0, k.1, k.2, k.3, k.4, k.5,
                k.6, k.7, k.8, k.9,
            )
            .await?;
            count += 1;
        }
        Ok(count)
    }

    pub async fn insert_mark_price(
        &self,
        symbol: &str,
        time: i64,
        mark_price: &str,
        index_price: &str,
        funding_rate: Option<&str>,
        next_funding_time: Option<i64>,
    ) -> Result<()> {
        let time_dt = chrono::DateTime::from_timestamp_millis(time)
            .unwrap_or_default();
        let next_funding_dt = next_funding_time
            .and_then(|t| chrono::DateTime::from_timestamp_millis(t));

        sqlx::query(
            r#"
            INSERT INTO mark_prices (time, symbol, mark_price, index_price, funding_rate, next_funding_time)
            VALUES ($1, $2, $3::DECIMAL, $4::DECIMAL, $5, $6)
            ON CONFLICT (symbol, time) DO UPDATE SET
                mark_price = EXCLUDED.mark_price,
                index_price = EXCLUDED.index_price,
                funding_rate = EXCLUDED.funding_rate,
                next_funding_time = EXCLUDED.next_funding_time
            "#,
        )
        .bind(time_dt)
        .bind(symbol)
        .bind(mark_price)
        .bind(index_price)
        .bind(funding_rate)
        .bind(next_funding_dt)
        .execute(&self.pool)
        .await
        .context("failed to insert mark_price")?;
        Ok(())
    }

    pub async fn insert_funding_rate(
        &self,
        symbol: &str,
        time: i64,
        funding_rate: &str,
        funding_time: i64,
        mark_price: Option<&str>,
    ) -> Result<()> {
        let time_dt = chrono::DateTime::from_timestamp_millis(time)
            .unwrap_or_default();
        let funding_time_dt = chrono::DateTime::from_timestamp_millis(funding_time)
            .unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO funding_rates (time, symbol, funding_rate, funding_time, mark_price)
            VALUES ($1, $2, $3::DECIMAL, $4, $5::DECIMAL)
            ON CONFLICT (symbol, time) DO UPDATE SET
                funding_rate = EXCLUDED.funding_rate,
                funding_time = EXCLUDED.funding_time,
                mark_price = EXCLUDED.mark_price
            "#,
        )
        .bind(time_dt)
        .bind(symbol)
        .bind(funding_rate)
        .bind(funding_time_dt)
        .bind(mark_price)
        .execute(&self.pool)
        .await
        .context("failed to insert funding_rate")?;
        Ok(())
    }

    pub async fn insert_open_interest(
        &self,
        symbol: &str,
        time: i64,
        open_interest: &str,
    ) -> Result<()> {
        let time_dt = chrono::DateTime::from_timestamp_millis(time)
            .unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO open_interests (time, symbol, open_interest)
            VALUES ($1, $2, $3::DECIMAL)
            ON CONFLICT (symbol, time) DO UPDATE SET
                open_interest = EXCLUDED.open_interest
            "#,
        )
        .bind(time_dt)
        .bind(symbol)
        .bind(open_interest)
        .execute(&self.pool)
        .await
        .context("failed to insert open_interest")?;
        Ok(())
    }
}
