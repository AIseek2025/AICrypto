use crate::binance::rest::BinanceRestClient;
use crate::persistence::db::Db;
use anyhow::Result;
use std::time::Duration;
use tracing;

pub struct MarketDataCollector {
    rest: BinanceRestClient,
    db: Db,
}

impl MarketDataCollector {
    pub fn new(rest: BinanceRestClient, db: Db) -> Self {
        Self { rest, db }
    }

    pub async fn collect_mark_prices(&self) -> Result<usize> {
        let mark_prices = self.rest.get_mark_price(None).await?;
        let now = chrono::Utc::now().timestamp_millis();
        let mut count = 0;

        for mp in &mark_prices {
            if let Err(e) = self.db.insert_mark_price(
                &mp.symbol,
                now,
                &mp.mark_price,
                &mp.index_price,
                mp.last_funding_rate.as_deref(),
                mp.next_funding_time,
            ).await {
                tracing::debug!(symbol = %mp.symbol, error = %e, "failed to save mark_price");
                continue;
            }
            count += 1;
        }

        tracing::info!(count = count, "mark prices collected");
        Ok(count)
    }

    pub async fn collect_open_interests(&self, symbols: &[String]) -> Result<usize> {
        let now = chrono::Utc::now().timestamp_millis();
        let mut count = 0;

        for symbol in symbols {
            match self.rest.get_open_interest(symbol).await {
                Ok(oi) => {
                    if let Err(e) = self.db.insert_open_interest(
                        symbol,
                        now,
                        &oi.open_interest,
                    ).await {
                        tracing::debug!(symbol = %symbol, error = %e, "failed to save OI");
                    } else {
                        count += 1;
                    }
                }
                Err(e) => {
                    tracing::debug!(symbol = %symbol, error = %e, "failed to fetch OI");
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        tracing::info!(count = count, total = symbols.len(), "open interests collected");
        Ok(count)
    }

    pub async fn collect_funding_history(
        &self,
        symbol: &str,
        start_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<usize> {
        let rates = self.rest.get_funding_rate(symbol, start_time, None, limit).await?;
        let mut count = 0;

        for fr in &rates {
            if let Err(e) = self.db.insert_funding_rate(
                &fr.symbol,
                fr.funding_time,
                &fr.funding_rate,
                fr.funding_time,
                fr.mark_price.as_deref(),
            ).await {
                tracing::debug!(symbol = %fr.symbol, error = %e, "failed to save funding_rate");
                continue;
            }
            count += 1;
        }

        tracing::info!(symbol = %symbol, count = count, "funding rates collected");
        Ok(count)
    }

    pub async fn run_periodic_collection(
        &self,
        symbols: &[String],
        interval_secs: u64,
    ) {
        let mut tick = 0u64;
        loop {
            tick += 1;
            tracing::info!(tick = tick, "starting periodic market data collection");

            if let Err(e) = self.collect_mark_prices().await {
                tracing::error!(error = %e, "mark price collection failed");
            }

            if tick % 8 == 0 {
                if let Err(e) = self.collect_open_interests(symbols).await {
                    tracing::error!(error = %e, "OI collection failed");
                }
            }

            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    }
}
