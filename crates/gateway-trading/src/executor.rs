use crate::order_state_machine::{OrderEvent, OrderStateMachine};
use aicrypto_protocols::execution_report::{ExecutionReport, FeeDetail, OrderStatus};
use aicrypto_protocols::order_intent::OrderIntent;
use aicrypto_protocols::risk_decision::RiskDecision;
use aicrypto_protocols::risk_decision::RiskVerdict;
use chrono::Utc;
use tracing;
use uuid::Uuid;

pub struct TradeExecutor {
    osm: OrderStateMachine,
    exchange: String,
    dry_run: bool,
}

impl TradeExecutor {
    pub fn new(exchange: &str, dry_run: bool) -> Self {
        Self {
            osm: OrderStateMachine::new(),
            exchange: exchange.to_string(),
            dry_run,
        }
    }

    pub fn submit(
        &mut self,
        intent: &OrderIntent,
        risk_decision: &RiskDecision,
    ) -> anyhow::Result<ExecutionReport> {
        tracing::info!(
            intent_id = %intent.intent_id,
            symbol = %intent.symbol,
            side = ?intent.side,
            quantity = %intent.quantity,
            order_type = ?intent.order_type,
            dry_run = self.dry_run,
            "submitting order"
        );

        self.osm.create_order(&intent.intent_id, &intent.symbol);

        if risk_decision.decision != RiskVerdict::Allow {
            tracing::warn!(
                intent_id = %intent.intent_id,
                decision = ?risk_decision.decision,
                "order denied by risk engine"
            );
            let _ = self.osm.transition(&intent.intent_id, OrderEvent::RiskDenied);

            return Ok(self.build_report(intent, OrderStatus::Rejected));
        }

        self.osm.transition(&intent.intent_id, OrderEvent::RiskApproved)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        if self.dry_run {
            tracing::info!(intent_id = %intent.intent_id, "dry-run mode: simulating execution");
        } else {
            tracing::info!(intent_id = %intent.intent_id, "live mode: would send to exchange");
        }
        self.simulate_execution(intent)?;

        let order = self.osm.get_order(&intent.intent_id).unwrap();
        Ok(self.build_report_from_tracked(intent, order))
    }

    fn simulate_execution(&mut self, intent: &OrderIntent) -> anyhow::Result<()> {
        self.osm.transition(&intent.intent_id, OrderEvent::Sent)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let exchange_id = format!("sim-{}", Uuid::new_v4().as_simple());
        self.osm.transition(&intent.intent_id, OrderEvent::Acked {
            exchange_order_id: exchange_id,
        }).map_err(|e| anyhow::anyhow!("{}", e))?;

        let quantity: f64 = intent.quantity.parse().unwrap_or(0.0);
        let price = intent
            .price_limit
            .as_ref()
            .and_then(|p| p.parse::<f64>().ok())
            .unwrap_or(65000.0);

        let fill_qty = quantity;
        let fill_price = price * (1.0 + (rand_like_positive() - 0.5) * 0.001);

        self.osm.transition(&intent.intent_id, OrderEvent::Filled {
            filled_qty: fill_qty,
            fill_price,
        }).map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(())
    }

    fn build_report(&self, intent: &OrderIntent, status: OrderStatus) -> ExecutionReport {
        let order = self.osm.get_order(&intent.intent_id);
        let (filled_qty, avg_price, fees, exchange_order_id) = match order {
            Some(o) => (
                if o.filled_qty > 0.0 { Some(format!("{:.8}", o.filled_qty)) } else { None },
                if o.avg_fill_price > 0.0 { Some(format!("{:.8}", o.avg_fill_price)) } else { None },
                if o.commission > 0.0 { Some(FeeDetail {
                    commission: format!("{:.8}", o.commission),
                    commission_asset: "USDT".to_string(),
                })} else { None },
                o.exchange_order_id.clone(),
            ),
            None => (None, None, None, None),
        };

        ExecutionReport {
            report_id: Uuid::new_v4().to_string(),
            intent_id: intent.intent_id.clone(),
            exchange: self.exchange.clone(),
            symbol: intent.symbol.clone(),
            order_status: status,
            filled_qty,
            avg_fill_price: avg_price,
            fees,
            exchange_order_id,
            raw_status: None,
            reconcile_state: None,
            ts_report: Utc::now().timestamp_millis(),
        }
    }

    fn build_report_from_tracked(
        &self,
        intent: &OrderIntent,
        order: &crate::order_state_machine::TrackedOrder,
    ) -> ExecutionReport {
        ExecutionReport {
            report_id: Uuid::new_v4().to_string(),
            intent_id: intent.intent_id.clone(),
            exchange: self.exchange.clone(),
            symbol: intent.symbol.clone(),
            order_status: order.status.clone(),
            filled_qty: if order.filled_qty > 0.0 {
                Some(format!("{:.8}", order.filled_qty))
            } else {
                None
            },
            avg_fill_price: if order.avg_fill_price > 0.0 {
                Some(format!("{:.8}", order.avg_fill_price))
            } else {
                None
            },
            fees: if order.commission > 0.0 {
                Some(FeeDetail {
                    commission: format!("{:.8}", order.commission),
                    commission_asset: "USDT".to_string(),
                })
            } else {
                None
            },
            exchange_order_id: order.exchange_order_id.clone(),
            raw_status: None,
            reconcile_state: order.reconcile_state.clone(),
            ts_report: Utc::now().timestamp_millis(),
        }
    }

    pub fn osm(&self) -> &OrderStateMachine {
        &self.osm
    }
}

fn rand_like_positive() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as f64 / u32::MAX as f64).min(1.0).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aicrypto_protocols::order_intent::*;
    use aicrypto_protocols::risk_decision::{RiskDecision, RiskVerdict, Severity};

    fn make_intent(symbol: &str, quantity: &str, price: Option<&str>) -> OrderIntent {
        OrderIntent {
            intent_id: "test-intent-1".to_string(),
            account_scope: "main".to_string(),
            symbol: symbol.to_string(),
            side: Side::BUY,
            position_side: PositionSide::LONG,
            order_type: if price.is_some() { OrderType::LIMIT } else { OrderType::MARKET },
            quantity: quantity.to_string(),
            price_limit: price.map(|p| p.to_string()),
            reduce_only: false,
            leverage_hint: Some(3),
            take_profit_hint: None,
            stop_loss_hint: None,
            time_in_force: TimeInForce::GTC,
            origin_ref: "sig-001".to_string(),
            ts_intent: 1700000000,
        }
    }

    fn make_risk_decision(verdict: RiskVerdict) -> RiskDecision {
        RiskDecision {
            decision_id: "rd-001".to_string(),
            target_ref: "test-intent-1".to_string(),
            decision: verdict,
            severity: Severity::Info,
            rule_hits: vec![],
            exposure_snapshot: None,
            required_actions: vec![],
            review_required: false,
            ts_decision: 1700000000,
        }
    }

    #[test]
    fn test_execute_approved_order() {
        let mut executor = TradeExecutor::new("binance_testnet", true);
        let intent = make_intent("BTCUSDT", "0.1", Some("65000"));
        let risk = make_risk_decision(RiskVerdict::Allow);

        let report = executor.submit(&intent, &risk).unwrap();
        assert_eq!(report.order_status, OrderStatus::Filled);
        assert!(report.filled_qty.is_some());
        assert!(report.avg_fill_price.is_some());
        assert!(report.fees.is_some());
        assert!(report.exchange_order_id.is_some());
    }

    #[test]
    fn test_deny_rejected_order() {
        let mut executor = TradeExecutor::new("binance_testnet", true);
        let intent = make_intent("BTCUSDT", "0.1", Some("65000"));
        let risk = make_risk_decision(RiskVerdict::Deny);

        let report = executor.submit(&intent, &risk).unwrap();
        assert_eq!(report.order_status, OrderStatus::Rejected);
        assert!(report.filled_qty.is_none());
    }
}
