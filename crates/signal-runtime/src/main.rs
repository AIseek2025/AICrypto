#![allow(unreachable_code)]
use aicrypto_feature_engine::compute::compute_all_features;
use aicrypto_feature_engine::ohlcv::{OhlcvCandle, OhlcvSeries};
use aicrypto_foundation::config::AppConfig;
use aicrypto_foundation::observability;
use aicrypto_signal_runtime::market_state;
use aicrypto_signal_runtime::signal_engine::SignalEngine;
use aicrypto_signal_runtime::skill_registry::SkillRegistry;
use std::path::PathBuf;

#[cfg(feature = "server")]
use std::time::Duration;

fn find_project_root() -> PathBuf {
    let exe_dir = std::env::current_exe().unwrap_or_default();
    let mut dir = exe_dir.parent();
    while let Some(d) = dir {
        if d.join("skills").exists() && d.join("Cargo.toml").exists() {
            return d.to_path_buf();
        }
        dir = d.parent();
    }
    std::env::current_dir().unwrap_or_default()
}

fn generate_synthetic_series(symbol: &str, scenario: &str) -> OhlcvSeries {
    let mut series = OhlcvSeries::new(symbol, "1h");
    let base_price = match symbol {
        s if s.starts_with("BTC") => 65000.0,
        s if s.starts_with("ETH") => 3500.0,
        _ => 100.0,
    };

    let n = 100;
    for i in 0..n {
        let (open, high, low, close, volume) = match scenario {
            "bull" => {
                let trend = (i as f64) * 15.0;
                let noise = ((i * 7) as f64).sin() * 200.0;
                let o = base_price + trend + noise;
                let c = o + 50.0 + (i as f64).sin() * 30.0;
                let h = c + 100.0;
                let l = o - 80.0;
                let v = 1000.0 + (i as f64).sin().abs() * 500.0 + if i > 80 { 800.0 } else { 0.0 };
                (o, h, l, c, v)
            }
            "bear" => {
                let trend = -(i as f64) * 12.0;
                let noise = ((i * 13) as f64).cos() * 150.0;
                let o = base_price + trend + noise;
                let c = o - 60.0 - (i as f64).cos().abs() * 40.0;
                let h = o + 80.0;
                let l = c - 120.0;
                let v = 1200.0 + (i as f64).cos().abs() * 600.0 + if i > 85 { 1200.0 } else { 0.0 };
                (o, h, l, c, v)
            }
            "range" => {
                let noise = ((i as f64) * 0.3).sin() * 300.0;
                let o = base_price + noise;
                let c = o + (i as f64).sin() * 50.0;
                let h = o.max(c) + 60.0;
                let l = o.min(c) - 60.0;
                let v = 800.0 + (i as f64).sin().abs() * 200.0;
                (o, h, l, c, v)
            }
            _ => {
                let o = base_price + (i as f64).sin() * 100.0;
                let c = o + 20.0;
                let h = c + 50.0;
                let l = o - 50.0;
                let v = 1000.0;
                (o, h, l, c, v)
            }
        };

        series.candles.push(OhlcvCandle {
            time: 1700000000 + (i as i64) * 3600,
            open,
            high,
            low,
            close,
            volume,
            quote_volume: volume * close,
            trades: 1000 + i as i64,
        });
    }

    series
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init_tracing("signal-runtime");
    let config = AppConfig::from_env()?;

    tracing::info!(
        env = ?config.environment,
        "signal-runtime starting"
    );

    let project_root = find_project_root();
    let skills_dir = project_root.join("skills");

    tracing::info!(skills_dir = %skills_dir.display(), "loading skills from directory");

    let registry = SkillRegistry::load_from_dir(&skills_dir)?;
    tracing::info!(total_skills = registry.len(), "skill registry initialized");

    let engine = SignalEngine::new(registry);

    tracing::info!("running signal evaluation demo with synthetic data");

    let symbols = vec![
        ("BTCUSDT", "bull"),
        ("BTCUSDT", "bear"),
        ("ETHUSDT", "range"),
        ("ETHUSDT", "bull"),
    ];

    for (symbol, scenario) in &symbols {
        tracing::info!("--- Evaluating {} ({}) ---", symbol, scenario);

        let series = generate_synthetic_series(symbol, scenario);

        match compute_all_features(&series) {
            Some(fv) => {
                let market_state = market_state::classify_market_state(&fv.features);
                tracing::info!(
                    symbol = %fv.symbol,
                    market_state = %market_state,
                    "market state classified"
                );

                let top_features: Vec<String> = fv
                    .features
                    .iter()
                    .take(8)
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                tracing::info!(features = ?top_features, "sample features");

                let signals = engine.evaluate(&fv);
                if signals.is_empty() {
                    tracing::info!(symbol = %symbol, scenario = %scenario, "no signals emitted");
                } else {
                    for signal in &signals {
                        tracing::info!(
                            signal_id = %signal.signal_id,
                            signal_type = ?signal.signal_type,
                            symbol = %signal.symbol,
                            direction = ?signal.direction,
                            confidence = signal.confidence,
                            horizon = ?signal.horizon,
                            reason_codes = ?signal.reason_codes,
                            "signal emitted"
                        );
                    }
                }
            }
            None => {
                tracing::warn!(symbol = %symbol, "insufficient data for feature computation");
            }
        }
    }

    tracing::info!("signal-runtime demo complete");

    #[cfg(feature = "server")]
    {
        tracing::info!("entering idle mode (ctrl+c to stop)");
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            tracing::debug!("signal-runtime heartbeat");
        }
    }

    Ok(())
}
