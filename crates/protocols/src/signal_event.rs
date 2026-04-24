use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEvent {
    pub signal_id: String,
    pub signal_type: SignalType,
    pub symbol: String,
    pub direction: Direction,
    pub confidence: f64,
    pub horizon: Horizon,
    pub reason_codes: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub ts_signal: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    Entry,
    Exit,
    Add,
    Reduce,
    RiskAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction {
    LONG,
    SHORT,
    NEUTRAL,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Horizon {
    Scalp,
    Intraday,
    Swing,
    Positional,
}
