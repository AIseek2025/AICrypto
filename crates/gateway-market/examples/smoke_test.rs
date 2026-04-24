use aicrypto_foundation::observability;
use aicrypto_gateway_market::binance::rest::BinanceRestClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init_tracing("smoke-test");

    let rest = BinanceRestClient::new("https://testnet.binancefuture.com");

    println!("=== Smoke Test: Binance Futures Testnet ===\n");

    println!("1. Fetching exchangeInfo...");
    match rest.get_exchange_info().await {
        Ok(info) => {
            let trading: Vec<_> = info.symbols.iter()
                .filter(|s| s.is_trading())
                .collect();
            println!("   Total symbols: {}", info.symbols.len());
            println!("   Trading symbols: {}", trading.len());
            if let Some(first) = trading.first() {
                println!("   Sample: {} (base={}, quote={})",
                    first.symbol,
                    first.base_asset.as_deref().unwrap_or("?"),
                    first.quote_asset.as_deref().unwrap_or("?")
                );
            }
        }
        Err(e) => println!("   FAILED: {}", e),
    }

    println!("\n2. Fetching BTCUSDT klines (1h, last 5)...");
    match rest.get_klines("BTCUSDT", "1h", None, None, Some(5)).await {
        Ok(klines) => {
            println!("   Fetched {} candles", klines.len());
            for k in &klines {
                let dt = chrono::DateTime::from_timestamp_millis(k.open_time);
                println!("   {} | O:{} H:{} L:{} C:{} V:{}",
                    dt.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default(),
                    k.open, k.high, k.low, k.close, k.volume
                );
            }
        }
        Err(e) => println!("   FAILED: {}", e),
    }

    println!("\n3. Fetching mark prices...");
    match rest.get_mark_price(Some("BTCUSDT")).await {
        Ok(prices) => {
            for mp in &prices {
                println!("   {} | mark={} index={} funding={:?}",
                    mp.symbol, mp.mark_price, mp.index_price, mp.last_funding_rate
                );
            }
        }
        Err(e) => println!("   FAILED: {}", e),
    }

    println!("\n4. Fetching BTCUSDT open interest...");
    match rest.get_open_interest("BTCUSDT").await {
        Ok(oi) => {
            println!("   OI: {}", oi.open_interest);
        }
        Err(e) => println!("   FAILED: {}", e),
    }

    println!("\n5. Fetching BTCUSDT 24hr ticker...");
    match rest.get_24hr_ticker(Some("BTCUSDT")).await {
        Ok(tickers) => {
            for t in &tickers {
                println!("   {} | last={} vol={} change={}%",
                    t.symbol, t.last_price, t.quote_volume, t.price_change_percent
                );
            }
        }
        Err(e) => println!("   FAILED: {}", e),
    }

    println!("\n6. Fetching BTCUSDT funding rate...");
    match rest.get_funding_rate("BTCUSDT", None, None, Some(5)).await {
        Ok(rates) => {
            for r in &rates {
                let dt = chrono::DateTime::from_timestamp_millis(r.funding_time);
                println!("   {} | rate={} time={}",
                    r.symbol, r.funding_rate,
                    dt.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default()
                );
            }
        }
        Err(e) => println!("   FAILED: {}", e),
    }

    println!("\n=== Smoke Test Complete ===");
    Ok(())
}
