use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedOrder {
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: f64,
    pub stop_price: Option<f64>,
    pub status: OrderStatus,
    pub filled_qty: f64,
    pub filled_price: f64,
    pub commission: f64,
    pub commission_asset: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub reduce_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopMarket,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Canceled,
    Rejected,
}

impl SimulatedOrder {
    pub fn new_market_buy(symbol: &str, quantity: f64) -> Self {
        Self {
            order_id: format!("sim-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()),
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity,
            price: 0.0,
            stop_price: None,
            status: OrderStatus::Pending,
            filled_qty: 0.0,
            filled_price: 0.0,
            commission: 0.0,
            commission_asset: "USDT".to_string(),
            created_at: 0,
            updated_at: 0,
            reduce_only: false,
        }
    }

    pub fn new_market_sell(symbol: &str, quantity: f64, reduce_only: bool) -> Self {
        Self {
            order_id: format!("sim-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()),
            symbol: symbol.to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            quantity,
            price: 0.0,
            stop_price: None,
            status: OrderStatus::Pending,
            filled_qty: 0.0,
            filled_price: 0.0,
            commission: 0.0,
            commission_asset: "USDT".to_string(),
            created_at: 0,
            updated_at: 0,
            reduce_only,
        }
    }

    pub fn new_limit_buy(symbol: &str, quantity: f64, price: f64) -> Self {
        Self {
            order_id: format!("sim-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()),
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity,
            price,
            stop_price: None,
            status: OrderStatus::Pending,
            filled_qty: 0.0,
            filled_price: 0.0,
            commission: 0.0,
            commission_asset: "USDT".to_string(),
            created_at: 0,
            updated_at: 0,
            reduce_only: false,
        }
    }

    pub fn new_stop_market_sell(symbol: &str, quantity: f64, stop_price: f64) -> Self {
        Self {
            order_id: format!("sim-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()),
            symbol: symbol.to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::StopMarket,
            quantity,
            price: 0.0,
            stop_price: Some(stop_price),
            status: OrderStatus::Pending,
            filled_qty: 0.0,
            filled_price: 0.0,
            commission: 0.0,
            commission_asset: "USDT".to_string(),
            created_at: 0,
            updated_at: 0,
            reduce_only: true,
        }
    }
}
