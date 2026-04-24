use aicrypto_foundation::config::AppConfig;
use aicrypto_foundation::observability;
use aicrypto_feature_engine::compute::compute_all_features;
use aicrypto_feature_engine::db::FeatureDb;
use aicrypto_feature_engine::ohlcv::OhlcvSeries;
use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    observability::init_tracing("feature-engine");
    let config = AppConfig::from_env()?;

    tracing::info!(env = ?config.environment, "feature-engine starting");

    let db = Arc::new(FeatureDb::connect(&config.database.url).await?);
    db.ensure_schema().await?;

    let symbols = db.get_active_symbols().await?;
    tracing::info!(count = symbols.len(), "computing features for active symbols");

    let intervals = vec!["1h", "4h", "1d"];
    let mut computed = 0u64;

    for symbol in &symbols {
        for interval in &intervals {
            match db.load_klines_for_features(symbol, interval, Some(200)).await {
                Ok(candles) if candles.len() >= 50 => {
                    let mut series = OhlcvSeries::new(symbol, interval);
                    series.candles = candles;
                    if let Some(fv) = compute_all_features(&series) {
                        if let Err(e) = db.store_feature_vector(
                            &fv.symbol, &fv.feature_set, &fv.feature_version,
                            &fv.window, fv.ts_feature, &fv.features,
                        ).await {
                            tracing::warn!(symbol = %symbol, error = %e, "failed to store features");
                        } else {
                            computed += 1;
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::debug!(symbol = %symbol, error = %e, "failed to load klines");
                }
            }
        }
    }

    tracing::info!(computed = computed, "initial feature computation pass complete");

    #[cfg(feature = "server")]
    let db_bg = db.clone();
    #[cfg(feature = "server")]
    let symbols_bg = symbols.clone();
    #[cfg(feature = "server")]
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            let mut count = 0u64;
            for symbol in &symbols_bg {
                for interval in &["1h", "4h", "1d"] {
                    if let Ok(candles) = db_bg.load_klines_for_features(symbol, interval, Some(200)).await {
                        if candles.len() >= 50 {
                            let mut series = OhlcvSeries::new(symbol, interval);
                            series.candles = candles;
                            if let Some(fv) = compute_all_features(&series) {
                                if db_bg.store_feature_vector(
                                    &fv.symbol, &fv.feature_set, &fv.feature_version,
                                    &fv.window, fv.ts_feature, &fv.features,
                                ).await.is_ok() {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
            tracing::info!(computed = count, "periodic feature computation pass complete");
        }
    });

    #[cfg(feature = "server")]
    {
        tracing::info!("feature-engine fully operational — press Ctrl+C to stop");
        tokio::signal::ctrl_c().await?;
        tracing::info!("feature-engine shutting down");
    }

    #[cfg(not(feature = "server"))]
    tracing::info!("feature-engine demo complete");

    Ok(())
}
