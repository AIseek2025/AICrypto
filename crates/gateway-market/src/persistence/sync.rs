use crate::binance::models::BinanceSymbolInfo;
use crate::binance::rest::BinanceRestClient;
use crate::persistence::db::Db;
use anyhow::Result;
use tracing;

pub struct ExchangeInfoSync {
    rest: BinanceRestClient,
    db: Db,
}

impl ExchangeInfoSync {
    pub fn new(rest: BinanceRestClient, db: Db) -> Self {
        Self { rest, db }
    }

    pub async fn sync(&self) -> Result<usize> {
        tracing::info!("starting exchangeInfo sync");
        let exchange_info = self.rest.get_exchange_info().await?;

        let mut count = 0usize;
        for sym in &exchange_info.symbols {
            if let Err(e) = self.sync_symbol(sym).await {
                tracing::warn!(symbol = %sym.symbol, error = %e, "failed to sync symbol");
                continue;
            }
            count += 1;
        }

        tracing::info!(total = exchange_info.symbols.len(), synced = count, "exchangeInfo sync complete");
        Ok(count)
    }

    async fn sync_symbol(&self, sym: &BinanceSymbolInfo) -> Result<()> {
        let status = sym.status.as_deref().unwrap_or("trading");
        let contract_type = sym.contract_type.as_deref();
        let onboard_date = sym.onboard_date
            .and_then(|ts| chrono::DateTime::from_timestamp_millis(ts));

        self.db.upsert_symbol(
            &sym.symbol,
            contract_type,
            sym.base_asset.as_deref(),
            sym.quote_asset.as_deref(),
            sym.margin_asset.as_deref(),
            status,
            onboard_date,
            sym.price_precision.unwrap_or(8),
            sym.quantity_precision.unwrap_or(8),
            &sym.tick_size().map(|v| v.to_string()).unwrap_or_else(|| "0.01".to_string()),
            &sym.step_size().map(|v| v.to_string()).unwrap_or_else(|| "0.001".to_string()),
            sym.min_qty().map(|v| v.to_string()).as_deref(),
            sym.min_notional().map(|v| v.to_string()).as_deref(),
            sym.max_leverage(),
        ).await
    }
}
