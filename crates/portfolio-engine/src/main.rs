#![allow(unreachable_code)]
use aicrypto_foundation::config::AppConfig;
use aicrypto_foundation::observability;
use aicrypto_portfolio_engine::portfolio::PortfolioManager;
use aicrypto_protocols::signal_event::{Direction, Horizon, SignalEvent, SignalType};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init_tracing("portfolio-engine");
    let config = AppConfig::from_env()?;

    tracing::info!(
        env = ?config.environment,
        "portfolio-engine starting"
    );

    let mut pm = PortfolioManager::new(100000.0, "main")
        .with_max_risk(0.02)
        .with_max_exposure(0.80)
        .with_max_positions(5);

    tracing::info!(
        equity = pm.equity(),
        "portfolio manager initialized"
    );

    let signals = vec![
        make_signal("sig-1", SignalType::Entry, Direction::LONG, "BTCUSDT", 0.85),
        make_signal("sig-2", SignalType::Entry, Direction::SHORT, "ETHUSDT", 0.75),
        make_signal("sig-3", SignalType::Entry, Direction::LONG, "SOLUSDT", 0.65),
        make_signal("sig-4", SignalType::Reduce, Direction::NEUTRAL, "BTCUSDT", 0.5),
        make_signal("sig-5", SignalType::Exit, Direction::LONG, "ETHUSDT", 0.7),
    ];

    let prices = json!({
        "BTCUSDT": 65000.0,
        "ETHUSDT": 3500.0,
        "SOLUSDT": 150.0,
    });

    for signal in &signals {
        let price = prices[&signal.symbol].as_f64().unwrap_or(100.0);
        tracing::info!("--- Processing signal {} ---", signal.signal_id);

        match pm.process_signal(signal, price) {
            Some(intent) => {
                tracing::info!(
                    intent_id = %intent.intent_id,
                    symbol = %intent.symbol,
                    side = ?intent.side,
                    order_type = ?intent.order_type,
                    quantity = %intent.quantity,
                    reduce_only = intent.reduce_only,
                    "generated OrderIntent"
                );
            }
            None => {
                tracing::warn!(
                    signal_id = %signal.signal_id,
                    symbol = %signal.symbol,
                    "signal rejected by portfolio manager"
                );
            }
        }
    }

    let tracker = pm.tracker();
    tracing::info!("--- Portfolio Summary ---");
    tracing::info!(open_positions = tracker.open_position_count());
    tracing::info!(total_exposure = tracker.total_exposure());
    tracing::info!(unrealized_pnl = tracker.total_unrealized_pnl());
    tracing::info!(realized_pnl = tracker.total_realized_pnl());

    for pos in tracker.all_positions() {
        tracing::info!(
            symbol = %pos.symbol,
            side = ?pos.side,
            quantity = pos.quantity,
            entry = pos.entry_price,
            leverage = pos.leverage,
            "open position"
        );
    }

    tracing::info!("portfolio-engine demo complete");

    #[cfg(feature = "server")]
    {
        tracing::info!("entering idle mode (ctrl+c to stop)");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            tracing::debug!("portfolio-engine heartbeat");
        }
    }

    Ok(())
}

fn make_signal(
    id: &str,
    signal_type: SignalType,
    direction: Direction,
    symbol: &str,
    confidence: f64,
) -> SignalEvent {
    SignalEvent {
        signal_id: id.to_string(),
        signal_type,
        symbol: symbol.to_string(),
        direction,
        confidence,
        horizon: Horizon::Swing,
        reason_codes: vec!["demo".to_string()],
        evidence_refs: vec![],
        ts_signal: 1700000000,
    }
}
