use aicrypto_feature_engine::ohlcv::{OhlcvCandle, OhlcvSeries};
use aicrypto_foundation::config::AppConfig;
use aicrypto_foundation::observability;
use aicrypto_pipeline_integration::pipeline::Pipeline;
use aicrypto_protocols::signal_event::{Direction, SignalType};
use std::path::PathBuf;

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

fn generate_bull_series(symbol: &str) -> OhlcvSeries {
    let mut series = OhlcvSeries::new(symbol, "1h");
    let base_price = match symbol {
        s if s.starts_with("BTC") => 65000.0,
        s if s.starts_with("ETH") => 3500.0,
        s if s.starts_with("SOL") => 150.0,
        _ => 100.0,
    };

    for i in 0..120 {
        let trend = (i as f64) * 15.0;
        let noise = ((i * 7) as f64).sin() * 200.0;
        let o = base_price + trend + noise;
        let c = o + 50.0 + (i as f64).sin() * 30.0;
        let h = c + 100.0;
        let l = o - 80.0;
        let v = 1000.0 + (i as f64).sin().abs() * 500.0 + if i > 100 { 1500.0 } else { 0.0 };

        series.candles.push(OhlcvCandle {
            time: 1700000000 + (i as i64) * 3600,
            open: o,
            high: h,
            low: l,
            close: c,
            volume: v,
            quote_volume: v * c,
            trades: 1000 + i as i64,
        });
    }
    series
}

fn generate_bear_series(symbol: &str) -> OhlcvSeries {
    let mut series = OhlcvSeries::new(symbol, "1h");
    let base_price = match symbol {
        s if s.starts_with("BTC") => 65000.0,
        s if s.starts_with("ETH") => 3500.0,
        _ => 100.0,
    };

    for i in 0..120 {
        let trend = -(i as f64) * 12.0;
        let noise = ((i * 13) as f64).cos() * 150.0;
        let o = base_price + trend + noise;
        let c = o - 60.0 - (i as f64).cos().abs() * 40.0;
        let h = o + 80.0;
        let l = c - 120.0;
        let v = 1200.0 + (i as f64).cos().abs() * 600.0 + if i > 100 { 2000.0 } else { 0.0 };

        series.candles.push(OhlcvCandle {
            time: 1700000000 + (i as i64) * 3600,
            open: o,
            high: h,
            low: l,
            close: c,
            volume: v,
            quote_volume: v * c,
            trades: 1000 + i as i64,
        });
    }
    series
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init_tracing("pipeline-integration");
    let config = AppConfig::from_env()?;

    tracing::info!(env = ?config.environment, "pipeline-integration starting");

    let project_root = find_project_root();
    let skills_dir = project_root.join("skills");

    let mut pipeline = Pipeline::new(&skills_dir, 100000.0)?;

    tracing::info!("========================================");
    tracing::info!("  End-to-End Pipeline Integration Demo");
    tracing::info!("========================================");

    let scenarios = vec![
        ("BTC Bull Trend", generate_bull_series("BTCUSDT"), 68000.0),
        ("ETH Bull Trend", generate_bull_series("ETHUSDT"), 3700.0),
        ("BTC Bear Trend", generate_bear_series("BTCUSDT"), 58000.0),
        ("SOL Bull Trend", generate_bull_series("SOLUSDT"), 170.0),
    ];

    let mut total_signals = 0;
    let mut total_intents = 0;
    let mut total_executed = 0;
    let mut total_rejected_risk = 0;

    for (name, series, price) in &scenarios {
        tracing::info!("");
        tracing::info!(">>> Scenario: {} | candles={} | price={}", name, series.candles.len(), price);

        let result = pipeline.process_candles(&series, *price)?;

        total_signals += result.signals.len();
        total_intents += result.intents.len();
        total_executed += result.reports.iter().filter(|r| {
                            matches!(r.order_status, aicrypto_protocols::execution_report::OrderStatus::Filled)
                          }).count();
        total_rejected_risk += result.decisions.iter().filter(|d| {
            matches!(d.decision, aicrypto_protocols::risk_decision::RiskVerdict::Deny)
        }).count();

        for signal in &result.signals {
            tracing::info!(
                "  Signal: {} {} {} conf={:.0}% horizon={:?}",
                signal.signal_id.chars().take(8).collect::<String>(),
                signal.signal_type.as_ref(),
                signal.direction.as_ref(),
                signal.confidence * 100.0,
                signal.horizon,
            );
        }
    }

    let pm = pipeline.portfolio_manager();
    tracing::info!("");
    tracing::info!("========================================");
    tracing::info!("  Pipeline Summary");
    tracing::info!("========================================");
    tracing::info!(total_signals = total_signals);
    tracing::info!(total_intents = total_intents);
    tracing::info!(total_executed = total_executed);
    tracing::info!(total_rejected_by_risk = total_rejected_risk);
    tracing::info!(open_positions = pm.tracker().open_position_count());
    tracing::info!(total_exposure = pm.tracker().total_exposure());
    tracing::info!(unrealized_pnl = pm.tracker().total_unrealized_pnl());
    tracing::info!(realized_pnl = pm.tracker().total_realized_pnl());

    for pos in pm.tracker().all_positions() {
        tracing::info!(
            "  Position: {} {:?} qty={} entry={:.2} leverage={}x",
            pos.symbol,
            pos.side,
            pos.quantity,
            pos.entry_price,
            pos.leverage,
        );
    }

    tracing::info!("");
    tracing::info!("pipeline-integration demo complete");
    Ok(())
}

trait AsRefStr {
    fn as_ref(&self) -> &'static str;
}
impl AsRefStr for SignalType {
    fn as_ref(&self) -> &'static str {
        match self {
            SignalType::Entry => "ENTRY",
            SignalType::Exit => "EXIT",
            SignalType::Add => "ADD",
            SignalType::Reduce => "REDUCE",
            SignalType::RiskAlert => "RISK_ALERT",
        }
    }
}
impl AsRefStr for Direction {
    fn as_ref(&self) -> &'static str {
        match self {
            Direction::LONG => "LONG",
            Direction::SHORT => "SHORT",
            Direction::NEUTRAL => "NEUTRAL",
        }
    }
}
