use crate::order_builder::{calculate_position_size, OrderBuilder};
use crate::position_tracker::{PositionSide, PositionTracker};
use aicrypto_protocols::order_intent::OrderIntent;
use aicrypto_protocols::signal_event::{Direction, SignalEvent, SignalType};
use tracing;

pub struct PortfolioManager {
    tracker: PositionTracker,
    builder: OrderBuilder,
    equity: f64,
    max_risk_per_trade_pct: f64,
    max_total_exposure_pct: f64,
    max_positions: usize,
}

impl PortfolioManager {
    pub fn new(equity: f64, account_scope: &str) -> Self {
        Self {
            tracker: PositionTracker::new(),
            builder: OrderBuilder::new(account_scope).with_default_leverage(3),
            equity,
            max_risk_per_trade_pct: 0.02,
            max_total_exposure_pct: 0.80,
            max_positions: 5,
        }
    }

    pub fn with_max_risk(mut self, pct: f64) -> Self {
        self.max_risk_per_trade_pct = pct;
        self
    }

    pub fn with_max_exposure(mut self, pct: f64) -> Self {
        self.max_total_exposure_pct = pct;
        self
    }

    pub fn with_max_positions(mut self, n: usize) -> Self {
        self.max_positions = n;
        self
    }

    pub fn process_signal(&mut self, signal: &SignalEvent, current_price: f64) -> Option<OrderIntent> {
        tracing::info!(
            signal_id = %signal.signal_id,
            signal_type = ?signal.signal_type,
            direction = ?signal.direction,
            symbol = %signal.symbol,
            confidence = signal.confidence,
            "processing signal"
        );

        match signal.signal_type {
            SignalType::Entry => self.handle_entry(signal, current_price),
            SignalType::Exit => self.handle_exit(signal, current_price),
            SignalType::Add => self.handle_add(signal, current_price),
            SignalType::Reduce => self.handle_reduce(signal, current_price),
            SignalType::RiskAlert => self.handle_risk_alert(signal, current_price),
        }
    }

    fn handle_entry(&mut self, signal: &SignalEvent, current_price: f64) -> Option<OrderIntent> {
        if self.tracker.open_position_count() >= self.max_positions {
            tracing::warn!(
                open = self.tracker.open_position_count(),
                max = self.max_positions,
                "max positions reached, rejecting entry"
            );
            return None;
        }

        let max_notional = self.equity * self.max_total_exposure_pct;
        if self.tracker.total_exposure() >= max_notional {
            tracing::warn!("max total exposure reached, rejecting entry");
            return None;
        }

        if self.tracker.get_position(&signal.symbol).map_or(false, |p| p.is_open()) {
            tracing::warn!(symbol = %signal.symbol, "already have position, rejecting entry");
            return None;
        }

        let atr_pct = 0.02;
        let stop_distance = current_price * atr_pct;
        let stop_price = match signal.direction {
            Direction::LONG => current_price - stop_distance,
            Direction::SHORT => current_price + stop_distance,
            _ => current_price,
        };

        let quantity = calculate_position_size(
            self.equity,
            self.max_risk_per_trade_pct,
            current_price,
            stop_price,
            3,
        );

        if quantity <= 0.0 {
            tracing::warn!("calculated quantity is zero, rejecting entry");
            return None;
        }

        let intent = self.builder.signal_to_intent(signal, quantity, None);

        let pos_side = match signal.direction {
            Direction::LONG => PositionSide::Long,
            Direction::SHORT => PositionSide::Short,
            _ => PositionSide::Flat,
        };
        self.tracker.open_position(&signal.symbol, pos_side, quantity, current_price, 3);

        Some(intent)
    }

    fn handle_exit(&mut self, signal: &SignalEvent, _current_price: f64) -> Option<OrderIntent> {
        let pos = self.tracker.get_position(&signal.symbol)?;
        if !pos.is_open() {
            tracing::warn!(symbol = %signal.symbol, "no open position to exit");
            return None;
        }

        let intent = self.builder.signal_to_intent(signal, pos.quantity, None);
        Some(intent)
    }

    fn handle_add(&mut self, signal: &SignalEvent, current_price: f64) -> Option<OrderIntent> {
        let pos = self.tracker.get_position(&signal.symbol)?;
        if !pos.is_open() {
            tracing::warn!(symbol = %signal.symbol, "no position to add to");
            return None;
        }

        let add_qty = pos.quantity * 0.5;
        let intent = self.builder.signal_to_intent(signal, add_qty, None);

        self.tracker.open_position(&signal.symbol, pos.side.clone(), add_qty, current_price, pos.leverage);

        Some(intent)
    }

    fn handle_reduce(&mut self, signal: &SignalEvent, _current_price: f64) -> Option<OrderIntent> {
        let pos = self.tracker.get_position(&signal.symbol)?;
        if !pos.is_open() {
            return None;
        }

        let reduce_qty = pos.quantity * 0.5;
        let intent = self.builder.signal_to_intent(signal, reduce_qty, None);
        Some(intent)
    }

    fn handle_risk_alert(&mut self, signal: &SignalEvent, _current_price: f64) -> Option<OrderIntent> {
        let pos = self.tracker.get_position(&signal.symbol)?;
        if !pos.is_open() {
            return None;
        }

        let reduce_qty = pos.quantity * 0.5;
        let intent = self.builder.signal_to_intent(signal, reduce_qty, None);
        Some(intent)
    }

    pub fn tracker(&self) -> &PositionTracker {
        &self.tracker
    }

    pub fn equity(&self) -> f64 {
        self.equity
    }

    pub fn set_equity(&mut self, equity: f64) {
        self.equity = equity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aicrypto_protocols::signal_event::Horizon;

    fn make_entry_signal(direction: Direction, symbol: &str) -> SignalEvent {
        SignalEvent {
            signal_id: "sig-001".to_string(),
            signal_type: SignalType::Entry,
            symbol: symbol.to_string(),
            direction,
            confidence: 0.85,
            horizon: Horizon::Swing,
            reason_codes: vec!["breakout".to_string()],
            evidence_refs: vec!["high_20d".to_string()],
            ts_signal: 1700000000,
        }
    }

    fn make_exit_signal(symbol: &str) -> SignalEvent {
        SignalEvent {
            signal_id: "sig-002".to_string(),
            signal_type: SignalType::Exit,
            symbol: symbol.to_string(),
            direction: Direction::LONG,
            confidence: 0.7,
            horizon: Horizon::Swing,
            reason_codes: vec!["trailing_stop".to_string()],
            evidence_refs: vec![],
            ts_signal: 1700000100,
        }
    }

    #[test]
    fn test_full_entry_exit_cycle() {
        let mut pm = PortfolioManager::new(100000.0, "main");

        let entry_sig = make_entry_signal(Direction::LONG, "BTCUSDT");
        let intent = pm.process_signal(&entry_sig, 65000.0);
        assert!(intent.is_some());

        let intent = intent.unwrap();
        assert_eq!(intent.symbol, "BTCUSDT");

        let exit_sig = make_exit_signal("BTCUSDT");
        let exit_intent = pm.process_signal(&exit_sig, 68000.0);
        assert!(exit_intent.is_some());
    }

    #[test]
    fn test_reject_duplicate_position() {
        let mut pm = PortfolioManager::new(100000.0, "main");

        let sig = make_entry_signal(Direction::LONG, "BTCUSDT");
        assert!(pm.process_signal(&sig, 65000.0).is_some());
        assert!(pm.process_signal(&sig, 66000.0).is_none());
    }

    #[test]
    fn test_max_positions_limit() {
        let mut pm = PortfolioManager::new(100000.0, "main")
            .with_max_positions(2)
            .with_max_exposure(0.90)
            .with_max_risk(0.002);

        for sym in &["BTCUSDT", "ETHUSDT", "SOLUSDT"] {
            let sig = make_entry_signal(Direction::LONG, sym);
            let result = pm.process_signal(&sig, 50000.0);
        }

        assert_eq!(pm.tracker().open_position_count(), 2);
    }

    #[test]
    fn test_reduce_signal() {
        let mut pm = PortfolioManager::new(100000.0, "main");

        let entry = make_entry_signal(Direction::LONG, "BTCUSDT");
        pm.process_signal(&entry, 65000.0);

        let reduce = SignalEvent {
            signal_id: "sig-r1".to_string(),
            signal_type: SignalType::Reduce,
            symbol: "BTCUSDT".to_string(),
            direction: Direction::NEUTRAL,
            confidence: 0.6,
            horizon: Horizon::Intraday,
            reason_codes: vec!["volatility".to_string()],
            evidence_refs: vec![],
            ts_signal: 1700000100,
        };

        let intent = pm.process_signal(&reduce, 64000.0);
        assert!(intent.is_some());
    }
}
