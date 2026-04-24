use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub schema_name: String,
    pub schema_version: String,
    pub feature_set: String,
    pub feature_version: String,
    pub symbol: String,
    pub window: String,
    pub ts_feature: i64,
    pub features: serde_json::Map<String, serde_json::Value>,
    pub source_refs: Vec<String>,
}

impl FeatureVector {
    pub fn new(
        feature_set: impl Into<String>,
        symbol: impl Into<String>,
        window: impl Into<String>,
    ) -> Self {
        Self {
            schema_name: "feature_vector".to_string(),
            schema_version: "v1".to_string(),
            feature_set: feature_set.into(),
            feature_version: "v1".to_string(),
            symbol: symbol.into(),
            window: window.into(),
            ts_feature: 0,
            features: serde_json::Map::new(),
            source_refs: Vec::new(),
        }
    }

    pub fn with_feature(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.features.insert(key.into(), value);
        self
    }
}
