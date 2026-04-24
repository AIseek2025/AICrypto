use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalEvent {
    pub schema_name: String,
    pub schema_version: String,
    pub event_id: String,
    pub trace_id: String,
    pub source_type: SourceType,
    pub source_name: String,
    pub event_type: String,
    pub symbol: Option<String>,
    pub ts_event: i64,
    pub ts_ingested: i64,
    pub payload: serde_json::Value,
    pub quality_flags: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Exchange,
    Social,
    News,
    Onchain,
    System,
}

impl CanonicalEvent {
    pub fn new(
        source_type: SourceType,
        source_name: impl Into<String>,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            schema_name: "canonical_event".to_string(),
            schema_version: "v1".to_string(),
            event_id: Uuid::new_v4().to_string(),
            trace_id: Uuid::new_v4().to_string(),
            source_type,
            source_name: source_name.into(),
            event_type: event_type.into(),
            symbol: None,
            ts_event: now.timestamp_millis(),
            ts_ingested: now.timestamp_millis(),
            payload,
            quality_flags: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = trace_id.into();
        self
    }
}
