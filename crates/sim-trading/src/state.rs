use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimAccount {
    pub account_id: String,
    pub initial_equity: f64,
    pub equity: f64,
    pub cash: f64,
    pub started_at: i64,
    pub updated_at: i64,
}

impl SimAccount {
    pub fn new(initial_equity: f64) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            account_id: uuid::Uuid::new_v4().to_string(),
            initial_equity,
            equity: initial_equity,
            cash: initial_equity,
            started_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimPosition {
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl: f64,
    pub leverage: u32,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub opened_at: i64,
}

impl SimPosition {
    pub fn notional(&self) -> f64 {
        self.quantity * self.mark_price
    }

    pub fn update_mark_price(&mut self, price: f64) {
        self.mark_price = price;
        self.unrealized_pnl = match self.side.as_str() {
            "LONG" => (price - self.entry_price) * self.quantity,
            "SHORT" => (self.entry_price - price) * self.quantity,
            _ => 0.0,
        };
    }

    pub fn should_stop_loss(&self) -> bool {
        self.stop_loss.map_or(false, |sl| match self.side.as_str() {
            "LONG" => self.mark_price <= sl,
            "SHORT" => self.mark_price >= sl,
            _ => false,
        })
    }

    pub fn should_take_profit(&self) -> bool {
        self.take_profit.map_or(false, |tp| match self.side.as_str() {
            "LONG" => self.mark_price >= tp,
            "SHORT" => self.mark_price <= tp,
            _ => false,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub trade_id: String,
    pub symbol: String,
    pub side: String,
    pub direction: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub realized_pnl: f64,
    pnl_pct: f64,
    pub commission: f64,
    pub opened_at: i64,
    pub closed_at: i64,
    pub close_reason: String,
    pub skill_id: Option<String>,
    pub decision_log_id: Option<String>,
}

impl TradeRecord {
    pub fn new(
        symbol: &str, side: &str, direction: &str, qty: f64,
        entry: f64, exit: f64, opened_at: i64, close_reason: &str,
    ) -> Self {
        let pnl = match side {
            "LONG" => (exit - entry) * qty,
            "SHORT" => (entry - exit) * qty,
            _ => 0.0,
        };
        let commission = qty * exit * 0.0004;
        let pnl_pct = if entry > 0.0 { pnl / (entry * qty) } else { 0.0 };
        Self {
            trade_id: uuid::Uuid::new_v4().to_string(),
            symbol: symbol.to_string(),
            side: side.to_string(),
            direction: direction.to_string(),
            quantity: qty,
            entry_price: entry,
            exit_price: exit,
            realized_pnl: pnl,
            pnl_pct,
            commission,
            opened_at,
            closed_at: chrono::Utc::now().timestamp_millis(),
            close_reason: close_reason.to_string(),
            skill_id: None,
            decision_log_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub ts: i64,
    pub equity: f64,
    pub cash: f64,
    pub unrealized_pnl: f64,
    pub open_positions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLog {
    pub log_id: String,
    pub ts: i64,
    pub action: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub price: f64,
    pub confidence: f64,
    pub skill_id: Option<String>,
    pub market_state: String,
    pub feature_summary: Vec<String>,
    pub thinking: String,
    pub risk_decision: String,
    pub risk_rule_hits: Vec<String>,
}

impl DecisionLog {
    pub fn new(action: &str, symbol: &str, side: &str, qty: f64, price: f64) -> Self {
        Self {
            log_id: uuid::Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().timestamp_millis(),
            action: action.to_string(),
            symbol: symbol.to_string(),
            side: side.to_string(),
            quantity: qty,
            price: price,
            confidence: 0.0,
            skill_id: None,
            market_state: "UNKNOWN".to_string(),
            feature_summary: vec![],
            thinking: String::new(),
            risk_decision: "PENDING".to_string(),
            risk_rule_hits: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeReview {
    pub review_id: String,
    pub trade_id: String,
    pub symbol: String,
    pub side: String,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub close_reason: String,
    pub ts: i64,
    pub outcome: String,
    pub analysis: String,
    pub lessons: Vec<String>,
    pub skill_reinforced: Option<String>,
    pub risk_rule_added: Option<String>,
}

impl TradeReview {
    pub fn from_trade(trade: &TradeRecord) -> Self {
        let outcome = if trade.realized_pnl > 0.0 { "WIN" } else { "LOSS" }.to_string();
        let analysis = generate_analysis(trade);
        let lessons = generate_lessons(trade);
        let (skill_reinforced, risk_rule_added) = generate_improvements(trade);

        Self {
            review_id: uuid::Uuid::new_v4().to_string(),
            trade_id: trade.trade_id.clone(),
            symbol: trade.symbol.clone(),
            side: trade.side.clone(),
            pnl: trade.realized_pnl,
            pnl_pct: trade.pnl_pct,
            close_reason: trade.close_reason.clone(),
            ts: chrono::Utc::now().timestamp_millis(),
            outcome,
            analysis,
            lessons,
            skill_reinforced,
            risk_rule_added,
        }
    }
}

fn generate_analysis(trade: &TradeRecord) -> String {
    let direction = if trade.realized_pnl > 0.0 { "盈利" } else { "亏损" };
    format!(
        "交易{}: {} {} {} — 入场 {:.2}, 出场 {:.2}, 盈亏 {:.2} ({:.2}%). 关仓原因: {}",
        direction, trade.side, trade.quantity, trade.symbol,
        trade.entry_price, trade.exit_price, trade.realized_pnl,
        trade.pnl_pct * 100.0, trade.close_reason
    )
}

fn generate_lessons(trade: &TradeRecord) -> Vec<String> {
    let mut lessons = Vec::new();
    if trade.realized_pnl > 0.0 {
        lessons.push(format!("{} 趋势判断正确，{}策略有效", trade.symbol, trade.side));
        if trade.pnl_pct > 0.05 {
            lessons.push("大盈利交易值得分析是否可以加仓".to_string());
        }
    } else {
        lessons.push(format!("{} 交易亏损，需审查入场时机", trade.symbol));
        if trade.close_reason.contains("stop_loss") {
            lessons.push("止损被触发，考虑调整止损距离或入场条件".to_string());
        }
        if trade.pnl_pct < -0.05 {
            lessons.push("大额亏损，需强化风控阈值".to_string());
        }
    }
    lessons
}

fn generate_improvements(trade: &TradeRecord) -> (Option<String>, Option<String>) {
    if trade.realized_pnl > 0.0 {
        let skill = format!("reinforced_{}_{}_win", trade.symbol.to_lowercase(), trade.side.to_lowercase());
        (Some(skill), None)
    } else {
        let risk = format!("tighten_{}_after_{}_loss", trade.symbol.to_lowercase(), trade.close_reason.to_lowercase());
        (None, Some(risk))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimState {
    pub account: SimAccount,
    pub positions: HashMap<String, SimPosition>,
    pub trades: Vec<TradeRecord>,
    pub equity_curve: Vec<EquityPoint>,
    pub decision_logs: Vec<DecisionLog>,
    pub reviews: Vec<TradeReview>,
    pub cycle_count: u64,
    pub last_market_prices: HashMap<String, f64>,
}

impl SimState {
    pub fn new(initial_equity: f64) -> Self {
        Self {
            account: SimAccount::new(initial_equity),
            positions: HashMap::new(),
            trades: Vec::new(),
            equity_curve: Vec::new(),
            decision_logs: Vec::new(),
            reviews: Vec::new(),
            cycle_count: 0,
            last_market_prices: HashMap::new(),
        }
    }

    pub fn total_unrealized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.unrealized_pnl).sum()
    }

    pub fn total_exposure(&self) -> f64 {
        self.positions.values().map(|p| p.notional()).sum()
    }

    pub fn update_equity(&mut self) {
        let unrealized = self.total_unrealized_pnl();
        self.account.equity = self.account.cash + unrealized;
        self.account.updated_at = chrono::Utc::now().timestamp_millis();
    }

    pub fn record_equity_point(&mut self) {
        self.update_equity();
        self.equity_curve.push(EquityPoint {
            ts: chrono::Utc::now().timestamp_millis(),
            equity: self.account.equity,
            cash: self.account.cash,
            unrealized_pnl: self.total_unrealized_pnl(),
            open_positions: self.positions.len(),
        });
        if self.equity_curve.len() > 10000 {
            let drain_count = self.equity_curve.len() - 10000;
            self.equity_curve.drain(0..drain_count);
        }
    }

    pub fn open_position(
        &mut self,
        symbol: &str, side: &str, qty: f64, price: f64,
        leverage: u32, stop_loss: Option<f64>, take_profit: Option<f64>,
    ) {
        let cost = qty * price / leverage as f64;
        self.account.cash -= cost;
        self.positions.insert(symbol.to_string(), SimPosition {
            symbol: symbol.to_string(),
            side: side.to_string(),
            quantity: qty,
            entry_price: price,
            mark_price: price,
            unrealized_pnl: 0.0,
            leverage,
            stop_loss,
            take_profit,
            opened_at: chrono::Utc::now().timestamp_millis(),
        });
    }

    pub fn close_position(&mut self, symbol: &str, reason: &str) -> Option<TradeRecord> {
        let pos = self.positions.remove(symbol)?;
        let proceeds = pos.quantity * pos.mark_price / pos.leverage as f64;
        self.account.cash += proceeds + pos.unrealized_pnl;
        let trade = TradeRecord::new(
            symbol, &pos.side, &pos.side, pos.quantity,
            pos.entry_price, pos.mark_price, pos.opened_at, reason,
        );
        self.trades.push(trade.clone());
        Some(trade)
    }

    pub fn update_prices(&mut self, prices: &HashMap<String, f64>) {
        for (sym, price) in prices {
            self.last_market_prices.insert(sym.clone(), *price);
            if let Some(pos) = self.positions.get_mut(sym) {
                pos.update_mark_price(*price);
            }
        }
    }

    pub fn check_stop_loss_take_profit(&mut self) -> Vec<(String, String)> {
        let mut triggered = Vec::new();
        let symbols: Vec<String> = self.positions.keys().cloned().collect();
        for sym in symbols {
            if let Some(pos) = self.positions.get(&sym) {
                if pos.should_stop_loss() {
                    triggered.push((sym.clone(), "stop_loss".to_string()));
                } else if pos.should_take_profit() {
                    triggered.push((sym.clone(), "take_profit".to_string()));
                }
            }
        }
        triggered
    }

    pub fn win_rate(&self) -> f64 {
        if self.trades.is_empty() { return 0.0; }
        let wins = self.trades.iter().filter(|t| t.realized_pnl > 0.0).count();
        wins as f64 / self.trades.len() as f64
    }

    pub fn total_pnl(&self) -> f64 {
        self.account.equity - self.account.initial_equity
    }

    pub fn return_pct(&self) -> f64 {
        if self.account.initial_equity <= 0.0 { return 0.0; }
        self.total_pnl() / self.account.initial_equity
    }

    pub fn annualized_return(&self) -> f64 {
        let elapsed_ms = chrono::Utc::now().timestamp_millis() - self.account.started_at;
        let elapsed_years = elapsed_ms as f64 / (365.25 * 24.0 * 3600.0 * 1000.0);
        if elapsed_years <= 0.0 { return 0.0; }
        let total_return = self.return_pct();
        ((1.0 + total_return).powf(1.0 / elapsed_years) - 1.0) * 100.0
    }

    pub fn sharpe_ratio(&self) -> f64 {
        if self.trades.len() < 2 { return 0.0; }
        let returns: Vec<f64> = self.trades.iter().map(|t| t.pnl_pct).collect();
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
        let std_dev = variance.sqrt();
        if std_dev <= 0.0 { return 0.0; }
        (mean / std_dev) * (252.0_f64).sqrt()
    }

    pub fn max_drawdown(&self) -> f64 {
        if self.equity_curve.is_empty() { return 0.0; }
        let mut peak = 0.0_f64;
        let mut max_dd = 0.0_f64;
        for point in &self.equity_curve {
            if point.equity > peak {
                peak = point.equity;
            }
            let dd = (peak - point.equity) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
        max_dd
    }

    pub fn generate_review_for_trade(&mut self, trade: &TradeRecord) -> TradeReview {
        let review = TradeReview::from_trade(trade);
        self.reviews.push(review.clone());
        review
    }
}

use std::fs;
use std::path::Path;

impl SimState {
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let json = fs::read_to_string(path)?;
        let state: SimState = serde_json::from_str(&json)?;
        Ok(state)
    }
}
