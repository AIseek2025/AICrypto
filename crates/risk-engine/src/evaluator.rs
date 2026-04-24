use crate::rules::{RiskState, RuleConfig, RuleEngine};
use aicrypto_protocols::order_intent::OrderIntent;
use aicrypto_protocols::risk_decision::{RiskDecision, RiskVerdict, Severity};
use chrono::Utc;
use tracing;
use uuid::Uuid;

pub struct RiskEvaluator {
    engine: RuleEngine,
    state: RiskState,
}

impl RiskEvaluator {
    pub fn new(config: RuleConfig) -> Self {
        Self {
            engine: RuleEngine::new(config),
            state: RiskState::default(),
        }
    }

    pub fn with_state(mut self, state: RiskState) -> Self {
        self.state = state;
        self
    }

    pub fn evaluate(&mut self, intent: &OrderIntent) -> RiskDecision {
        tracing::info!(
            intent_id = %intent.intent_id,
            symbol = %intent.symbol,
            side = ?intent.side,
            order_type = ?intent.order_type,
            quantity = %intent.quantity,
            reduce_only = intent.reduce_only,
            "evaluating order intent"
        );

        if intent.reduce_only {
            let hits = self.engine.evaluate(intent, &self.state);
            let critical_hits: Vec<_> = hits.iter()
                .filter(|h| h.rule_id == "R001" || h.rule_id == "R004")
                .collect();

            if !critical_hits.is_empty() {
                tracing::warn!(
                    intent_id = %intent.intent_id,
                    hits = critical_hits.len(),
                    "reduce-only order blocked by critical risk rules"
                );
                return RiskDecision {
                    decision_id: Uuid::new_v4().to_string(),
                    target_ref: intent.intent_id.clone(),
                    decision: RiskVerdict::Deny,
                    severity: Severity::Critical,
                    rule_hits: hits,
                    exposure_snapshot: None,
                    required_actions: vec![],
                    review_required: false,
                    ts_decision: Utc::now().timestamp_millis(),
                };
            }

            tracing::info!(intent_id = %intent.intent_id, "reduce-only order, auto-approved");
            return RiskDecision {
                decision_id: Uuid::new_v4().to_string(),
                target_ref: intent.intent_id.clone(),
                decision: RiskVerdict::Allow,
                severity: Severity::Info,
                rule_hits: vec![],
                exposure_snapshot: None,
                required_actions: vec![],
                review_required: false,
                ts_decision: Utc::now().timestamp_millis(),
            };
        }

        let hits = self.engine.evaluate(intent, &self.state);

        let (decision, severity) = if hits.is_empty() {
            (RiskVerdict::Allow, Severity::Info)
        } else {
            let has_critical = hits.iter().any(|h| {
                h.rule_id == "R001" || h.rule_id == "R004"
            });

            if has_critical {
                (RiskVerdict::Deny, Severity::Critical)
            } else {
                let has_warning = hits.iter().any(|h| {
                    h.rule_id == "R003" || h.rule_id == "R007"
                });

                if has_warning {
                    (RiskVerdict::Shrink, Severity::Warning)
                } else {
                    (RiskVerdict::Review, Severity::Warning)
                }
            }
        };

        let required_actions = if decision == RiskVerdict::Shrink {
            vec!["reduce_position_size".to_string()]
        } else if decision == RiskVerdict::Review {
            vec!["manual_review_required".to_string()]
        } else {
            vec![]
        };

        let review_required = decision == RiskVerdict::Review;

        let exposure = serde_json::json!({
            "total_exposure": self.state.total_exposure,
            "open_positions": self.state.open_positions.len(),
            "daily_pnl": self.state.daily_pnl,
            "equity": self.state.equity,
        });

        tracing::info!(
            intent_id = %intent.intent_id,
            decision = ?decision,
            severity = ?severity,
            rule_hits = hits.len(),
            "risk evaluation complete"
        );

        RiskDecision {
            decision_id: Uuid::new_v4().to_string(),
            target_ref: intent.intent_id.clone(),
            decision,
            severity,
            rule_hits: hits,
            exposure_snapshot: Some(exposure),
            required_actions,
            review_required,
            ts_decision: Utc::now().timestamp_millis(),
        }
    }

    pub fn update_state(&mut self, state: RiskState) {
        self.state = state;
    }

    pub fn state(&self) -> &RiskState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aicrypto_protocols::order_intent::*;

    fn make_intent(leverage: Option<u32>, reduce_only: bool) -> OrderIntent {
        OrderIntent {
            intent_id: "test-001".to_string(),
            account_scope: "main".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: Side::BUY,
            position_side: PositionSide::LONG,
            order_type: OrderType::MARKET,
            quantity: "0.01".to_string(),
            price_limit: Some("65000".to_string()),
            reduce_only,
            leverage_hint: leverage,
            take_profit_hint: None,
            stop_loss_hint: None,
            time_in_force: TimeInForce::IOC,
            origin_ref: "sig-001".to_string(),
            ts_intent: 1700000000,
        }
    }

    #[test]
    fn test_reduce_only_auto_approved() {
        let mut evaluator = RiskEvaluator::new(RuleConfig::default());
        let intent = make_intent(Some(3), true);

        let decision = evaluator.evaluate(&intent);
        assert_eq!(decision.decision, RiskVerdict::Allow);
    }

    #[test]
    fn test_reduce_only_blocked_by_critical_rule() {
        let mut evaluator = RiskEvaluator::new(RuleConfig::default());
        let intent = make_intent(Some(10), true);

        let decision = evaluator.evaluate(&intent);
        assert_eq!(decision.decision, RiskVerdict::Deny);
    }

    #[test]
    fn test_deny_excessive_leverage() {
        let mut evaluator = RiskEvaluator::new(RuleConfig::default());
        let intent = make_intent(Some(10), false);

        let decision = evaluator.evaluate(&intent);
        assert_eq!(decision.decision, RiskVerdict::Deny);
    }

    #[test]
    fn test_allow_normal_order() {
        let mut state = RiskState::default();
        state.equity = 100000.0;
        let mut evaluator = RiskEvaluator::new(RuleConfig::default()).with_state(state);

        let intent = make_intent(Some(3), false);
        let decision = evaluator.evaluate(&intent);
        assert_eq!(decision.decision, RiskVerdict::Allow);
    }

    #[test]
    fn test_deny_daily_loss_exceeded() {
        let mut state = RiskState::default();
        state.equity = 100000.0;
        state.daily_pnl = -5000.0;
        let mut evaluator = RiskEvaluator::new(RuleConfig::default()).with_state(state);

        let intent = make_intent(Some(2), false);
        let decision = evaluator.evaluate(&intent);
        assert_eq!(decision.decision, RiskVerdict::Deny);
    }

    #[test]
    fn test_shrink_excessive_exposure() {
        let mut state = RiskState::default();
        state.total_exposure = 199500.0;
        state.equity = 100000.0;
        let mut evaluator = RiskEvaluator::new(RuleConfig::default()).with_state(state);

        let intent = make_intent(Some(2), false);
        let decision = evaluator.evaluate(&intent);
        assert_eq!(decision.decision, RiskVerdict::Shrink);
        assert!(decision.required_actions.contains(&"reduce_position_size".to_string()));
    }
}
