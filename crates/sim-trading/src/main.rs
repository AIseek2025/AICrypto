use aicrypto_sim_trading::engine::AutoTrader;
use aicrypto_foundation::observability;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    observability::init_tracing("sim-trading");

    tracing::info!("sim-trading engine starting");

    let skills_dir = Path::new("skills");
    let state_path = Path::new("/tmp/aicrypto-sim-state.json");
    let initial_equity = 10_000.0;

    let trader = AutoTrader::new(skills_dir, initial_equity, state_path)
        .expect("failed to init auto-trader");

    let trader = Arc::new(Mutex::new(trader));

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let trader_clone = trader.clone();
    let api_state: Arc<Mutex<Option<aicrypto_sim_trading::SimState>>> = Arc::new(Mutex::new(None));

    let api_state_clone = api_state.clone();
    std::thread::spawn(move || {
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8090));
        let app = create_api_router(api_state_clone);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            tracing::info!("sim-trading API listening on http://0.0.0.0:8090");
            axum::serve(listener, app).await.unwrap();
        });
    });

    let tick_interval = Duration::from_secs(5);
    let mut cycle = 0u64;

    tracing::info!(
        initial_equity = initial_equity,
        tick_interval_secs = tick_interval.as_secs(),
        "auto-trading loop started — Ctrl+C to stop"
    );

    loop {
        cycle += 1;
        {
            let mut trader = trader.lock().unwrap();
            match trader.run_cycle() {
                Ok(()) => {
                    let st = &trader.state;
                    if cycle % 20 == 0 {
                        tracing::info!(
                            cycle = cycle,
                            equity = format!("{:.2}", st.account.equity),
                            pnl = format!("{:.2}", st.total_pnl()),
                            positions = st.positions.len(),
                            trades = st.trades.len(),
                            "trading cycle"
                        );
                    }
                    let mut api = api_state.lock().unwrap();
                    *api = Some(st.clone());
                }
                Err(e) => {
                    tracing::error!(error = %e, "trading cycle failed");
                }
            }
        }
        std::thread::sleep(tick_interval);
    }
}

fn create_api_router(
    state: Arc<Mutex<Option<aicrypto_sim_trading::SimState>>>,
) -> axum::Router {
    use axum::{routing::get, Json, Router, extract::State as AxumState};

    let app = Router::new()
        .route("/api/sim/account", get({
            let state = state.clone();
            move || {
                let state = state.clone();
                async move {
                    let guard = state.lock().unwrap();
                    match guard.as_ref() {
                        Some(s) => Json(serde_json::json!({
                            "account_id": s.account.account_id,
                            "initial_equity": s.account.initial_equity,
                            "equity": s.account.equity,
                            "cash": s.account.cash,
                            "total_pnl": s.total_pnl(),
                            "return_pct": format!("{:.2}%", s.return_pct() * 100.0),
                            "annualized_return": format!("{:.2}%", s.annualized_return()),
                            "win_rate": format!("{:.1}%", s.win_rate() * 100.0),
                            "sharpe_ratio": format!("{:.2}", s.sharpe_ratio()),
                            "max_drawdown": format!("{:.2}%", s.max_drawdown() * 100.0),
                            "total_trades": s.trades.len(),
                            "open_positions": s.positions.len(),
                            "total_exposure": s.total_exposure(),
                            "started_at": s.account.started_at,
                            "cycle_count": s.cycle_count,
                        })),
                        None => Json(serde_json::json!({"error": "not ready"})),
                    }
                }
            }
        }))
        .route("/api/sim/positions", get({
            let state = state.clone();
            move || {
                let state = state.clone();
                async move {
                    let guard = state.lock().unwrap();
                    match guard.as_ref() {
                        Some(s) => {
                            let positions: Vec<_> = s.positions.values().cloned().collect();
                            Json(serde_json::json!(positions))
                        }
                        None => Json(serde_json::json!([])),
                    }
                }
            }
        }))
        .route("/api/sim/trades", get({
            let state = state.clone();
            move || {
                let state = state.clone();
                async move {
                    let guard = state.lock().unwrap();
                    match guard.as_ref() {
                        Some(s) => {
                            let trades: Vec<_> = s.trades.iter().rev().take(200).cloned().collect();
                            Json(serde_json::json!(trades))
                        }
                        None => Json(serde_json::json!([])),
                    }
                }
            }
        }))
        .route("/api/sim/equity-curve", get({
            let state = state.clone();
            move || {
                let state = state.clone();
                async move {
                    let guard = state.lock().unwrap();
                    match guard.as_ref() {
                        Some(s) => {
                            let curve: Vec<_> = s.equity_curve.iter().rev().take(500).cloned().collect();
                            Json(serde_json::json!(curve))
                        }
                        None => Json(serde_json::json!([])),
                    }
                }
            }
        }))
        .route("/api/sim/decisions", get({
            let state = state.clone();
            move || {
                let state = state.clone();
                async move {
                    let guard = state.lock().unwrap();
                    match guard.as_ref() {
                        Some(s) => {
                            let logs: Vec<_> = s.decision_logs.iter().rev().take(100).cloned().collect();
                            Json(serde_json::json!(logs))
                        }
                        None => Json(serde_json::json!([])),
                    }
                }
            }
        }))
        .route("/api/sim/reviews", get({
            let state = state.clone();
            move || {
                let state = state.clone();
                async move {
                    let guard = state.lock().unwrap();
                    match guard.as_ref() {
                        Some(s) => {
                            let reviews: Vec<_> = s.reviews.iter().rev().take(100).cloned().collect();
                            Json(serde_json::json!(reviews))
                        }
                        None => Json(serde_json::json!([])),
                    }
                }
            }
        }))
        .route("/api/sim/prices", get({
            let state = state.clone();
            move || {
                let state = state.clone();
                async move {
                    let guard = state.lock().unwrap();
                    match guard.as_ref() {
                        Some(s) => Json(serde_json::json!(s.last_market_prices)),
                        None => Json(serde_json::json!({})),
                    }
                }
            }
        }))
        .route("/api/sim/learning", get({
            let state = state.clone();
            move || {
                let state = state.clone();
                async move {
                    let guard = state.lock().unwrap();
                    match guard.as_ref() {
                        Some(s) => {
                            let wins: Vec<_> = s.reviews.iter().filter(|r| r.outcome == "WIN").collect();
                            let losses: Vec<_> = s.reviews.iter().filter(|r| r.outcome == "LOSS").collect();
                            let skills_reinforced: Vec<_> = wins.iter()
                                .filter_map(|r| r.skill_reinforced.clone()).collect();
                            let rules_added: Vec<_> = losses.iter()
                                .filter_map(|r| r.risk_rule_added.clone()).collect();
                            Json(serde_json::json!({
                                "total_reviews": s.reviews.len(),
                                "wins": wins.len(),
                                "losses": losses.len(),
                                "skills_reinforced": skills_reinforced,
                                "risk_rules_added": rules_added,
                                "recent_lessons": s.reviews.iter().rev().take(10)
                                    .flat_map(|r| r.lessons.clone())
                                    .collect::<Vec<_>>(),
                            }))
                        }
                        None => Json(serde_json::json!({"total_reviews": 0})),
                    }
                }
            }
        }))
        .with_state(());

    app
}
