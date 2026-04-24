use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub side: PositionSide,
    pub quantity: f64,
    pub entry_price: f64,
    pub mark_price: Option<f64>,
    pub leverage: u32,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PositionSide {
    Long,
    Short,
    Flat,
}

impl Position {
    pub fn new(symbol: &str, side: PositionSide) -> Self {
        Self {
            symbol: symbol.to_string(),
            side,
            quantity: 0.0,
            entry_price: 0.0,
            mark_price: None,
            leverage: 1,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        }
    }

    pub fn is_open(&self) -> bool {
        self.quantity > 0.0 && self.side != PositionSide::Flat
    }

    pub fn notional_value(&self) -> f64 {
        self.quantity * self.mark_price.unwrap_or(self.entry_price)
    }

    pub fn margin_used(&self) -> f64 {
        let lev = self.leverage.max(1) as f64;
        self.notional_value() / lev
    }

    pub fn update_mark_price(&mut self, price: f64) {
        self.mark_price = Some(price);
        self.unrealized_pnl = match self.side {
            PositionSide::Long => (price - self.entry_price) * self.quantity,
            PositionSide::Short => (self.entry_price - price) * self.quantity,
            PositionSide::Flat => 0.0,
        };
    }
}

pub struct PositionTracker {
    positions: HashMap<String, Position>,
    total_realized_pnl: f64,
}

impl PositionTracker {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            total_realized_pnl: 0.0,
        }
    }

    pub fn open_position(
        &mut self,
        symbol: &str,
        side: PositionSide,
        quantity: f64,
        price: f64,
        leverage: u32,
    ) {
        let pos = self.positions.entry(symbol.to_string()).or_insert_with(|| {
            Position::new(symbol, side)
        });

        if pos.side != side && pos.quantity > 0.0 {
            tracing::warn!(
                symbol = symbol,
                current_side = ?pos.side,
                new_side = ?side,
                "position side mismatch, closing existing first"
            );
            self.total_realized_pnl += pos.unrealized_pnl;
            pos.realized_pnl += pos.unrealized_pnl;
            pos.quantity = 0.0;
            pos.unrealized_pnl = 0.0;
        }

        if pos.quantity > 0.0 {
            let total_cost = pos.entry_price * pos.quantity + price * quantity;
            let total_qty = pos.quantity + quantity;
            pos.entry_price = total_cost / total_qty;
            pos.quantity = total_qty;
        } else {
            pos.entry_price = price;
            pos.quantity = quantity;
        }
        pos.side = side;
        pos.leverage = leverage;

        tracing::info!(
            symbol = symbol,
            side = ?side,
            quantity = quantity,
            price = price,
            leverage = leverage,
            total_qty = pos.quantity,
            avg_entry = pos.entry_price,
            "position opened/increased"
        );
    }

    pub fn reduce_position(
        &mut self,
        symbol: &str,
        quantity: f64,
        price: f64,
    ) -> Option<f64> {
        let pos = self.positions.get_mut(symbol)?;

        if pos.quantity < quantity {
            tracing::warn!(
                symbol = symbol,
                requested = quantity,
                available = pos.quantity,
                "reduce quantity exceeds position"
            );
            return None;
        }

        let realized = match pos.side {
            PositionSide::Long => (price - pos.entry_price) * quantity,
            PositionSide::Short => (pos.entry_price - price) * quantity,
            PositionSide::Flat => 0.0,
        };

        pos.quantity -= quantity;
        pos.realized_pnl += realized;
        self.total_realized_pnl += realized;

        if pos.quantity < 1e-10 {
            pos.quantity = 0.0;
            pos.side = PositionSide::Flat;
            pos.unrealized_pnl = 0.0;
            tracing::info!(
                symbol = symbol,
                realized_pnl = realized,
                "position fully closed"
            );
        } else {
            tracing::info!(
                symbol = symbol,
                reduced_qty = quantity,
                remaining = pos.quantity,
                realized_pnl = realized,
                "position reduced"
            );
        }

        Some(realized)
    }

    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    pub fn all_positions(&self) -> Vec<&Position> {
        self.positions.values().filter(|p| p.is_open()).collect()
    }

    pub fn total_unrealized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.unrealized_pnl).sum()
    }

    pub fn total_realized_pnl(&self) -> f64 {
        self.total_realized_pnl
    }

    pub fn total_exposure(&self) -> f64 {
        self.positions.values().filter(|p| p.is_open()).map(|p| p.notional_value()).sum()
    }

    pub fn open_position_count(&self) -> usize {
        self.positions.values().filter(|p| p.is_open()).count()
    }

    pub fn update_mark_price(&mut self, symbol: &str, price: f64) {
        if let Some(pos) = self.positions.get_mut(symbol) {
            pos.update_mark_price(price);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_and_close_position() {
        let mut tracker = PositionTracker::new();

        tracker.open_position("BTCUSDT", PositionSide::Long, 0.1, 65000.0, 3);

        let pos = tracker.get_position("BTCUSDT").unwrap();
        assert_eq!(pos.quantity, 0.1);
        assert_eq!(pos.entry_price, 65000.0);
        assert_eq!(pos.leverage, 3);

        let realized = tracker.reduce_position("BTCUSDT", 0.1, 68000.0).unwrap();
        assert_eq!(realized, 300.0);

        let pos = tracker.get_position("BTCUSDT").unwrap();
        assert_eq!(pos.quantity, 0.0);
        assert_eq!(pos.side, PositionSide::Flat);
        assert_eq!(tracker.total_realized_pnl(), 300.0);
    }

    #[test]
    fn test_add_to_position() {
        let mut tracker = PositionTracker::new();

        tracker.open_position("BTCUSDT", PositionSide::Long, 0.1, 60000.0, 2);
        tracker.open_position("BTCUSDT", PositionSide::Long, 0.1, 70000.0, 2);

        let pos = tracker.get_position("BTCUSDT").unwrap();
        assert_eq!(pos.quantity, 0.2);
        assert!((pos.entry_price - 65000.0).abs() < 1.0);
    }

    #[test]
    fn test_short_position() {
        let mut tracker = PositionTracker::new();

        tracker.open_position("ETHUSDT", PositionSide::Short, 1.0, 3500.0, 2);
        let realized = tracker.reduce_position("ETHUSDT", 1.0, 3200.0).unwrap();
        assert_eq!(realized, 300.0);
    }

    #[test]
    fn test_mark_price_update() {
        let mut tracker = PositionTracker::new();
        tracker.open_position("BTCUSDT", PositionSide::Long, 0.5, 60000.0, 2);

        let pos = tracker.get_position("BTCUSDT").unwrap();
        assert_eq!(pos.quantity, 0.5);
        assert_eq!(pos.entry_price, 60000.0);
    }

    #[test]
    fn test_total_exposure() {
        let mut tracker = PositionTracker::new();
        tracker.open_position("BTCUSDT", PositionSide::Long, 0.1, 60000.0, 2);
        tracker.open_position("ETHUSDT", PositionSide::Short, 1.0, 3500.0, 2);

        let exposure = tracker.total_exposure();
        assert!(exposure > 0.0);
        assert_eq!(tracker.open_position_count(), 2);
    }
}
