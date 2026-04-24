use crate::state::SimState;
use aicrypto_feature_engine::compute::compute_all_features;
use aicrypto_feature_engine::ohlcv::{OhlcvCandle, OhlcvSeries};
use aicrypto_portfolio_engine::portfolio::PortfolioManager;
use aicrypto_risk_engine::evaluator::RiskEvaluator;
use aicrypto_risk_engine::rules::{RiskState, RuleConfig};
use aicrypto_signal_runtime::signal_engine::SignalEngine;
use aicrypto_signal_runtime::skill_registry::SkillRegistry;
use aicrypto_gateway_trading::executor::TradeExecutor;
use aicrypto_protocols::risk_decision::RiskVerdict;
use std::collections::HashMap;
use std::path::Path;
use tracing;

const SYMBOLS: &[&str] = &["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT"];
const INITIAL_PRICES: &[f64] = &[67000.0, 3500.0, 148.0, 610.0];

pub struct SimMarket {
    pub prices: HashMap<String, f64>,
    pub candles: HashMap<String, Vec<OhlcvCandle>>,
    tick: u64,
}

impl SimMarket {
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        let mut candles = HashMap::new();
        for (i, sym) in SYMBOLS.iter().enumerate() {
            prices.insert(sym.to_string(), INITIAL_PRICES[i]);
            candles.insert(sym.to_string(), Vec::new());
        }
        Self { prices, candles, tick: 0 }
    }

    pub fn tick(&mut self) -> HashMap<String, f64> {
        self.tick += 1;
        let mut rng = rand::rng();

        for (i, sym) in SYMBOLS.iter().enumerate() {
            let base_price = INITIAL_PRICES[i];
            let current = *self.prices.get(*sym).unwrap_or(&base_price);
            let drift = (base_price - current) * 0.0001;
            let vol = base_price * 0.002;
            let noise: f64 = rand::Rng::random_range(&mut rng, -1.0..1.0);
            let new_price = (current + drift + vol * noise).max(base_price * 0.5);

            self.prices.insert(sym.to_string(), new_price);

            let candle = OhlcvCandle {
                time: self.tick as i64 * 3600_000,
                open: current,
                high: current.max(new_price) * 1.001,
                low: current.min(new_price) * 0.999,
                close: new_price,
                volume: 1000.0 + noise.abs() * 5000.0,
                quote_volume: new_price * (1000.0 + noise.abs() * 5000.0),
                trades: 100,
            };
            self.candles.get_mut(*sym).unwrap().push(candle);

            let history = self.candles.get_mut(*sym).unwrap();
            if history.len() > 500 {
                history.drain(0..history.len() - 500);
            }
        }
        self.prices.clone()
    }

    pub fn make_series(&self, symbol: &str) -> Option<OhlcvSeries> {
        let candles = self.candles.get(symbol)?;
        if candles.len() < 30 {
            return None;
        }
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
        let volumes: Vec<f64> = candles.iter().map(|c| c.volume).collect();

        Some(OhlcvSeries {
            symbol: symbol.to_string(),
            interval: "1h".to_string(),
            candles: candles.clone(),
        })
    }
}

pub struct AutoTrader {
    pub state: SimState,
    market: SimMarket,
    signal_engine: SignalEngine,
    portfolio_manager: PortfolioManager,
    risk_evaluator: RiskEvaluator,
    executor: TradeExecutor,
    state_path: std::path::PathBuf,
}

impl AutoTrader {
    pub fn new(skills_dir: &Path, initial_equity: f64, state_path: &Path) -> anyhow::Result<Self> {
        let state = if state_path.exists() {
            match SimState::load(state_path) {
                Ok(s) => {
                    tracing::info!(equity = s.account.equity, trades = s.trades.len(), "restored sim state");
                    s
                }
                Err(e) => {
                    tracing::warn!(error = %e, "failed to load state, creating fresh");
                    SimState::new(initial_equity)
                }
            }
        } else {
            SimState::new(initial_equity)
        };

        let registry = SkillRegistry::load_from_dir(skills_dir)?;
        tracing::info!(skills = registry.len(), "auto-trader: skills loaded");

        let signal_engine = SignalEngine::new(registry);
        let portfolio_manager = PortfolioManager::new(state.account.equity, "sim")
            .with_max_risk(0.03)
            .with_max_exposure(0.70)
            .with_max_positions(4);

        let risk_state = RiskState {
            equity: state.account.equity,
            ..Default::default()
        };
        let risk_evaluator = RiskEvaluator::new(RuleConfig::default()).with_state(risk_state);
        let executor = TradeExecutor::new("sim_exchange", true);

        Ok(Self {
            state,
            market: SimMarket::new(),
            signal_engine,
            portfolio_manager,
            risk_evaluator,
            executor,
            state_path: state_path.to_path_buf(),
        })
    }

    pub fn run_cycle(&mut self) -> anyhow::Result<()> {
        self.state.cycle_count += 1;

        let prices = self.market.tick();
        self.state.update_prices(&prices);

        let sl_tp_triggered = self.state.check_stop_loss_take_profit();
        for (sym, reason) in &sl_tp_triggered {
            if let Some(trade) = self.state.close_position(sym, reason) {
                tracing::info!(
                    symbol = %sym, reason = %reason, pnl = trade.realized_pnl,
                    "position closed by {}",
                    if reason == "stop_loss" { "stop-loss" } else { "take-profit" }
                );
                let review = self.state.generate_review_for_trade(&trade);
                tracing::info!(
                    review_id = %review.review_id, outcome = %review.outcome,
                    "trade review generated"
                );
            }
        }

        for sym in SYMBOLS.iter() {
            if let Some(series) = self.market.make_series(sym) {
                if let Some(fv) = compute_all_features(&series) {
                    let signals = self.signal_engine.evaluate(&fv);

                    for signal in &signals {
                        let price = *prices.get(*sym).unwrap_or(&0.0);
                        if price <= 0.0 { continue; }

                        let mut decision = crate::state::DecisionLog::new(
                            "trade", sym,
                            &format!("{:?}", signal.direction), 0.0, price,
                        );
                        decision.confidence = signal.confidence;
                        decision.market_state = format!("{:?}", fv.features.get("market_state")
                            .and_then(|v| v.as_str()).unwrap_or("UNKNOWN"));

                        if let Some(intent) = self.portfolio_manager.process_signal(signal, price) {
                            let risk_decision = self.risk_evaluator.evaluate(&intent);
                            decision.risk_decision = format!("{:?}", risk_decision.decision);
                            decision.risk_rule_hits = risk_decision.rule_hits.iter()
                                .map(|h| format!("{}: {}", h.rule_id, h.rule_name))
                                .collect();

                            match risk_decision.decision {
                                RiskVerdict::Allow => {
                                    let qty: f64 = intent.quantity.parse().unwrap_or(0.0);
                                    decision.quantity = qty;

                                    let leverage = intent.leverage_hint.unwrap_or(3);
                                    let sl_distance = price * 0.03;
                                    let stop_loss = match signal.direction {
                                        aicrypto_protocols::signal_event::Direction::LONG => Some(price - sl_distance),
                                        aicrypto_protocols::signal_event::Direction::SHORT => Some(price + sl_distance),
                                        _ => None,
                                    };
                                    let tp_distance = price * 0.06;
                                    let take_profit = match signal.direction {
                                        aicrypto_protocols::signal_event::Direction::LONG => Some(price + tp_distance),
                                        aicrypto_protocols::signal_event::Direction::SHORT => Some(price - tp_distance),
                                        _ => None,
                                    };

                                    let side = format!("{:?}", signal.direction);
                                    self.state.open_position(
                                        sym, &side, qty, price, leverage, stop_loss, take_profit,
                                    );

                                    decision.thinking = format!(
                                        "基于 {} 策略信号(置信度 {:.0}%), {} {} 数量 {:.4} @ {:.2}. 杠杆 {}x, 止损 {:.2}, 止盈 {:.2}",
                                        signal.reason_codes.join("+"), signal.confidence * 100.0,
                                        side, sym, qty, price, leverage,
                                        stop_loss.unwrap_or(0.0), take_profit.unwrap_or(0.0),
                                    );
                                    tracing::info!(symbol = %sym, side = %side, qty = qty, price = price, "opened position");
                                }
                                RiskVerdict::Shrink => {
                                    let details: Vec<String> = risk_decision.rule_hits.iter()
                                        .map(|h| h.detail.clone()).collect();
                                    decision.thinking = format!(
                                        "信号被风控缩减: {} — 原始数量被缩减50%",
                                        details.join("; ")
                                    );
                                }
                                RiskVerdict::Deny => {
                                    let details: Vec<String> = risk_decision.rule_hits.iter()
                                        .map(|h| h.detail.clone()).collect();
                                    decision.thinking = format!(
                                        "交易被风控拒绝: {}",
                                        details.join("; ")
                                    );
                                }
                                RiskVerdict::Review => {
                                    decision.thinking = "交易需要人工审核，模拟环境自动跳过".to_string();
                                }
                            }
                        } else {
                            decision.thinking = format!(
                                "信号被组合管理器拒绝 — 可能已有仓位或达到上限"
                            );
                        }

                        self.state.decision_logs.push(decision);
                    }
                }
            }
        }

        self.state.record_equity_point();

        if self.state.cycle_count % 10 == 0 {
            self.state.save(&self.state_path)?;
            tracing::debug!("state saved");
        }

        Ok(())
    }

    pub fn portfolio_manager(&self) -> &PortfolioManager {
        &self.portfolio_manager
    }

    pub fn risk_evaluator(&self) -> &RiskEvaluator {
        &self.risk_evaluator
    }
}
