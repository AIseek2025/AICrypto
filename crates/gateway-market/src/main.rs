use aicrypto_gateway_market::persistence::collector::MarketDataCollector;
use aicrypto_gateway_market::persistence::sync::ExchangeInfoSync;
use aicrypto_gateway_market::persistence::backfill::KlineBackfill;
use aicrypto_gateway_market::persistence::db::Db;
use aicrypto_gateway_market::binance::rest::BinanceRestClient;
use aicrypto_gateway_market::binance::ws::{WsMarketStream, WsMessage};
use aicrypto_gateway_market::binance::mapper;
use aicrypto_foundation::config::AppConfig;
use aicrypto_foundation::observability;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    observability::init_tracing("gateway-market");
    let config = AppConfig::from_env()?;

    tracing::info!(
        env = ?config.environment,
        testnet = config.binance.testnet,
        rest_url = %config.binance.rest_base_url,
        "gateway-market starting"
    );

    let rest = BinanceRestClient::new(&config.binance.rest_base_url);
    let db = Db::connect(&config.database.url).await?;
    db.run_schema().await?;

    let db = Arc::new(db);
    let rest = Arc::new(rest);

    tracing::info!("step 1: syncing exchangeInfo...");
    let sync = ExchangeInfoSync::new((*rest).clone(), (*db).clone());
    let symbol_count = sync.sync().await?;
    tracing::info!(symbol_count = symbol_count, "exchangeInfo synced");

    let symbols = get_active_symbols(&db).await?;
    tracing::info!(count = symbols.len(), "active symbols loaded");

    if !symbols.is_empty() {
        let backfill_symbols: Vec<String> = symbols.iter().take(50).cloned().collect();
        tracing::info!(
            count = backfill_symbols.len(),
            "step 2: backfilling klines for top symbols"
        );
        let backfill = KlineBackfill::new((*rest).clone(), (*db).clone());
        let intervals = vec!["1m", "5m", "15m", "1h", "4h", "1d"];
        match backfill.backfill_multiple(&backfill_symbols, &intervals, 30).await {
            Ok(count) => tracing::info!(total_klines = count, "kline backfill complete"),
            Err(e) => tracing::error!(error = %e, "kline backfill failed"),
        }
    }

    let (ws_tx, mut ws_rx) = mpsc::channel::<WsMessage>(10000);

    let ws_symbols: Vec<String> = symbols.iter().take(20).cloned().collect();
    let ws_subscriptions = build_ws_subscriptions(&ws_symbols);

    let ws_handle = tokio::spawn(async move {
        let ws_url = std::env::var("BINANCE_WS_URL")
            .unwrap_or_else(|_| "wss://stream.binancefuture.com/ws".to_string());
        let stream = WsMarketStream::new(&ws_url, ws_tx);
        if let Err(e) = stream.run(ws_subscriptions).await {
            tracing::error!(error = %e, "WebSocket stream failed");
        }
    });

    let collector_rest = (*rest).clone();
    let collector_db = (*db).clone();
    let collector_symbols = symbols.clone();
    let collector_handle = tokio::spawn(async move {
        let collector = MarketDataCollector::new(collector_rest, collector_db);
        collector.run_periodic_collection(&collector_symbols, 60).await;
    });

    let db_clone = (*db).clone();
    let ws_processor = tokio::spawn(async move {
        while let Some(msg) = ws_rx.recv().await {
            match msg {
                WsMessage::Kline(k) => {
                    if k.kline.is_closed {
                        let _event = mapper::ws_kline_to_event(&k);
                        tracing::debug!(
                            symbol = %k.symbol,
                            close = %k.kline.close,
                            "kline closed"
                        );
                        if let Err(e) = db_clone.insert_kline(
                            &k.symbol,
                            &k.kline.interval,
                            k.kline.start_time,
                            &k.kline.open,
                            &k.kline.high,
                            &k.kline.low,
                            &k.kline.close,
                            &k.kline.volume,
                            k.kline.close_time,
                            &k.kline.quote_volume,
                            k.kline.trades,
                            &"0".to_string(),
                        ).await {
                            tracing::debug!(error = %e, "failed to save ws kline");
                        }
                    }
                }
                WsMessage::MarkPrice(mp) => {
                    tracing::trace!(symbol = %mp.symbol, mark_price = %mp.mark_price, "mark price update");
                    let now = chrono::Utc::now().timestamp_millis();
                    if let Err(e) = db_clone.insert_mark_price(
                        &mp.symbol,
                        now,
                        &mp.mark_price,
                        &mp.index_price,
                        mp.funding_rate.as_deref(),
                        mp.next_funding_time,
                    ).await {
                        tracing::debug!(error = %e, "failed to save ws mark_price");
                    }
                }
                WsMessage::Ticker(t) => {
                    tracing::trace!(symbol = %t.symbol, last = %t.last_price, "ticker update");
                }
                WsMessage::AggTrade(t) => {
                    tracing::trace!(symbol = %t.symbol, price = %t.price, "agg trade");
                }
                WsMessage::Depth(d) => {
                    tracing::trace!(symbol = %d.symbol, "depth update");
                }
                WsMessage::Raw(text) => {
                    tracing::trace!(len = text.len(), "raw WS message");
                }
            }
        }
    });

    #[cfg(feature = "server")]
    {
        tracing::info!("gateway-market fully operational — press Ctrl+C to stop");
        tokio::signal::ctrl_c().await?;
        tracing::info!("gateway-market shutting down...");

        ws_handle.abort();
        collector_handle.abort();
        ws_processor.abort();
    }

    #[cfg(not(feature = "server"))]
    {
        tracing::info!("gateway-market demo complete — shutting down");
        ws_handle.abort();
        collector_handle.abort();
        ws_processor.abort();
    }

    Ok(())
}

async fn get_active_symbols(db: &Arc<Db>) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT symbol FROM symbols WHERE status = 'TRADING' OR status = 'trading' ORDER BY symbol"
    )
    .fetch_all(db.pool())
    .await?;

    Ok(rows)
}

fn build_ws_subscriptions(symbols: &[String]) -> Vec<String> {
    let mut subs = Vec::new();
    for symbol in symbols {
        subs.push(WsMarketStream::build_kline_stream(symbol, "1m"));
        subs.push(WsMarketStream::build_mark_price_stream(symbol));
    }
    subs
}
