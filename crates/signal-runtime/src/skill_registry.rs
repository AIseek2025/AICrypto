use aicrypto_protocols::skill_spec::{SkillFamily, SkillSpec, SkillStatus};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing;

#[derive(Debug, Clone, Deserialize)]
pub struct SkillYaml {
    pub skill_id: String,
    pub skill_name: String,
    pub skill_family: String,
    pub version: String,
    pub status: String,
    pub entry_conditions: EntryConditions,
    pub position_rules: serde_json::Value,
    pub add_rules: Option<serde_json::Value>,
    pub exit_rules: serde_json::Value,
    pub risk_rules: serde_json::Value,
    pub applicable_market_states: Vec<String>,
    pub input_contract: serde_json::Value,
    pub output_contract: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntryConditions {
    pub description: String,
    #[serde(default)]
    pub checks: Vec<ConditionCheck>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConditionCheck {
    #[serde(rename = "type")]
    pub check_type: String,
    pub field: Option<String>,
    pub operator: Option<String>,
    pub threshold: Option<f64>,
    pub reference: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RegisteredSkill {
    pub spec: SkillSpec,
    pub checks: Vec<ConditionCheck>,
    pub output_contract: serde_json::Value,
}

pub struct SkillRegistry {
    skills: HashMap<String, RegisteredSkill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let mut registry = Self::new();
        let families = ["trend", "short", "correlation", "risk"];

        for family in &families {
            let family_dir = dir.join(family);
            if !family_dir.exists() {
                tracing::warn!(family = family, "skills directory not found, skipping");
                continue;
            }

            let entries = std::fs::read_dir(&family_dir)
                .with_context(|| format!("reading skills dir: {}", family_dir.display()))?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext != "yaml" && ext != "yml" {
                        continue;
                    }
                } else {
                    continue;
                }

                match registry.load_yaml(&path) {
                    Ok(id) => tracing::info!(skill = %id, path = %path.display(), "loaded skill"),
                    Err(e) => tracing::error!(path = %path.display(), error = %e, "failed to load skill"),
                }
            }
        }

        tracing::info!(total = registry.skills.len(), "skill registry loaded");
        Ok(registry)
    }

    fn load_yaml(&mut self, path: &Path) -> Result<String> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let yaml: SkillYaml = serde_yaml::from_str(&content)
            .with_context(|| format!("parsing {}", path.display()))?;

        let skill_id = yaml.skill_id.clone();
        let checks = yaml.entry_conditions.checks.clone();

        let family = match yaml.skill_family.to_lowercase().as_str() {
            "trend" => SkillFamily::Trend,
            "short" => SkillFamily::Short,
            "correlation" => SkillFamily::Correlation,
            "risk" => SkillFamily::Risk,
            other => {
                tracing::warn!(family = other, "unknown skill family, defaulting to Trend");
                SkillFamily::Trend
            }
        };

        let status = match yaml.status.to_lowercase().as_str() {
            "draft" => SkillStatus::Draft,
            "backtest_passed" => SkillStatus::BacktestPassed,
            "paper_approved" => SkillStatus::PaperApproved,
            "live" => SkillStatus::Live,
            "disabled" => SkillStatus::Disabled,
            _ => SkillStatus::Draft,
        };

        let spec = SkillSpec {
            skill_id: yaml.skill_id,
            skill_name: yaml.skill_name,
            skill_family: family,
            entry_conditions: serde_json::to_value(&yaml.entry_conditions)
                .unwrap_or(serde_json::Value::Null),
            position_rules: yaml.position_rules,
            add_rules: yaml.add_rules,
            reduce_rules: None,
            exit_rules: yaml.exit_rules,
            risk_rules: yaml.risk_rules,
            applicable_market_states: yaml.applicable_market_states,
            input_contract: yaml.input_contract,
            output_contract: yaml.output_contract.clone(),
            status,
            version: yaml.version,
        };

        let registered = RegisteredSkill {
            spec,
            checks,
            output_contract: yaml.output_contract,
        };

        self.skills.insert(skill_id.clone(), registered);
        Ok(skill_id)
    }

    pub fn get(&self, skill_id: &str) -> Option<&RegisteredSkill> {
        self.skills.get(skill_id)
    }

    pub fn find_by_family(&self, family: &SkillFamily) -> Vec<&RegisteredSkill> {
        self.skills
            .values()
            .filter(|s| {
                std::mem::discriminant(&s.spec.skill_family) == std::mem::discriminant(family)
                    && matches!(s.spec.status, SkillStatus::BacktestPassed | SkillStatus::PaperApproved | SkillStatus::Live)
            })
            .collect()
    }

    pub fn find_by_market_state(&self, state: &str) -> Vec<&RegisteredSkill> {
        self.skills
            .values()
            .filter(|s| {
                s.spec.applicable_market_states.iter().any(|ms| ms == state)
                    && matches!(s.spec.status, SkillStatus::BacktestPassed | SkillStatus::PaperApproved | SkillStatus::Live)
            })
            .collect()
    }

    pub fn all_skills(&self) -> Vec<&RegisteredSkill> {
        self.skills.values().collect()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    pub fn register(&mut self, skill: RegisteredSkill) {
        self.skills.insert(skill.spec.skill_id.clone(), skill);
    }
}

pub fn evaluate_conditions(
    checks: &[ConditionCheck],
    features: &serde_json::Map<String, serde_json::Value>,
) -> bool {
    checks.iter().all(|check| {
        let field_name = match (&check.field, &check.reference) {
            (Some(f), _) => f,
            (None, Some(r)) => r,
            _ => return false,
        };

        let feature_val = match features.get(field_name) {
            Some(v) => v,
            None => return false,
        };

        let feature_f64 = match feature_val {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
            serde_json::Value::Bool(b) => {
                if *b { 1.0 } else { 0.0 }
            }
            _ => return false,
        };

        let threshold = match check.threshold {
            Some(t) => t,
            None => return false,
        };

        let operator = match &check.operator {
            Some(op) => op.as_str(),
            None => return false,
        };

        match operator {
            "gt" => feature_f64 > threshold,
            "lt" => feature_f64 < threshold,
            "gte" => feature_f64 >= threshold,
            "lte" => feature_f64 <= threshold,
            "eq" => (feature_f64 - threshold).abs() < 1e-9,
            "ne" => (feature_f64 - threshold).abs() >= 1e-9,
            other => {
                tracing::warn!(operator = other, "unknown operator, check fails");
                false
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_conditions_gt_lt() {
        let mut features = serde_json::Map::new();
        features.insert("rsi_14".into(), serde_json::json!(65.0));
        features.insert("ma20_slope".into(), serde_json::json!(0.05));
        features.insert("volume_ratio_5".into(), serde_json::json!(2.0));

        let checks = vec![
            ConditionCheck {
                check_type: "trend".into(),
                field: Some("ma20_slope".into()),
                operator: Some("gt".into()),
                threshold: Some(0.0),
                reference: None,
            },
            ConditionCheck {
                check_type: "volume".into(),
                field: Some("volume_ratio_5".into()),
                operator: Some("gt".into()),
                threshold: Some(1.5),
                reference: None,
            },
            ConditionCheck {
                check_type: "not_overbought".into(),
                field: Some("rsi_14".into()),
                operator: Some("lt".into()),
                threshold: Some(80.0),
                reference: None,
            },
        ];

        assert!(evaluate_conditions(&checks, &features));
    }

    #[test]
    fn test_evaluate_conditions_fails() {
        let mut features = serde_json::Map::new();
        features.insert("rsi_14".into(), serde_json::json!(85.0));

        let checks = vec![
            ConditionCheck {
                check_type: "not_overbought".into(),
                field: Some("rsi_14".into()),
                operator: Some("lt".into()),
                threshold: Some(80.0),
                reference: None,
            },
        ];

        assert!(!evaluate_conditions(&checks, &features));
    }

    #[test]
    fn test_evaluate_conditions_missing_field() {
        let features = serde_json::Map::new();
        let checks = vec![
            ConditionCheck {
                check_type: "test".into(),
                field: Some("nonexistent".into()),
                operator: Some("gt".into()),
                threshold: Some(0.0),
                reference: None,
            },
        ];

        assert!(!evaluate_conditions(&checks, &features));
    }

    #[test]
    fn test_evaluate_conditions_bool_field() {
        let mut features = serde_json::Map::new();
        features.insert("breakout_high_20d".into(), serde_json::json!(true));

        let checks = vec![
            ConditionCheck {
                check_type: "breakout".into(),
                field: Some("breakout_high_20d".into()),
                operator: Some("eq".into()),
                threshold: Some(1.0),
                reference: None,
            },
        ];

        assert!(evaluate_conditions(&checks, &features));
    }

    #[test]
    fn test_load_from_dir() {
        let project_root = std::env::current_dir()
            .expect("get cwd")
            .parent()
            .and_then(|p| p.parent())
            .expect("go up to project root")
            .to_path_buf();
        let skills_dir = project_root.join("skills");

        if !skills_dir.exists() {
            eprintln!("skills dir not found at {}, skipping", skills_dir.display());
            return;
        }

        let registry = SkillRegistry::load_from_dir(&skills_dir).expect("load skills");
        assert!(registry.len() >= 10, "expected at least 10 skills, got {}", registry.len());

        let trend_skills = registry.find_by_family(&SkillFamily::Trend);
        assert!(!trend_skills.is_empty());

        let bull_skills = registry.find_by_market_state("BULL_TREND");
        assert!(!bull_skills.is_empty());
    }
}
