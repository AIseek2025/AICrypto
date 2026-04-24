use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderIntent {
    pub intent_id: String,
    pub account_scope: String,
    pub symbol: String,
    pub side: Side,
    pub position_side: PositionSide,
    pub order_type: OrderType,
    pub quantity: String,
    pub price_limit: Option<String>,
    pub reduce_only: bool,
    pub leverage_hint: Option<u32>,
    pub take_profit_hint: Option<TakeProfitHint>,
    pub stop_loss_hint: Option<StopLossHint>,
    pub time_in_force: TimeInForce,
    pub origin_ref: String,
    pub ts_intent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PositionSide {
    LONG,
    SHORT,
    BOTH,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(non_camel_case_types)]
pub enum OrderType {
    LIMIT,
    MARKET,
    STOP,
    STOP_MARKET,
    TAKE_PROFIT,
    TAKE_PROFIT_MARKET,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
    GTX,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitHint {
    pub tp_type: String,
    pub activation_price: Option<String>,
    pub callback_rate: Option<String>,
    pub stop_price: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossHint {
    pub sl_type: String,
    pub stop_price: String,
}
