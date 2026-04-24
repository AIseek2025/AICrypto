use aicrypto_protocols::order_intent::*;
use aicrypto_protocols::signal_event::{Direction, Horizon, SignalEvent, SignalType};
use chrono::Utc;
use tracing;
use uuid::Uuid;

pub struct OrderBuilder {
    account_scope: String,
    default_leverage: u32,
    default_time_in_force: TimeInForce,
}

impl OrderBuilder {
    pub fn new(account_scope: &str) -> Self {
        Self {
            account_scope: account_scope.to_string(),
            default_leverage: 3,
            default_time_in_force: TimeInForce::GTC,
        }
    }

    pub fn with_default_leverage(mut self, leverage: u32) -> Self {
        self.default_leverage = leverage;
        self
    }

    pub fn signal_to_intent(
        &self,
        signal: &SignalEvent,
        quantity: f64,
        price: Option<f64>,
    ) -> OrderIntent {
        let (side, position_side) = match &signal.direction {
            Direction::LONG => (Side::BUY, PositionSide::LONG),
            Direction::SHORT => (Side::SELL, PositionSide::SHORT),
            Direction::NEUTRAL => match &signal.signal_type {
                SignalType::Reduce | SignalType::RiskAlert => (Side::SELL, PositionSide::BOTH),
                _ => (Side::BUY, PositionSide::BOTH),
            },
        };

        let (order_type, price_limit, time_in_force) = match price {
            Some(p) => (OrderType::LIMIT, Some(format!("{:.8}", p)), self.default_time_in_force.clone()),
            None => (OrderType::MARKET, None, TimeInForce::IOC),
        };

        let is_reduce = matches!(
            signal.signal_type,
            SignalType::Exit | SignalType::Reduce | SignalType::RiskAlert
        );

        let leverage_hint = if is_reduce {
            None
        } else {
            Some(self.default_leverage)
        };

        OrderIntent {
            intent_id: Uuid::new_v4().to_string(),
            account_scope: self.account_scope.clone(),
            symbol: signal.symbol.clone(),
            side,
            position_side,
            order_type,
            quantity: format!("{:.8}", quantity),
            price_limit,
            reduce_only: is_reduce,
            leverage_hint,
            take_profit_hint: None,
            stop_loss_hint: None,
            time_in_force,
            origin_ref: signal.signal_id.clone(),
            ts_intent: Utc::now().timestamp_millis(),
        }
    }

    pub fn build_stop_loss(
        &self,
        symbol: &str,
        position_side: PositionSide,
        quantity: f64,
        stop_price: f64,
        origin_ref: &str,
    ) -> OrderIntent {
        OrderIntent {
            intent_id: Uuid::new_v4().to_string(),
            account_scope: self.account_scope.clone(),
            symbol: symbol.to_string(),
            side: Side::SELL,
            position_side,
            order_type: OrderType::STOP_MARKET,
            quantity: format!("{:.8}", quantity),
            price_limit: None,
            reduce_only: true,
            leverage_hint: None,
            take_profit_hint: None,
            stop_loss_hint: Some(StopLossHint {
                sl_type: "stop_market".to_string(),
                stop_price: format!("{:.8}", stop_price),
            }),
            time_in_force: TimeInForce::GTC,
            origin_ref: origin_ref.to_string(),
            ts_intent: Utc::now().timestamp_millis(),
        }
    }

    pub fn build_take_profit(
        &self,
        symbol: &str,
        position_side: PositionSide,
        quantity: f64,
        stop_price: f64,
        origin_ref: &str,
    ) -> OrderIntent {
        OrderIntent {
            intent_id: Uuid::new_v4().to_string(),
            account_scope: self.account_scope.clone(),
            symbol: symbol.to_string(),
            side: Side::SELL,
            position_side,
            order_type: OrderType::TAKE_PROFIT_MARKET,
            quantity: format!("{:.8}", quantity),
            price_limit: None,
            reduce_only: true,
            leverage_hint: None,
            take_profit_hint: Some(TakeProfitHint {
                tp_type: "take_profit_market".to_string(),
                activation_price: None,
                callback_rate: None,
                stop_price: Some(format!("{:.8}", stop_price)),
            }),
            stop_loss_hint: None,
            time_in_force: TimeInForce::GTC,
            origin_ref: origin_ref.to_string(),
            ts_intent: Utc::now().timestamp_millis(),
        }
    }
}

pub fn calculate_position_size(
    equity: f64,
    risk_pct: f64,
    entry_price: f64,
    stop_price: f64,
    leverage: u32,
) -> f64 {
    if entry_price == 0.0 || stop_price == 0.0 {
        return 0.0;
    }

    let risk_amount = equity * risk_pct;
    let price_risk = (entry_price - stop_price).abs();
    if price_risk == 0.0 {
        return 0.0;
    }

    let base_qty = risk_amount / price_risk;
    let leveraged_qty = base_qty * leverage as f64;

    let max_notional = equity * leverage as f64;
    let max_qty = max_notional / entry_price;

    leveraged_qty.min(max_qty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aicrypto_protocols::signal_event::SignalEvent;

    fn make_signal(signal_type: SignalType, direction: Direction, symbol: &str) -> SignalEvent {
        SignalEvent {
            signal_id: "sig-001".to_string(),
            signal_type,
            symbol: symbol.to_string(),
            direction,
            confidence: 0.8,
            horizon: Horizon::Swing,
            reason_codes: vec!["breakout".to_string()],
            evidence_refs: vec!["high_20d".to_string()],
            ts_signal: 1700000000,
        }
    }

    #[test]
    fn test_entry_long_market() {
        let builder = OrderBuilder::new("main");
        let signal = make_signal(SignalType::Entry, Direction::LONG, "BTCUSDT");
        let intent = builder.signal_to_intent(&signal, 0.1, None);

        assert_eq!(intent.side, Side::BUY);
        assert_eq!(intent.position_side, PositionSide::LONG);
        assert_eq!(intent.order_type, OrderType::MARKET);
        assert!(!intent.reduce_only);
        assert_eq!(intent.leverage_hint, Some(3));
    }

    #[test]
    fn test_entry_short_limit() {
        let builder = OrderBuilder::new("main");
        let signal = make_signal(SignalType::Entry, Direction::SHORT, "ETHUSDT");
        let intent = builder.signal_to_intent(&signal, 1.0, Some(3500.0));

        assert_eq!(intent.side, Side::SELL);
        assert_eq!(intent.position_side, PositionSide::SHORT);
        assert_eq!(intent.order_type, OrderType::LIMIT);
        assert_eq!(intent.price_limit, Some("3500.00000000".to_string()));
    }

    #[test]
    fn test_exit_reduce_only() {
        let builder = OrderBuilder::new("main");
        let signal = make_signal(SignalType::Exit, Direction::LONG, "BTCUSDT");
        let intent = builder.signal_to_intent(&signal, 0.1, None);

        assert!(intent.reduce_only);
        assert_eq!(intent.order_type, OrderType::MARKET);
        assert_eq!(intent.time_in_force, TimeInForce::IOC);
    }

    #[test]
    fn test_risk_alert_neutral() {
        let builder = OrderBuilder::new("main");
        let signal = make_signal(SignalType::RiskAlert, Direction::NEUTRAL, "BTCUSDT");
        let intent = builder.signal_to_intent(&signal, 0.05, None);

        assert_eq!(intent.side, Side::SELL);
        assert!(intent.reduce_only);
    }

    #[test]
    fn test_position_size_calculation() {
        let qty = calculate_position_size(10000.0, 0.02, 65000.0, 63000.0, 3);
        assert!(qty > 0.0);
        assert!(qty <= 10000.0 * 3.0 / 65000.0);
    }

    #[test]
    fn test_stop_loss_builder() {
        let builder = OrderBuilder::new("main");
        let sl = builder.build_stop_loss("BTCUSDT", PositionSide::LONG, 0.1, 63000.0, "sig-001");
        assert_eq!(sl.order_type, OrderType::STOP_MARKET);
        assert!(sl.reduce_only);
        assert!(sl.stop_loss_hint.is_some());
    }
}
