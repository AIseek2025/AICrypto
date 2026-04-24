use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSpec {
    pub skill_id: String,
    pub skill_name: String,
    pub skill_family: SkillFamily,
    pub entry_conditions: serde_json::Value,
    pub position_rules: serde_json::Value,
    pub add_rules: Option<serde_json::Value>,
    pub reduce_rules: Option<serde_json::Value>,
    pub exit_rules: serde_json::Value,
    pub risk_rules: serde_json::Value,
    pub applicable_market_states: Vec<String>,
    pub input_contract: serde_json::Value,
    pub output_contract: serde_json::Value,
    pub status: SkillStatus,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillFamily {
    Trend,
    Short,
    Correlation,
    Risk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillStatus {
    Draft,
    BacktestPassed,
    PaperApproved,
    Live,
    Disabled,
}
