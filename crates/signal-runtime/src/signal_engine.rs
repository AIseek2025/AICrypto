use crate::market_state;
use crate::skill_registry::{evaluate_conditions, SkillRegistry};
use aicrypto_protocols::feature_vector::FeatureVector;
use aicrypto_protocols::signal_event::{Direction, Horizon, SignalEvent, SignalType};
use chrono::Utc;
use tracing;
use uuid::Uuid;

pub struct SignalEngine {
    registry: SkillRegistry,
}

impl SignalEngine {
    pub fn new(registry: SkillRegistry) -> Self {
        Self { registry }
    }

    pub fn evaluate(&self, feature_vector: &FeatureVector) -> Vec<SignalEvent> {
        let market_state = market_state::classify_market_state(&feature_vector.features);
        tracing::debug!(
            symbol = %feature_vector.symbol,
            market_state = %market_state,
            "classified market state"
        );

        let applicable_skills = self.registry.find_by_market_state(&market_state.to_string());
        tracing::debug!(
            symbol = %feature_vector.symbol,
            market_state = %market_state,
            candidate_skills = applicable_skills.len(),
            "found candidate skills"
        );

        let mut signals = Vec::new();

        for skill in applicable_skills {
            if !evaluate_conditions(&skill.checks, &feature_vector.features) {
                tracing::debug!(
                    skill = %skill.spec.skill_id,
                    "conditions not met, skipping"
                );
                continue;
            }

            tracing::info!(
                skill = %skill.spec.skill_id,
                symbol = %feature_vector.symbol,
                market_state = %market_state,
                "skill conditions met, emitting signal"
            );

            let (signal_type, direction, horizon) = parse_output_contract(&skill.output_contract);

            let confidence = compute_confidence(&skill.checks, &feature_vector.features);

            let reason_codes: Vec<String> = skill
                .checks
                .iter()
                .map(|c| c.check_type.clone())
                .collect();

            let evidence_refs: Vec<String> = skill
                .checks
                .iter()
                .filter_map(|c| c.field.as_ref())
                .cloned()
                .collect();

            let signal = SignalEvent {
                signal_id: Uuid::new_v4().to_string(),
                signal_type,
                symbol: feature_vector.symbol.clone(),
                direction,
                confidence,
                horizon,
                reason_codes,
                evidence_refs,
                ts_signal: Utc::now().timestamp_millis(),
            };

            signals.push(signal);
        }

        signals
    }

    pub fn registry(&self) -> &SkillRegistry {
        &self.registry
    }
}

fn parse_output_contract(
    contract: &serde_json::Value,
) -> (SignalType, Direction, Horizon) {
    let signal_type = contract
        .get("signal_type")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "entry" => SignalType::Entry,
            "exit" => SignalType::Exit,
            "add" => SignalType::Add,
            "reduce" => SignalType::Reduce,
            "risk_alert" => SignalType::RiskAlert,
            _ => SignalType::Entry,
        })
        .unwrap_or(SignalType::Entry);

    let direction = contract
        .get("direction")
        .and_then(|v| v.as_str())
        .map(|s| match s.to_uppercase().as_str() {
            "LONG" => Direction::LONG,
            "SHORT" => Direction::SHORT,
            "NEUTRAL" => Direction::NEUTRAL,
            _ => Direction::NEUTRAL,
        })
        .unwrap_or(Direction::NEUTRAL);

    let horizon = contract
        .get("horizon")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "scalp" => Horizon::Scalp,
            "intraday" => Horizon::Intraday,
            "swing" => Horizon::Swing,
            "positional" => Horizon::Positional,
            _ => Horizon::Swing,
        })
        .unwrap_or(Horizon::Swing);

    (signal_type, direction, horizon)
}

fn compute_confidence(
    checks: &[crate::skill_registry::ConditionCheck],
    features: &serde_json::Map<String, serde_json::Value>,
) -> f64 {
    if checks.is_empty() {
        return 0.5;
    }

    let mut strength_sum = 0.0;
    let mut count = 0;

    for check in checks {
        let field_name = match (&check.field, &check.reference) {
            (Some(f), _) => f,
            (None, Some(r)) => r,
            _ => continue,
        };

        let feature_val = match features.get(field_name).and_then(|v| v.as_f64()) {
            Some(v) => v,
            None => continue,
        };

        let threshold = match check.threshold {
            Some(t) => t,
            None => continue,
        };

        if threshold == 0.0 {
            continue;
        }

        let margin = (feature_val - threshold).abs() / threshold.abs();
        let strength = (0.5 + 0.5 * margin.min(1.0)).min(1.0);
        strength_sum += strength;
        count += 1;
    }

    if count == 0 {
        return 0.5;
    }

    (strength_sum / count as f64).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_registry::{ConditionCheck, RegisteredSkill, SkillRegistry};
    use aicrypto_protocols::skill_spec::{SkillFamily, SkillSpec, SkillStatus};
    use serde_json::json;

    fn make_test_registry() -> SkillRegistry {
        let mut registry = SkillRegistry::new();

        let checks = vec![
            ConditionCheck {
                check_type: "trend_alignment".into(),
                field: Some("ma20_slope".into()),
                operator: Some("gt".into()),
                threshold: Some(0.0),
                reference: None,
            },
            ConditionCheck {
                check_type: "volume_confirmation".into(),
                field: Some("volume_ratio_5".into()),
                operator: Some("gt".into()),
                threshold: Some(1.5),
                reference: None,
            },
        ];

        let spec = SkillSpec {
            skill_id: "test_bull_entry".into(),
            skill_name: "Test Bull Entry".into(),
            skill_family: SkillFamily::Trend,
            entry_conditions: json!(null),
            position_rules: json!(null),
            add_rules: None,
            reduce_rules: None,
            exit_rules: json!(null),
            risk_rules: json!(null),
            applicable_market_states: vec!["BULL_TREND".into()],
            input_contract: json!(null),
            output_contract: json!(null),
            status: SkillStatus::Draft,
            version: "v1".into(),
        };

        let output_contract = json!({
            "signal_type": "entry",
            "direction": "LONG",
            "horizon": "swing"
        });

        let registered = RegisteredSkill {
            spec,
            checks,
            output_contract,
        };

        registry.register(registered);
        registry
    }

    #[test]
    fn test_signal_engine_emits_signal() {
        let registry = make_test_registry();
        let engine = SignalEngine::new(registry);

        let mut features = serde_json::Map::new();
        features.insert("ma20_slope".into(), json!(0.005));
        features.insert("rsi_14".into(), json!(60.0));
        features.insert("atr_pct".into(), json!(0.015));
        features.insert("bb_position".into(), json!(0.7));
        features.insert("volume_ratio_5".into(), json!(2.5));
        features.insert("hist_vol_20".into(), json!(0.3));
        features.insert("bb_width".into(), json!(0.05));
        features.insert("price_above_sma20".into(), json!(true));
        features.insert("price_above_sma50".into(), json!(true));
        features.insert("price_return".into(), json!(0.01));

        let fv = FeatureVector {
            schema_name: "feature_vector".into(),
            schema_version: "v1".into(),
            feature_set: "all".into(),
            feature_version: "v1".into(),
            symbol: "BTCUSDT".into(),
            window: "1h".into(),
            ts_feature: 1700000000,
            features,
            source_refs: vec![],
        };

        let signals = engine.evaluate(&fv);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].symbol, "BTCUSDT");
        assert_eq!(signals[0].direction, Direction::LONG);
        assert_eq!(signals[0].signal_type, SignalType::Entry);
    }

    #[test]
    fn test_signal_engine_no_signal_when_conditions_fail() {
        let registry = make_test_registry();
        let engine = SignalEngine::new(registry);

        let mut features = serde_json::Map::new();
        features.insert("ma20_slope".into(), json!(-0.005));
        features.insert("rsi_14".into(), json!(35.0));
        features.insert("atr_pct".into(), json!(0.02));
        features.insert("bb_position".into(), json!(0.2));
        features.insert("volume_ratio_5".into(), json!(0.8));
        features.insert("hist_vol_20".into(), json!(0.4));
        features.insert("bb_width".into(), json!(0.06));
        features.insert("price_above_sma20".into(), json!(false));
        features.insert("price_above_sma50".into(), json!(false));
        features.insert("price_return".into(), json!(-0.01));

        let fv = FeatureVector {
            schema_name: "feature_vector".into(),
            schema_version: "v1".into(),
            feature_set: "all".into(),
            feature_version: "v1".into(),
            symbol: "BTCUSDT".into(),
            window: "1h".into(),
            ts_feature: 1700000000,
            features,
            source_refs: vec![],
        };

        let signals = engine.evaluate(&fv);
        assert!(signals.is_empty());
    }
}
