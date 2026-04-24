use aicrypto_protocols::execution_report::{OrderStatus, ReconcileState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderEvent {
    RiskApproved,
    RiskDenied,
    Sent,
    Acked { exchange_order_id: String },
    PartiallyFilled { filled_qty: f64, fill_price: f64 },
    Filled { filled_qty: f64, fill_price: f64 },
    CancelRequested,
    Canceled,
    Rejected { reason: String },
    Expired,
    Reconciled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedOrder {
    pub intent_id: String,
    pub symbol: String,
    pub status: OrderStatus,
    pub filled_qty: f64,
    pub avg_fill_price: f64,
    pub exchange_order_id: Option<String>,
    pub reject_reason: Option<String>,
    pub reconcile_state: Option<ReconcileState>,
    pub commission: f64,
}

impl TrackedOrder {
    pub fn new(intent_id: &str, symbol: &str) -> Self {
        Self {
            intent_id: intent_id.to_string(),
            symbol: symbol.to_string(),
            status: OrderStatus::Created,
            filled_qty: 0.0,
            avg_fill_price: 0.0,
            exchange_order_id: None,
            reject_reason: None,
            reconcile_state: None,
            commission: 0.0,
        }
    }
}

pub struct OrderStateMachine {
    orders: HashMap<String, TrackedOrder>,
}

impl OrderStateMachine {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
        }
    }

    pub fn create_order(&mut self, intent_id: &str, symbol: &str) {
        let order = TrackedOrder::new(intent_id, symbol);
        self.orders.insert(intent_id.to_string(), order);
        tracing::info!(intent_id = intent_id, symbol = symbol, "order created");
    }

    pub fn transition(&mut self, intent_id: &str, event: OrderEvent) -> Result<OrderStatus, String> {
        let order = self.orders.get_mut(intent_id)
            .ok_or_else(|| format!("order not found: {}", intent_id))?;

        let new_status = match (&order.status, &event) {
            (OrderStatus::Created, OrderEvent::RiskApproved) => OrderStatus::RiskApproved,
            (OrderStatus::Created, OrderEvent::RiskDenied) => {
                order.reject_reason = Some("risk denied".to_string());
                OrderStatus::Rejected
            }
            (OrderStatus::RiskApproved, OrderEvent::Sent) => OrderStatus::Sent,
            (OrderStatus::Sent, OrderEvent::Acked { exchange_order_id }) => {
                order.exchange_order_id = Some(exchange_order_id.clone());
                OrderStatus::Acked
            }
            (OrderStatus::Sent | OrderStatus::Acked, OrderEvent::PartiallyFilled { filled_qty, fill_price }) => {
                let prev_qty = order.filled_qty;
                let prev_price = order.avg_fill_price;
                let total_qty = prev_qty + filled_qty;
                if total_qty > 0.0 {
                    order.avg_fill_price = (prev_price * prev_qty + fill_price * filled_qty) / total_qty;
                }
                order.filled_qty = total_qty;
                order.commission += filled_qty * fill_price * 0.0004;
                OrderStatus::PartiallyFilled
            }
            (OrderStatus::Sent | OrderStatus::Acked | OrderStatus::PartiallyFilled,
             OrderEvent::Filled { filled_qty, fill_price }) => {
                let prev_qty = order.filled_qty;
                let prev_price = order.avg_fill_price;
                let total_qty = prev_qty + filled_qty;
                if total_qty > 0.0 {
                    order.avg_fill_price = (prev_price * prev_qty + fill_price * filled_qty) / total_qty;
                }
                order.filled_qty = total_qty;
                order.commission += filled_qty * fill_price * 0.0004;
                order.reconcile_state = Some(ReconcileState::Pending);
                OrderStatus::Filled
            }
            (OrderStatus::Acked | OrderStatus::PartiallyFilled, OrderEvent::CancelRequested) => {
                OrderStatus::CancelPending
            }
            (OrderStatus::CancelPending, OrderEvent::Canceled) => OrderStatus::Canceled,
            (OrderStatus::Sent | OrderStatus::Acked, OrderEvent::Rejected { reason }) => {
                order.reject_reason = Some(reason.clone());
                OrderStatus::Rejected
            }
            (OrderStatus::Acked, OrderEvent::Expired) => OrderStatus::Expired,
            (OrderStatus::Filled, OrderEvent::Reconciled) => {
                order.reconcile_state = Some(ReconcileState::Matched);
                OrderStatus::Reconciled
            }
            (current, ev) => {
                return Err(format!("invalid transition: {:?} + {:?}", current, ev));
            }
        };

        tracing::info!(
            intent_id = intent_id,
            from = ?order.status,
            to = ?new_status,
            "order state transition"
        );
        order.status = new_status.clone();
        Ok(new_status)
    }

    pub fn get_order(&self, intent_id: &str) -> Option<&TrackedOrder> {
        self.orders.get(intent_id)
    }

    pub fn active_orders(&self) -> Vec<&TrackedOrder> {
        self.orders.values()
            .filter(|o| !matches!(o.status,
                OrderStatus::Filled | OrderStatus::Canceled |
                OrderStatus::Rejected | OrderStatus::Expired | OrderStatus::Reconciled))
            .collect()
    }

    pub fn total_orders(&self) -> usize {
        self.orders.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path_market_order() {
        let mut osm = OrderStateMachine::new();
        osm.create_order("intent-1", "BTCUSDT");

        osm.transition("intent-1", OrderEvent::RiskApproved).unwrap();
        osm.transition("intent-1", OrderEvent::Sent).unwrap();
        osm.transition("intent-1", OrderEvent::Acked {
            exchange_order_id: "ex-123".to_string(),
        }).unwrap();
        osm.transition("intent-1", OrderEvent::Filled {
            filled_qty: 0.1,
            fill_price: 65000.0,
        }).unwrap();

        let order = osm.get_order("intent-1").unwrap();
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.filled_qty, 0.1);
        assert_eq!(order.avg_fill_price, 65000.0);
        assert!(order.commission > 0.0);
    }

    #[test]
    fn test_partial_fill_then_full() {
        let mut osm = OrderStateMachine::new();
        osm.create_order("intent-2", "ETHUSDT");

        osm.transition("intent-2", OrderEvent::RiskApproved).unwrap();
        osm.transition("intent-2", OrderEvent::Sent).unwrap();
        osm.transition("intent-2", OrderEvent::PartiallyFilled {
            filled_qty: 0.5,
            fill_price: 3500.0,
        }).unwrap();
        osm.transition("intent-2", OrderEvent::Filled {
            filled_qty: 0.5,
            fill_price: 3510.0,
        }).unwrap();

        let order = osm.get_order("intent-2").unwrap();
        assert_eq!(order.filled_qty, 1.0);
        assert!((order.avg_fill_price - 3505.0).abs() < 1.0);
    }

    #[test]
    fn test_risk_denied() {
        let mut osm = OrderStateMachine::new();
        osm.create_order("intent-3", "BTCUSDT");

        osm.transition("intent-3", OrderEvent::RiskDenied).unwrap();
        let order = osm.get_order("intent-3").unwrap();
        assert_eq!(order.status, OrderStatus::Rejected);
    }

    #[test]
    fn test_invalid_transition() {
        let mut osm = OrderStateMachine::new();
        osm.create_order("intent-4", "BTCUSDT");

        let result = osm.transition("intent-4", OrderEvent::Filled {
            filled_qty: 0.1,
            fill_price: 65000.0,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_flow() {
        let mut osm = OrderStateMachine::new();
        osm.create_order("intent-5", "SOLUSDT");

        osm.transition("intent-5", OrderEvent::RiskApproved).unwrap();
        osm.transition("intent-5", OrderEvent::Sent).unwrap();
        osm.transition("intent-5", OrderEvent::Acked {
            exchange_order_id: "ex-456".to_string(),
        }).unwrap();
        osm.transition("intent-5", OrderEvent::CancelRequested).unwrap();
        osm.transition("intent-5", OrderEvent::Canceled).unwrap();

        let order = osm.get_order("intent-5").unwrap();
        assert_eq!(order.status, OrderStatus::Canceled);
    }
}
