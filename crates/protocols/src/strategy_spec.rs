use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySpec {
    pub strategy_id: String,
    pub strategy_name: String,
    pub strategy_type: StrategyType,
    pub owner: String,
    pub input_requirements: serde_json::Value,
    pub signal_model: serde_json::Value,
    pub risk_assumptions: serde_json::Value,
    pub execution_constraints: serde_json::Value,
    pub status: StrategyStatus,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyType {
    Trend,
    MeanReversion,
    EventDriven,
    Correlation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyStatus {
    Draft,
    Testing,
    Paper,
    Live,
    Disabled,
}
