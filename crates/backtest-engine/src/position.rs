use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub side: PositionSide,
    pub quantity: f64,
    pub entry_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub commission_paid: f64,
    pub open_time: i64,
    pub bars_held: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PositionSide {
    Long,
    Short,
}

impl Position {
    pub fn new_long(symbol: &str, quantity: f64, entry_price: f64, time: i64) -> Self {
        Self {
            symbol: symbol.to_string(),
            side: PositionSide::Long,
            quantity,
            entry_price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            commission_paid: 0.0,
            open_time: time,
            bars_held: 0,
        }
    }

    pub fn new_short(symbol: &str, quantity: f64, entry_price: f64, time: i64) -> Self {
        Self {
            symbol: symbol.to_string(),
            side: PositionSide::Short,
            quantity,
            entry_price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            commission_paid: 0.0,
            open_time: time,
            bars_held: 0,
        }
    }

    pub fn market_value(&self, current_price: f64) -> f64 {
        self.quantity * current_price
    }

    pub fn update_unrealized(&mut self, current_price: f64) {
        self.unrealized_pnl = match self.side {
            PositionSide::Long => (current_price - self.entry_price) * self.quantity,
            PositionSide::Short => (self.entry_price - current_price) * self.quantity,
        };
    }

    pub fn close(&mut self, exit_price: f64, commission: f64) -> f64 {
        let pnl = match self.side {
            PositionSide::Long => (exit_price - self.entry_price) * self.quantity,
            PositionSide::Short => (self.entry_price - exit_price) * self.quantity,
        };
        self.commission_paid += commission;
        self.quantity = 0.0;
        pnl
    }

    pub fn is_open(&self) -> bool {
        self.quantity > 0.0
    }
}
