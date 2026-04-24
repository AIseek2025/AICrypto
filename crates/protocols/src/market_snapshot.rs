use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub schema_name: String,
    pub schema_version: String,
    pub symbol: String,
    pub exchange: String,
    pub market_type: String,
    pub last_price: String,
    pub mark_price: Option<String>,
    pub index_price: Option<String>,
    pub funding_rate: Option<String>,
    pub open_interest: Option<String>,
    pub volume_24h: Option<String>,
    pub ts_snapshot: i64,
}

impl MarketSnapshot {
    pub fn new(symbol: impl Into<String>, exchange: impl Into<String>) -> Self {
        Self {
            schema_name: "market_snapshot".to_string(),
            schema_version: "v1".to_string(),
            symbol: symbol.into(),
            exchange: exchange.into(),
            market_type: "usds_m_futures".to_string(),
            last_price: "0".to_string(),
            mark_price: None,
            index_price: None,
            funding_rate: None,
            open_interest: None,
            volume_24h: None,
            ts_snapshot: 0,
        }
    }
}
