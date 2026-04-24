use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDecision {
    pub decision_id: String,
    pub target_ref: String,
    pub decision: RiskVerdict,
    pub severity: Severity,
    pub rule_hits: Vec<RuleHit>,
    pub exposure_snapshot: Option<serde_json::Value>,
    pub required_actions: Vec<String>,
    pub review_required: bool,
    pub ts_decision: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskVerdict {
    Allow,
    Deny,
    Review,
    Shrink,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleHit {
    pub rule_id: String,
    pub rule_name: String,
    pub detail: String,
}
