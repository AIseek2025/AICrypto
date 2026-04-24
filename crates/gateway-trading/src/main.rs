#![allow(unreachable_code)]
use aicrypto_foundation::config::AppConfig;
use aicrypto_foundation::observability;
use aicrypto_gateway_trading::executor::TradeExecutor;
use aicrypto_protocols::order_intent::*;
use aicrypto_protocols::risk_decision::{RiskDecision, RiskVerdict, Severity};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init_tracing("gateway-trading");
    let config = AppConfig::from_env()?;

    tracing::info!(
        env = ?config.environment,
        testnet = config.binance.testnet,
        "gateway-trading starting"
    );

    let mut executor = TradeExecutor::new("binance_testnet", true);

    tracing::info!("gateway-trading initialized (dry-run mode)");

    let scenarios = vec![
        ("BTC LONG entry (approved)", make_intent("BTCUSDT", Side::BUY, "0.1", Some("65000")), make_decision(RiskVerdict::Allow)),
        ("ETH SHORT entry (denied)", make_intent("ETHUSDT", Side::SELL, "1.0", Some("3500")), make_decision(RiskVerdict::Deny)),
        ("SOL LONG entry (approved)", make_intent("SOLUSDT", Side::BUY, "10.0", Some("150")), make_decision(RiskVerdict::Allow)),
        ("BTC reduce (approved)", make_intent("BTCUSDT", Side::SELL, "0.05", None), make_decision(RiskVerdict::Allow)),
    ];

    for (desc, intent, risk) in &scenarios {
        tracing::info!("--- {} ---", desc);

        let report = executor.submit(intent, risk)?;

        tracing::info!(
            report_id = %report.report_id,
            intent_id = %report.intent_id,
            symbol = %report.symbol,
            status = ?report.order_status,
            filled_qty = ?report.filled_qty,
            avg_price = ?report.avg_fill_price,
            fees = ?report.fees,
            exchange_id = ?report.exchange_order_id,
            "execution report"
        );
    }

    let osm = executor.osm();
    tracing::info!("--- Trading Summary ---");
    tracing::info!(total_orders = osm.total_orders());
    tracing::info!(active_orders = osm.active_orders().len());

    tracing::info!("gateway-trading demo complete");

    #[cfg(feature = "server")]
    {
        tracing::info!("entering idle mode (ctrl+c to stop)");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            tracing::debug!("gateway-trading heartbeat");
        }
    }

    Ok(())
}

fn make_intent(symbol: &str, side: Side, quantity: &str, price: Option<&str>) -> OrderIntent {
    OrderIntent {
        intent_id: uuid::Uuid::new_v4().to_string(),
        account_scope: "main".to_string(),
        symbol: symbol.to_string(),
        side,
        position_side: PositionSide::LONG,
        order_type: if price.is_some() { OrderType::LIMIT } else { OrderType::MARKET },
        quantity: quantity.to_string(),
        price_limit: price.map(|p| p.to_string()),
        reduce_only: false,
        leverage_hint: Some(3),
        take_profit_hint: None,
        stop_loss_hint: None,
        time_in_force: TimeInForce::GTC,
        origin_ref: "demo".to_string(),
        ts_intent: 1700000000,
    }
}

fn make_decision(verdict: RiskVerdict) -> RiskDecision {
    let is_deny = verdict == RiskVerdict::Deny;
    RiskDecision {
        decision_id: uuid::Uuid::new_v4().to_string(),
        target_ref: "demo".to_string(),
        decision: verdict,
        severity: if is_deny { Severity::Critical } else { Severity::Info },
        rule_hits: vec![],
        exposure_snapshot: None,
        required_actions: vec![],
        review_required: false,
        ts_decision: 1700000000,
    }
}
