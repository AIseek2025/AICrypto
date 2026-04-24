use crate::binance::rest::BinanceRestClient;
use crate::persistence::db::Db;
use anyhow::Result;
use std::time::Duration;
use tracing;

pub struct KlineBackfill {
    rest: BinanceRestClient,
    db: Db,
}

impl KlineBackfill {
    pub fn new(rest: BinanceRestClient, db: Db) -> Self {
        Self { rest, db }
    }

    pub async fn backfill_symbol(
        &self,
        symbol: &str,
        interval: &str,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<u64> {
        let candle_ms = Self::interval_to_ms(interval);
        let mut current = start_ms;
        let mut total = 0u64;

        while current < end_ms {
            let batch_end = (current + candle_ms * 1500).min(end_ms);
            let klines = self.rest
                .get_klines(symbol, interval, Some(current), Some(batch_end), Some(1500))
                .await?;

            if klines.is_empty() {
                break;
            }

            let batch_count = klines.len();
            for k in &klines {
                if let Err(e) = self.db.insert_kline(
                    symbol, interval,
                    k.open_time, &k.open, &k.high, &k.low, &k.close, &k.volume,
                    k.close_time, &k.quote_volume, k.trades, &k.taker_buy_volume,
                ).await {
                    tracing::warn!(symbol = %symbol, error = %e, "failed to insert kline");
                }
            }

            total += batch_count as u64;
            tracing::info!(
                symbol = %symbol, interval = %interval,
                progress = format!("{}/{}", total, (end_ms - start_ms) / candle_ms),
                batch = batch_count,
                "kline backfill progress"
            );

            if batch_count < 1500 {
                break;
            }
            current = klines.last().map(|k| k.close_time + 1).unwrap_or(batch_end + 1);
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        tracing::info!(
            symbol = %symbol, interval = %interval,
            total = total, "kline backfill complete"
        );
        Ok(total)
    }

    pub async fn backfill_multiple(
        &self,
        symbols: &[String],
        intervals: &[&str],
        days_back: u32,
    ) -> Result<u64> {
        let end_ms = chrono::Utc::now().timestamp_millis();
        let start_ms = end_ms - (days_back as i64 * 24 * 3600 * 1000);
        let mut total = 0u64;

        for symbol in symbols {
            for interval in intervals {
                match self.backfill_symbol(symbol, interval, start_ms, end_ms).await {
                    Ok(count) => total += count,
                    Err(e) => {
                        tracing::error!(symbol = %symbol, interval = %interval, error = %e, "backfill failed");
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        tracing::info!(total = total, "all backfill complete");
        Ok(total)
    }

    fn interval_to_ms(interval: &str) -> i64 {
        match interval {
            "1m" => 60_000,
            "3m" => 180_000,
            "5m" => 300_000,
            "15m" => 900_000,
            "30m" => 1_800_000,
            "1h" => 3_600_000,
            "2h" => 7_200_000,
            "4h" => 14_400_000,
            "6h" => 21_600_000,
            "8h" => 28_800_000,
            "12h" => 43_200_000,
            "1d" => 86_400_000,
            "3d" => 259_200_000,
            "1w" => 604_800_000,
            "1M" => 2_592_000_000,
            _ => 3_600_000,
        }
    }
}
