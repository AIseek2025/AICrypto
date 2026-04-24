use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfoResponse {
    pub timezone: String,
    pub server_time: i64,
    pub rate_limits: Vec<RateLimit>,
    pub symbols: Vec<BinanceSymbolInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub interval_num: Option<i64>,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceSymbolInfo {
    pub symbol: String,
    pub pair: String,
    pub contract_type: Option<String>,
    pub delivery_date: Option<i64>,
    pub onboard_date: Option<i64>,
    pub status: Option<String>,
    pub contract_size: Option<f64>,
    pub margin_asset: Option<String>,
    pub maint_margin_percent: Option<String>,
    pub required_margin_percent: Option<String>,
    pub base_asset: Option<String>,
    pub quote_asset: Option<String>,
    pub price_precision: Option<i32>,
    pub quantity_precision: Option<i32>,
    pub base_asset_precision: Option<i32>,
    pub quote_asset_precision: Option<i32>,
    pub equal_qty_precision: Option<i32>,
    pub filters: Vec<serde_json::Value>,
    pub order_types: Option<Vec<String>>,
    pub time_in_force: Option<Vec<String>>,
    pub liquidation_fee: Option<String>,
    pub market_take_bound: Option<String>,
}

impl BinanceSymbolInfo {
    pub fn tick_size(&self) -> Option<f64> {
        self.filters.iter().find_map(|f| {
            if f.get("filterType").and_then(|v| v.as_str()) == Some("PRICE_FILTER") {
                f.get("tickSize").and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
            } else {
                None
            }
        })
    }

    pub fn step_size(&self) -> Option<f64> {
        self.filters.iter().find_map(|f| {
            if f.get("filterType").and_then(|v| v.as_str()) == Some("LOT_SIZE") {
                f.get("stepSize").and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
            } else {
                None
            }
        })
    }

    pub fn min_qty(&self) -> Option<f64> {
        self.filters.iter().find_map(|f| {
            if f.get("filterType").and_then(|v| v.as_str()) == Some("MARKET_LOT_SIZE") {
                f.get("minQty").and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
            } else {
                None
            }
        })
    }

    pub fn min_notional(&self) -> Option<f64> {
        self.filters.iter().find_map(|f| {
            if f.get("filterType").and_then(|v| v.as_str()) == Some("MIN_NOTIONAL") {
                f.get("notional").and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
            } else {
                None
            }
        })
    }

    pub fn max_leverage(&self) -> Option<i32> {
        self.filters.iter().find_map(|f| {
            if f.get("filterType").and_then(|v| v.as_str()) == Some("MAX_NUM_POSITIONS") {
                None
            } else if f.get("filterType").and_then(|v| v.as_str()) == Some("LEVERAGE") {
                f.get("maxLeverage").and_then(|v| v.as_i64()).map(|v| v as i32)
            } else {
                None
            }
        })
    }

    pub fn is_trading(&self) -> bool {
        self.status.as_deref() == Some("TRADING")
    }
}

#[derive(Debug, Clone)]
pub struct BinanceKline {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub close_time: i64,
    pub quote_volume: String,
    pub trades: i64,
    pub taker_buy_volume: String,
}

impl BinanceKline {
    pub fn from_raw(raw: &[serde_json::Value]) -> Option<Self> {
        Some(Self {
            open_time: raw.get(0)?.as_i64()?,
            open: raw.get(1)?.as_str()?.to_string(),
            high: raw.get(2)?.as_str()?.to_string(),
            low: raw.get(3)?.as_str()?.to_string(),
            close: raw.get(4)?.as_str()?.to_string(),
            volume: raw.get(5)?.as_str()?.to_string(),
            close_time: raw.get(6)?.as_i64()?,
            quote_volume: raw.get(7)?.as_str()?.to_string(),
            trades: raw.get(8)?.as_i64()?,
            taker_buy_volume: raw.get(9)?.as_str()?.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkPriceResponse {
    pub symbol: String,
    pub mark_price: String,
    pub index_price: String,
    pub estimated_settle_price: Option<String>,
    pub last_funding_rate: Option<String>,
    pub next_funding_time: Option<i64>,
    pub interest_rate: Option<String>,
    pub time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingRateResponse {
    pub symbol: String,
    pub funding_rate: String,
    pub funding_time: i64,
    pub mark_price: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenInterestResponse {
    pub symbol: Option<String>,
    pub open_interest: String,
    pub time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker24hrResponse {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub last_price: String,
    pub volume: String,
    pub quote_volume: String,
    pub open_price: Option<String>,
    pub high_price: String,
    pub low_price: String,
    pub weighted_avg_price: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsKlineEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: WsKlineData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsKlineData {
    #[serde(rename = "t")]
    pub start_time: i64,
    #[serde(rename = "T")]
    pub close_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "x")]
    pub is_closed: bool,
    #[serde(rename = "q")]
    pub quote_volume: String,
    #[serde(rename = "n")]
    pub trades: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMarkPriceEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub mark_price: String,
    #[serde(rename = "i")]
    pub index_price: String,
    #[serde(rename = "r")]
    pub funding_rate: Option<String>,
    #[serde(rename = "T")]
    pub next_funding_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAggTradeEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "q")]
    pub quantity: String,
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsTickerEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub last_price: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "q")]
    pub quote_volume: String,
    #[serde(rename = "h")]
    pub high_price: String,
    #[serde(rename = "l")]
    pub low_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsDepthEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "U")]
    pub first_update_id: i64,
    #[serde(rename = "u")]
    pub final_update_id: i64,
    #[serde(rename = "b")]
    pub bids: Vec<Vec<String>>,
    #[serde(rename = "a")]
    pub asks: Vec<Vec<String>>,
}
