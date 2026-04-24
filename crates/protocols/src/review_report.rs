use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub review_id: String,
    pub review_type: ReviewType,
    pub target_ref: String,
    pub summary: String,
    pub findings: Vec<Finding>,
    pub recommendations: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub reviewer: String,
    pub ts_review: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewType {
    Strategy,
    Skill,
    Daily,
    Weekly,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub category: String,
    pub description: String,
    pub severity: String,
    pub data: Option<serde_json::Value>,
}
