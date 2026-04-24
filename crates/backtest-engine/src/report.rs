use crate::metrics::BacktestMetrics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestReport {
    pub strategy_name: String,
    pub symbol: String,
    pub interval: String,
    pub start_time: i64,
    pub end_time: i64,
    pub initial_equity: f64,
    pub commission_rate: f64,
    pub slippage_bps: f64,
    pub total_bars: usize,
    pub metrics: BacktestMetrics,
}

impl BacktestReport {
    pub fn to_summary(&self) -> String {
        let m = &self.metrics;
        format!(
            "=== Backtest Report: {} {} {} ===\n\
             Period: {} -> {}\n\
             Bars: {} | Trades: {} | Win Rate: {:.1}%\n\
             Total PnL: {:.2} | Commission: {:.2} | Net PnL: {:.2}\n\
             Avg Win: {:.2} | Avg Loss: {:.2} | Profit Factor: {:.2}\n\
             Max Drawdown: {:.2} ({:.1}%)\n\
             Sharpe Ratio: {:.2}\n\
             Avg Bars Held: {:.1} | Max Consec Wins: {} | Max Consec Losses: {}",
            self.strategy_name, self.symbol, self.interval,
            chrono::DateTime::from_timestamp_millis(self.start_time)
                .map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
            chrono::DateTime::from_timestamp_millis(self.end_time)
                .map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
            self.total_bars, m.total_trades, m.win_rate * 100.0,
            m.total_pnl, m.total_commission, m.net_pnl,
            m.avg_win, m.avg_loss, m.profit_factor,
            m.max_drawdown, m.max_drawdown_pct,
            m.sharpe_ratio,
            m.avg_bars_held, m.max_consecutive_wins, m.max_consecutive_losses,
        )
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}
