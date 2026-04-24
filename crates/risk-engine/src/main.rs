use aicrypto_foundation::config::AppConfig;
use aicrypto_foundation::observability;
use aicrypto_risk_engine::evaluator::RiskEvaluator;
use aicrypto_risk_engine::rules::{RiskState, RuleConfig};
use aicrypto_protocols::order_intent::*;
use aicrypto_protocols::risk_decision::RiskVerdict;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init_tracing("risk-engine");
    let config = AppConfig::from_env()?;

    tracing::info!(
        env = ?config.environment,
        "risk-engine starting"
    );

    let rule_config = RuleConfig::default();
    let state = RiskState {
        total_exposure: 50000.0,
        open_positions: vec![("BTCUSDT".to_string(), 50000.0)].into_iter().collect(),
        daily_pnl: -500.0,
        equity: 100000.0,
        open_orders: 2,
        last_loss_time: None,
        current_time: 1000,
    };

    let mut evaluator = RiskEvaluator::new(rule_config).with_state(state);

    tracing::info!("risk-engine initialized with {} rules", 7);

    let intents = vec![
        ("Normal entry", make_intent("BTCUSDT", Side::BUY, "0.01", Some("65000"), Some(3), false)),
        ("Excessive leverage", make_intent("ETHUSDT", Side::BUY, "1.0", Some("3500"), Some(10), false)),
        ("Reduce order", make_intent("BTCUSDT", Side::SELL, "0.005", Some("66000"), None, true)),
        ("Huge position", make_intent("SOLUSDT", Side::BUY, "100.0", Some("150"), Some(3), false)),
    ];

    for (desc, intent) in &intents {
        tracing::info!("--- Evaluating: {} ---", desc);
        let decision = evaluator.evaluate(intent);

        tracing::info!(
            intent_id = %intent.intent_id,
            decision = ?decision.decision,
            severity = ?decision.severity,
            rule_hits = decision.rule_hits.len(),
            review_required = decision.review_required,
            "risk decision"
        );

        for hit in &decision.rule_hits {
            tracing::warn!(
                rule_id = %hit.rule_id,
                rule_name = %hit.rule_name,
                detail = %hit.detail,
                "rule hit"
            );
        }
    }

    tracing::info!("risk-engine demo complete");

    #[cfg(feature = "server")]
    {
        tracing::info!("entering idle mode (ctrl+c to stop)");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            tracing::debug!("risk-engine heartbeat");
        }
    }

    Ok(())
}

fn make_intent(
    symbol: &str,
    side: Side,
    quantity: &str,
    price: Option<&str>,
    leverage: Option<u32>,
    reduce_only: bool,
) -> OrderIntent {
    OrderIntent {
        intent_id: uuid::Uuid::new_v4().to_string(),
        account_scope: "main".to_string(),
        symbol: symbol.to_string(),
        side,
        position_side: PositionSide::LONG,
        order_type: if price.is_some() { OrderType::LIMIT } else { OrderType::MARKET },
        quantity: quantity.to_string(),
        price_limit: price.map(|p| p.to_string()),
        reduce_only,
        leverage_hint: leverage,
        take_profit_hint: None,
        stop_loss_hint: None,
        time_in_force: TimeInForce::GTC,
        origin_ref: "demo".to_string(),
        ts_intent: 1700000000,
    }
}
