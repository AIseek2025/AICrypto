use aicrypto_protocols::order_intent::OrderIntent;
use aicrypto_protocols::risk_decision::RuleHit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub max_position_notional: f64,
    pub max_total_exposure: f64,
    pub max_leverage: u32,
    pub max_single_risk_pct: f64,
    pub max_daily_loss_pct: f64,
    pub max_correlated_exposure_pct: f64,
    pub cooldown_after_loss_bars: i64,
    pub max_open_orders: usize,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            max_position_notional: 50000.0,
            max_total_exposure: 200000.0,
            max_leverage: 5,
            max_single_risk_pct: 0.05,
            max_daily_loss_pct: 0.03,
            max_correlated_exposure_pct: 0.40,
            cooldown_after_loss_bars: 24,
            max_open_orders: 10,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RiskState {
    pub total_exposure: f64,
    pub open_positions: HashMap<String, f64>,
    pub daily_pnl: f64,
    pub equity: f64,
    pub open_orders: usize,
    pub last_loss_time: Option<i64>,
    pub current_time: i64,
}

pub struct RuleEngine {
    config: RuleConfig,
}

impl RuleEngine {
    pub fn new(config: RuleConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(&self, intent: &OrderIntent, state: &RiskState) -> Vec<RuleHit> {
        let mut hits = Vec::new();

        self.check_leverage(intent, &mut hits);
        self.check_position_size(intent, state, &mut hits);
        self.check_total_exposure(intent, state, &mut hits);
        self.check_daily_loss(state, &mut hits);
        self.check_cooldown(state, &mut hits);
        self.check_open_orders(state, &mut hits);
        self.check_risk_per_trade(intent, state, &mut hits);

        hits
    }

    fn check_leverage(&self, intent: &OrderIntent, hits: &mut Vec<RuleHit>) {
        if let Some(lev) = intent.leverage_hint {
            if lev > self.config.max_leverage {
                hits.push(RuleHit {
                    rule_id: "R001".to_string(),
                    rule_name: "max_leverage".to_string(),
                    detail: format!(
                        "requested leverage {} exceeds max {}",
                        lev, self.config.max_leverage
                    ),
                });
            }
        }
    }

    fn check_position_size(&self, intent: &OrderIntent, state: &RiskState, hits: &mut Vec<RuleHit>) {
        if let Ok(qty) = intent.quantity.parse::<f64>() {
            let price = intent
                .price_limit
                .as_ref()
                .and_then(|p| p.parse::<f64>().ok())
                .unwrap_or(state.equity * 0.01);

            let notional = qty * price;
            let existing = state.open_positions.get(&intent.symbol).copied().unwrap_or(0.0);
            let total_notional = existing + notional;

            if total_notional > self.config.max_position_notional {
                hits.push(RuleHit {
                    rule_id: "R002".to_string(),
                    rule_name: "max_position_notional".to_string(),
                    detail: format!(
                        "position notional ${:.2} exceeds max ${:.2}",
                        total_notional, self.config.max_position_notional
                    ),
                });
            }
        }
    }

    fn check_total_exposure(&self, intent: &OrderIntent, state: &RiskState, hits: &mut Vec<RuleHit>) {
        if let Ok(qty) = intent.quantity.parse::<f64>() {
            let price = intent
                .price_limit
                .as_ref()
                .and_then(|p| p.parse::<f64>().ok())
                .unwrap_or(state.equity * 0.01);

            let additional = qty * price;
            let new_total = state.total_exposure + additional;

            if new_total > self.config.max_total_exposure {
                hits.push(RuleHit {
                    rule_id: "R003".to_string(),
                    rule_name: "max_total_exposure".to_string(),
                    detail: format!(
                        "total exposure ${:.2} would exceed max ${:.2}",
                        new_total, self.config.max_total_exposure
                    ),
                });
            }
        }
    }

    fn check_daily_loss(&self, state: &RiskState, hits: &mut Vec<RuleHit>) {
        if state.equity > 0.0 {
            let daily_loss_pct = state.daily_pnl.abs() / state.equity;
            if state.daily_pnl < 0.0 && daily_loss_pct > self.config.max_daily_loss_pct {
                hits.push(RuleHit {
                    rule_id: "R004".to_string(),
                    rule_name: "max_daily_loss".to_string(),
                    detail: format!(
                        "daily loss {:.2}% exceeds max {:.2}%",
                        daily_loss_pct * 100.0,
                        self.config.max_daily_loss_pct * 100.0
                    ),
                });
            }
        }
    }

    fn check_cooldown(&self, state: &RiskState, hits: &mut Vec<RuleHit>) {
        if let Some(last_loss) = state.last_loss_time {
            let bars_since = state.current_time - last_loss;
            if bars_since < self.config.cooldown_after_loss_bars {
                hits.push(RuleHit {
                    rule_id: "R005".to_string(),
                    rule_name: "cooldown_after_loss".to_string(),
                    detail: format!(
                        "cooldown period active: {} bars remaining",
                        self.config.cooldown_after_loss_bars - bars_since
                    ),
                });
            }
        }
    }

    fn check_open_orders(&self, state: &RiskState, hits: &mut Vec<RuleHit>) {
        if state.open_orders >= self.config.max_open_orders {
            hits.push(RuleHit {
                rule_id: "R006".to_string(),
                rule_name: "max_open_orders".to_string(),
                detail: format!(
                    "open orders {} exceeds max {}",
                    state.open_orders, self.config.max_open_orders
                ),
            });
        }
    }

    fn check_risk_per_trade(&self, intent: &OrderIntent, state: &RiskState, hits: &mut Vec<RuleHit>) {
        if state.equity <= 0.0 {
            return;
        }
        if let Ok(qty) = intent.quantity.parse::<f64>() {
            let price = intent
                .price_limit
                .as_ref()
                .and_then(|p| p.parse::<f64>().ok())
                .unwrap_or(state.equity * 0.01);

            let notional = qty * price;
            let risk_pct = notional / state.equity;

            if risk_pct > self.config.max_single_risk_pct {
                hits.push(RuleHit {
                    rule_id: "R007".to_string(),
                    rule_name: "max_single_risk".to_string(),
                    detail: format!(
                        "single trade risk {:.2}% exceeds max {:.2}%",
                        risk_pct * 100.0,
                        self.config.max_single_risk_pct * 100.0
                    ),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aicrypto_protocols::order_intent::*;

    fn make_intent(side: Side, quantity: &str, price: Option<&str>, leverage: Option<u32>) -> OrderIntent {
        OrderIntent {
            intent_id: "test-001".to_string(),
            account_scope: "main".to_string(),
            symbol: "BTCUSDT".to_string(),
            side,
            position_side: PositionSide::LONG,
            order_type: OrderType::MARKET,
            quantity: quantity.to_string(),
            price_limit: price.map(|p| p.to_string()),
            reduce_only: false,
            leverage_hint: leverage,
            take_profit_hint: None,
            stop_loss_hint: None,
            time_in_force: TimeInForce::IOC,
            origin_ref: "sig-001".to_string(),
            ts_intent: 1700000000,
        }
    }

    #[test]
    fn test_leverage_rule() {
        let engine = RuleEngine::new(RuleConfig::default());
        let state = RiskState::default();
        let intent = make_intent(Side::BUY, "0.1", None, Some(10));

        let hits = engine.evaluate(&intent, &state);
        let leverage_hit = hits.iter().find(|h| h.rule_id == "R001");
        assert!(leverage_hit.is_some());
    }

    #[test]
    fn test_leverage_ok() {
        let engine = RuleEngine::new(RuleConfig::default());
        let state = RiskState::default();
        let intent = make_intent(Side::BUY, "0.1", Some("65000"), Some(3));

        let hits = engine.evaluate(&intent, &state);
        let leverage_hit = hits.iter().find(|h| h.rule_id == "R001");
        assert!(leverage_hit.is_none());
    }

    #[test]
    fn test_daily_loss_rule() {
        let engine = RuleEngine::new(RuleConfig::default());
        let mut state = RiskState::default();
        state.equity = 100000.0;
        state.daily_pnl = -5000.0;

        let intent = make_intent(Side::BUY, "0.01", Some("65000"), Some(2));
        let hits = engine.evaluate(&intent, &state);
        let daily_hit = hits.iter().find(|h| h.rule_id == "R004");
        assert!(daily_hit.is_some());
    }

    #[test]
    fn test_total_exposure_rule() {
        let engine = RuleEngine::new(RuleConfig::default());
        let mut state = RiskState::default();
        state.total_exposure = 190000.0;

        let intent = make_intent(Side::BUY, "1.0", Some("65000"), Some(2));
        let hits = engine.evaluate(&intent, &state);
        let exposure_hit = hits.iter().find(|h| h.rule_id == "R003");
        assert!(exposure_hit.is_some());
    }

    #[test]
    fn test_cooldown_rule() {
        let engine = RuleEngine::new(RuleConfig::default());
        let mut state = RiskState::default();
        state.current_time = 100;
        state.last_loss_time = Some(90);

        let intent = make_intent(Side::BUY, "0.01", Some("65000"), Some(2));
        let hits = engine.evaluate(&intent, &state);
        let cooldown_hit = hits.iter().find(|h| h.rule_id == "R005");
        assert!(cooldown_hit.is_some());
    }

    #[test]
    fn test_all_clear() {
        let engine = RuleEngine::new(RuleConfig::default());
        let mut state = RiskState::default();
        state.equity = 100000.0;

        let intent = make_intent(Side::BUY, "0.01", Some("65000"), Some(2));
        let hits = engine.evaluate(&intent, &state);
        assert!(hits.is_empty());
    }
}
