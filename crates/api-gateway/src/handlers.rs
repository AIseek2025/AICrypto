use aicrypto_feature_engine::ohlcv::{OhlcvCandle, OhlcvSeries};
use aicrypto_pipeline_integration::pipeline::Pipeline;
use aicrypto_portfolio_engine::position_tracker::PositionSide;
use aicrypto_protocols::execution_report::OrderStatus;
use aicrypto_protocols::risk_decision::RiskVerdict;
use aicrypto_protocols::signal_event::SignalEvent;
use aicrypto_signal_runtime::skill_registry::SkillRegistry;
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub pipeline: Pipeline,
    pub signal_history: Vec<SignalEvent>,
    pub risk_history: Vec<RiskDecisionRecord>,
    pub execution_history: Vec<ExecutionRecord>,
}

impl AppState {
    pub fn new(skills_dir: &PathBuf) -> Result<Self> {
        let pipeline = Pipeline::new(skills_dir, 100000.0)?;
        Ok(Self {
            pipeline,
            signal_history: Vec::new(),
            risk_history: Vec::new(),
            execution_history: Vec::new(),
        })
    }
}

pub type SharedState = Arc<RwLock<AppState>>;

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/portfolio", get(get_portfolio))
        .route("/api/portfolio/positions", get(get_positions))
        .route("/api/skills", get(get_skills))
        .route("/api/skills/{id}", get(get_skill))
        .route("/api/signals", get(get_signals))
        .route("/api/risk/events", get(get_risk_events))
        .route("/api/risk/rules", get(get_risk_rules))
        .route("/api/executions", get(get_executions))
        .route("/api/run-pipeline", post(run_pipeline))
        .route("/api/market/{symbol}/evaluate", post(evaluate_symbol))
        .with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_secs: u64,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
        uptime_secs: 0,
    })
}

#[derive(Serialize)]
struct PortfolioResponse {
    equity: f64,
    total_exposure: f64,
    unrealized_pnl: f64,
    realized_pnl: f64,
    open_positions: usize,
}

async fn get_portfolio(State(state): State<SharedState>) -> Json<PortfolioResponse> {
    let s = state.read().await;
    let pm = s.pipeline.portfolio_manager();
    Json(PortfolioResponse {
        equity: pm.equity(),
        total_exposure: pm.tracker().total_exposure(),
        unrealized_pnl: pm.tracker().total_unrealized_pnl(),
        realized_pnl: pm.tracker().total_realized_pnl(),
        open_positions: pm.tracker().open_position_count(),
    })
}

#[derive(Serialize)]
struct PositionResponse {
    symbol: String,
    side: String,
    quantity: f64,
    entry_price: f64,
    leverage: u32,
    unrealized_pnl: f64,
}

async fn get_positions(State(state): State<SharedState>) -> Json<Vec<PositionResponse>> {
    let s = state.read().await;
    let pm = s.pipeline.portfolio_manager();
    Json(
        pm.tracker()
            .all_positions()
            .iter()
            .map(|p| PositionResponse {
                symbol: p.symbol.clone(),
                side: format!("{:?}", p.side),
                quantity: p.quantity,
                entry_price: p.entry_price,
                leverage: p.leverage,
                unrealized_pnl: p.unrealized_pnl,
            })
            .collect(),
    )
}

#[derive(Serialize)]
struct SkillResponse {
    skill_id: String,
    skill_name: String,
    skill_family: String,
    status: String,
    direction: String,
    market_states: Vec<String>,
}

async fn get_skills(State(state): State<SharedState>) -> Json<Vec<SkillResponse>> {
    let s = state.read().await;
    let engine = s.pipeline.signal_engine();
    let registry = engine.registry();
    Json(
        registry
            .all_skills()
            .iter()
            .map(|sk| SkillResponse {
                skill_id: sk.spec.skill_id.clone(),
                skill_name: sk.spec.skill_name.clone(),
                skill_family: format!("{:?}", sk.spec.skill_family).to_lowercase(),
                status: format!("{:?}", sk.spec.status).to_lowercase(),
                direction: sk.spec.applicable_market_states.first().cloned().unwrap_or_default(),
                market_states: sk.spec.applicable_market_states.clone(),
            })
            .collect(),
    )
}

async fn get_skill(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<SkillResponse>, StatusCode> {
    let s = state.read().await;
    let engine = s.pipeline.signal_engine();
    let registry = engine.registry();
    let sk = registry.get(&id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(SkillResponse {
        skill_id: sk.spec.skill_id.clone(),
        skill_name: sk.spec.skill_name.clone(),
        skill_family: format!("{:?}", sk.spec.skill_family).to_lowercase(),
        status: format!("{:?}", sk.spec.status).to_lowercase(),
        direction: String::new(),
        market_states: sk.spec.applicable_market_states.clone(),
    }))
}

#[derive(Serialize)]
struct SignalResponse {
    signal_id: String,
    signal_type: String,
    symbol: String,
    direction: String,
    confidence: f64,
    horizon: String,
    reason_codes: Vec<String>,
}

async fn get_signals(State(state): State<SharedState>) -> Json<Vec<SignalResponse>> {
    let s = state.read().await;
    Json(
        s.signal_history
            .iter()
            .map(|sig| SignalResponse {
                signal_id: sig.signal_id.clone(),
                signal_type: format!("{:?}", sig.signal_type).to_lowercase(),
                symbol: sig.symbol.clone(),
                direction: format!("{:?}", sig.direction),
                confidence: sig.confidence,
                horizon: format!("{:?}", sig.horizon).to_lowercase(),
                reason_codes: sig.reason_codes.clone(),
            })
            .collect(),
    )
}

#[derive(Serialize, Clone)]
pub struct RiskDecisionRecord {
    pub decision_id: String,
    pub intent_id: String,
    pub symbol: String,
    pub decision: String,
    pub severity: String,
    pub rule_hits: Vec<RuleHitRecord>,
}

#[derive(Serialize, Clone)]
pub struct RuleHitRecord {
    pub rule_id: String,
    pub rule_name: String,
    pub detail: String,
}

#[derive(Serialize, Clone)]
pub struct ExecutionRecord {
    pub report_id: String,
    pub intent_id: String,
    pub symbol: String,
    pub status: String,
    pub filled_qty: Option<String>,
    pub avg_fill_price: Option<String>,
    pub fees: Option<String>,
}

async fn get_risk_events(State(state): State<SharedState>) -> Json<Vec<RiskDecisionRecord>> {
    let s = state.read().await;
    Json(s.risk_history.clone())
}

async fn get_risk_rules() -> Json<Vec<serde_json::Value>> {
    Json(vec![
        serde_json::json!({"rule_id": "R001", "name": "max_leverage", "threshold": "5x", "severity": "Critical"}),
        serde_json::json!({"rule_id": "R002", "name": "max_position_notional", "threshold": "$50,000", "severity": "Warning"}),
        serde_json::json!({"rule_id": "R003", "name": "max_total_exposure", "threshold": "$200,000", "severity": "Warning"}),
        serde_json::json!({"rule_id": "R004", "name": "max_daily_loss", "threshold": "3%", "severity": "Critical"}),
        serde_json::json!({"rule_id": "R005", "name": "cooldown_after_loss", "threshold": "24 bars", "severity": "Info"}),
        serde_json::json!({"rule_id": "R006", "name": "max_open_orders", "threshold": "10", "severity": "Info"}),
        serde_json::json!({"rule_id": "R007", "name": "max_single_risk", "threshold": "5%", "severity": "Warning"}),
    ])
}

async fn get_executions(State(state): State<SharedState>) -> Json<Vec<ExecutionRecord>> {
    let s = state.read().await;
    Json(s.execution_history.clone())
}

#[derive(Deserialize)]
struct PipelineRequest {
    scenarios: Option<Vec<ScenarioDef>>,
}

#[derive(Deserialize)]
struct ScenarioDef {
    symbol: String,
    trend: String,
    candles: Option<usize>,
    price: Option<f64>,
}

#[derive(Serialize)]
struct PipelineRunResponse {
    scenarios_run: usize,
    total_signals: usize,
    total_intents: usize,
    total_executed: usize,
    total_rejected_risk: usize,
    open_positions: usize,
}

async fn run_pipeline(
    State(state): State<SharedState>,
    Json(req): Json<PipelineRequest>,
) -> Json<PipelineRunResponse> {
    let mut s = state.write().await;

    let scenarios: Vec<(String, OhlcvSeries, f64)> = req.scenarios.map_or_else(
        || {
            vec![
                ("BTCUSDT".into(), make_bull_series("BTCUSDT", 120), 68000.0),
                ("ETHUSDT".into(), make_bull_series("ETHUSDT", 120), 3700.0),
                ("SOLUSDT".into(), make_bull_series("SOLUSDT", 120), 170.0),
            ]
        },
        |defs| {
            defs.iter()
                .map(|d| {
                    let n = d.candles.unwrap_or(120);
                    let series = match d.trend.as_str() {
                        "bear" => make_bear_series(&d.symbol, n),
                        _ => make_bull_series(&d.symbol, n),
                    };
                    let price = d.price.unwrap_or_else(|| match d.symbol.as_str() {
                        s if s.starts_with("BTC") => 68000.0,
                        s if s.starts_with("ETH") => 3700.0,
                        _ => 150.0,
                    });
                    (d.symbol.clone(), series, price)
                })
                .collect()
        },
    );

    let mut total_signals = 0;
    let mut total_intents = 0;
    let mut total_executed = 0;
    let mut total_rejected = 0;

    for (_name, series, price) in &scenarios {
        if let Ok(result) = s.pipeline.process_candles(series, *price) {
            for sig in &result.signals {
                s.signal_history.push(sig.clone());
            }
            for dec in &result.decisions {
                s.risk_history.push(RiskDecisionRecord {
                    decision_id: dec.decision_id.clone(),
                    intent_id: dec.target_ref.clone(),
                    symbol: result.intents.first().map(|i| i.symbol.clone()).unwrap_or_default(),
                    decision: format!("{:?}", dec.decision).to_lowercase(),
                    severity: format!("{:?}", dec.severity).to_lowercase(),
                    rule_hits: dec.rule_hits.iter().map(|h| RuleHitRecord {
                        rule_id: h.rule_id.clone(),
                        rule_name: h.rule_name.clone(),
                        detail: h.detail.clone(),
                    }).collect(),
                });
                if matches!(dec.decision, RiskVerdict::Deny) {
                    total_rejected += 1;
                }
            }
            for rep in &result.reports {
                if matches!(rep.order_status, OrderStatus::Filled) {
                    total_executed += 1;
                }
                s.execution_history.push(ExecutionRecord {
                    report_id: rep.report_id.clone(),
                    intent_id: rep.intent_id.clone(),
                    symbol: rep.symbol.clone(),
                    status: format!("{:?}", rep.order_status),
                    filled_qty: rep.filled_qty.clone(),
                    avg_fill_price: rep.avg_fill_price.clone(),
                    fees: rep.fees.as_ref().map(|f| format!("{} {}", f.commission, f.commission_asset)),
                });
            }
            total_signals += result.signals.len();
            total_intents += result.intents.len();
        }
    }

    let open = s.pipeline.portfolio_manager().tracker().open_position_count();
    Json(PipelineRunResponse {
        scenarios_run: scenarios.len(),
        total_signals,
        total_intents,
        total_executed,
        total_rejected_risk: total_rejected,
        open_positions: open,
    })
}

#[derive(Deserialize)]
struct EvaluateRequest {
    candles: usize,
}

async fn evaluate_symbol(
    State(state): State<SharedState>,
    Path(symbol): Path<String>,
    Json(req): Json<EvaluateRequest>,
) -> Result<Json<PipelineRunResponse>, StatusCode> {
    let mut s = state.write().await;
    let series = make_bull_series(&symbol, req.candles.max(60));
    let price = series.candles.last().map(|c| c.close).unwrap_or(100.0);

    let result = s.pipeline.process_candles(&series, price).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for sig in &result.signals {
        s.signal_history.push(sig.clone());
    }

    let open = s.pipeline.portfolio_manager().tracker().open_position_count();
    Ok(Json(PipelineRunResponse {
        scenarios_run: 1,
        total_signals: result.signals.len(),
        total_intents: result.intents.len(),
        total_executed: result.reports.iter().filter(|r| matches!(r.order_status, OrderStatus::Filled)).count(),
        total_rejected_risk: 0,
        open_positions: open,
    }))
}

fn make_bull_series(symbol: &str, n: usize) -> OhlcvSeries {
    let mut series = OhlcvSeries::new(symbol, "1h");
    let base = match symbol {
        s if s.starts_with("BTC") => 65000.0,
        s if s.starts_with("ETH") => 3500.0,
        s if s.starts_with("SOL") => 150.0,
        _ => 100.0,
    };
    for i in 0..n {
        let trend = (i as f64) * 15.0;
        let noise = ((i * 7) as f64).sin() * 200.0;
        let o = base + trend + noise;
        let c = o + 50.0 + (i as f64).sin() * 30.0;
        let h = c + 100.0;
        let l = o - 80.0;
        let v = 1000.0 + (i as f64).sin().abs() * 500.0 + if i > n - 20 { 1500.0 } else { 0.0 };
        series.candles.push(OhlcvCandle {
            time: 1700000000 + (i as i64) * 3600,
            open: o, high: h, low: l, close: c,
            volume: v, quote_volume: v * c, trades: 1000 + i as i64,
        });
    }
    series
}

fn make_bear_series(symbol: &str, n: usize) -> OhlcvSeries {
    let mut series = OhlcvSeries::new(symbol, "1h");
    let base = match symbol {
        s if s.starts_with("BTC") => 65000.0,
        s if s.starts_with("ETH") => 3500.0,
        _ => 100.0,
    };
    for i in 0..n {
        let trend = -(i as f64) * 12.0;
        let noise = ((i * 13) as f64).cos() * 150.0;
        let o = base + trend + noise;
        let c = o - 60.0 - (i as f64).cos().abs() * 40.0;
        let h = o + 80.0;
        let l = c - 120.0;
        let v = 1200.0 + (i as f64).cos().abs() * 600.0 + if i > n - 20 { 2000.0 } else { 0.0 };
        series.candles.push(OhlcvCandle {
            time: 1700000000 + (i as i64) * 3600,
            open: o, high: h, low: l, close: c,
            volume: v, quote_volume: v * c, trades: 1000 + i as i64,
        });
    }
    series
}
