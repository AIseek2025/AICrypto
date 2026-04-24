use aicrypto_feature_engine::compute::compute_all_features;
use aicrypto_feature_engine::ohlcv::OhlcvSeries;
use aicrypto_foundation::bus::{
    publish, deserialize_message,
    SUBJECT_EXECUTION_REPORT, SUBJECT_FEATURE_VECTOR, SUBJECT_ORDER_INTENT,
    SUBJECT_RISK_DECISION, SUBJECT_SIGNAL_EVENT,
};
use aicrypto_gateway_trading::executor::TradeExecutor;
use aicrypto_portfolio_engine::portfolio::PortfolioManager;
use aicrypto_protocols::execution_report::ExecutionReport;
use aicrypto_protocols::feature_vector::FeatureVector;
use aicrypto_protocols::order_intent::OrderIntent;
use aicrypto_protocols::risk_decision::RiskDecision;
use aicrypto_protocols::signal_event::SignalEvent;
use aicrypto_risk_engine::evaluator::RiskEvaluator;
use aicrypto_risk_engine::rules::{RiskState, RuleConfig};
use aicrypto_signal_runtime::signal_engine::SignalEngine;
use aicrypto_signal_runtime::skill_registry::SkillRegistry;
use anyhow::Result;
use futures_util::StreamExt;
use std::path::Path;
use tracing;

pub struct PipelineResult {
    pub feature_vector: Option<FeatureVector>,
    pub signals: Vec<SignalEvent>,
    pub intents: Vec<OrderIntent>,
    pub decisions: Vec<RiskDecision>,
    pub reports: Vec<ExecutionReport>,
}

pub struct Pipeline {
    signal_engine: SignalEngine,
    portfolio_manager: PortfolioManager,
    risk_evaluator: RiskEvaluator,
    executor: TradeExecutor,
}

impl Pipeline {
    pub fn new(skills_dir: &Path, equity: f64) -> Result<Self> {
        let registry = SkillRegistry::load_from_dir(skills_dir)?;
        tracing::info!(skills = registry.len(), "pipeline: skills loaded");

        let signal_engine = SignalEngine::new(registry);
        let portfolio_manager = PortfolioManager::new(equity, "main")
            .with_max_risk(0.02)
            .with_max_exposure(0.80)
            .with_max_positions(5);

        let rule_config = RuleConfig::default();
        let risk_state = RiskState {
            equity,
            ..Default::default()
        };
        let risk_evaluator = RiskEvaluator::new(rule_config).with_state(risk_state);

        let executor = TradeExecutor::new("binance_testnet", true);

        Ok(Self {
            signal_engine,
            portfolio_manager,
            risk_evaluator,
            executor,
        })
    }

    pub fn process_candles(&mut self, series: &OhlcvSeries, current_price: f64) -> Result<PipelineResult> {
        let symbol = &series.symbol;

        tracing::info!("=== Pipeline: {} ===", symbol);

        // Stage 1: Feature computation
        let feature_vector = match compute_all_features(series) {
            Some(fv) => {
                tracing::info!(
                    symbol = symbol,
                    features = fv.features.len(),
                    "Stage 1 ✓ FeatureVector computed"
                );
                Some(fv)
            }
            None => {
                tracing::warn!(symbol = symbol, "Stage 1 ✗ insufficient data for features");
                return Ok(PipelineResult {
                    feature_vector: None,
                    signals: vec![],
                    intents: vec![],
                    decisions: vec![],
                    reports: vec![],
                });
            }
        };

        // Stage 2: Signal evaluation
        let signals = if let Some(ref fv) = feature_vector {
            let sigs = self.signal_engine.evaluate(fv);
            tracing::info!(
                symbol = symbol,
                signals = sigs.len(),
                "Stage 2 ✓ SignalEvent generated"
            );
            sigs
        } else {
            vec![]
        };

        // Stage 3: Portfolio management
        let mut intents = Vec::new();
        for signal in &signals {
            if let Some(intent) = self.portfolio_manager.process_signal(signal, current_price) {
                tracing::info!(
                    intent_id = %intent.intent_id,
                    symbol = %intent.symbol,
                    side = ?intent.side,
                    quantity = %intent.quantity,
                    "Stage 3 ✓ OrderIntent generated"
                );
                intents.push(intent);
            } else {
                tracing::warn!(
                    signal_id = %signal.signal_id,
                    symbol = %signal.symbol,
                    "Stage 3 ✗ signal rejected by portfolio manager"
                );
            }
        }

        // Stage 4: Risk evaluation
        let mut decisions = Vec::new();
        for intent in &intents {
            let decision = self.risk_evaluator.evaluate(intent);
            tracing::info!(
                intent_id = %intent.intent_id,
                decision = ?decision.decision,
                severity = ?decision.severity,
                rule_hits = decision.rule_hits.len(),
                "Stage 4 ✓ RiskDecision made"
            );
            for hit in &decision.rule_hits {
                tracing::warn!(
                    rule_id = %hit.rule_id,
                    rule_name = %hit.rule_name,
                    detail = %hit.detail,
                    "  rule hit"
                );
            }
            decisions.push(decision);
        }

        // Stage 5: Execution
        let mut reports = Vec::new();
        for (intent, decision) in intents.iter().zip(decisions.iter()) {
            match self.executor.submit(intent, decision) {
                Ok(report) => {
                    tracing::info!(
                        report_id = %report.report_id,
                        status = ?report.order_status,
                        filled_qty = ?report.filled_qty,
                        avg_price = ?report.avg_fill_price,
                        "Stage 5 ✓ ExecutionReport generated"
                    );
                    reports.push(report);
                }
                Err(e) => {
                    tracing::error!(
                        intent_id = %intent.intent_id,
                        error = %e,
                        "Stage 5 ✗ execution failed"
                    );
                }
            }
        }

        tracing::info!(
            symbol = symbol,
            signals = signals.len(),
            intents = intents.len(),
            decisions = decisions.len(),
            reports = reports.len(),
            "=== Pipeline complete ==="
        );

        Ok(PipelineResult {
            feature_vector,
            signals,
            intents,
            decisions,
            reports,
        })
    }

    pub fn portfolio_manager(&self) -> &PortfolioManager {
        &self.portfolio_manager
    }

    pub fn risk_evaluator(&self) -> &RiskEvaluator {
        &self.risk_evaluator
    }

    pub fn executor(&self) -> &TradeExecutor {
        &self.executor
    }

    pub fn signal_engine(&self) -> &SignalEngine {
        &self.signal_engine
    }
}

pub async fn run_nats_pipeline(
    nats_url: &str,
    skills_dir: &Path,
    equity: f64,
) -> Result<()> {
    let bus = aicrypto_foundation::bus::BusClient::new(nats_url);
    tracing::info!(nats_url = nats_url, "connecting to NATS...");
    let client = bus.connect().await?;
    tracing::info!("connected to NATS");

    let mut subscriber = aicrypto_foundation::bus::subscribe(&client, SUBJECT_FEATURE_VECTOR).await?;

    let registry = SkillRegistry::load_from_dir(skills_dir)?;
    let signal_engine = SignalEngine::new(registry);
    let mut portfolio_manager = PortfolioManager::new(equity, "main");
    let mut risk_evaluator = RiskEvaluator::new(RuleConfig::default());
    let mut executor = TradeExecutor::new("binance_testnet", true);

    tracing::info!("NATS pipeline ready, waiting for FeatureVector messages...");

    while let Some(msg) = subscriber.next().await {
        match deserialize_message::<FeatureVector>(&msg) {
            Ok(fv) => {
                tracing::info!(symbol = %fv.symbol, "received FeatureVector");

                let signals = signal_engine.evaluate(&fv);
                for signal in signals {
                    let _ = publish(&client, SUBJECT_SIGNAL_EVENT, &signal).await;

                    if let Some(intent) = portfolio_manager.process_signal(&signal, 0.0) {
                        let _ = publish(&client, SUBJECT_ORDER_INTENT, &intent).await;

                        let decision = risk_evaluator.evaluate(&intent);
                        let _ = publish(&client, SUBJECT_RISK_DECISION, &decision).await;

                        if let Ok(report) = executor.submit(&intent, &decision) {
                            let _ = publish(&client, SUBJECT_EXECUTION_REPORT, &report).await;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to deserialize FeatureVector");
            }
        }
    }

    Ok(())
}


