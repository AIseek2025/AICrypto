use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub total_pnl: f64,
    pub total_commission: f64,
    pub net_pnl: f64,
    pub max_drawdown: f64,
    pub max_drawdown_pct: f64,
    pub sharpe_ratio: f64,
    pub avg_bars_held: f64,
    pub max_consecutive_wins: usize,
    pub max_consecutive_losses: usize,
    pub equity_curve: Vec<EquityPoint>,
    pub trade_log: Vec<TradeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub time: i64,
    pub equity: f64,
    pub drawdown: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub trade_id: usize,
    pub symbol: String,
    pub side: String,
    pub entry_time: i64,
    pub exit_time: i64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub commission: f64,
    pub bars_held: i64,
    pub exit_reason: String,
}

impl BacktestMetrics {
    pub fn compute(trades: Vec<TradeRecord>, initial_equity: f64) -> Self {
        let total_trades = trades.len();
        if total_trades == 0 {
            return Self {
                total_trades: 0, winning_trades: 0, losing_trades: 0,
                win_rate: 0.0, avg_win: 0.0, avg_loss: 0.0, profit_factor: 0.0,
                total_pnl: 0.0, total_commission: 0.0, net_pnl: 0.0,
                max_drawdown: 0.0, max_drawdown_pct: 0.0, sharpe_ratio: 0.0,
                avg_bars_held: 0.0, max_consecutive_wins: 0, max_consecutive_losses: 0,
                equity_curve: Vec::new(), trade_log: Vec::new(),
            };
        }

        let winning: Vec<&TradeRecord> = trades.iter().filter(|t| t.pnl > 0.0).collect();
        let losing: Vec<&TradeRecord> = trades.iter().filter(|t| t.pnl <= 0.0).collect();

        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let total_commission: f64 = trades.iter().map(|t| t.commission).sum();
        let net_pnl = total_pnl - total_commission;

        let gross_profit: f64 = winning.iter().map(|t| t.pnl).sum();
        let gross_loss: f64 = losing.iter().map(|t| t.pnl.abs()).sum();
        let profit_factor = if gross_loss == 0.0 && gross_profit > 0.0 {
            gross_profit / 0.0001
        } else if gross_loss == 0.0 {
            0.0
        } else {
            gross_profit / gross_loss
        };

        let avg_win = if winning.is_empty() { 0.0 } else { winning.iter().map(|t| t.pnl).sum::<f64>() / winning.len() as f64 };
        let avg_loss = if losing.is_empty() { 0.0 } else { losing.iter().map(|t| t.pnl).sum::<f64>() / losing.len() as f64 };

        let mut equity = initial_equity;
        let mut peak = equity;
        let mut max_dd = 0.0_f64;
        let mut max_drawdown_pct = 0.0_f64;
        let mut equity_curve = Vec::new();
        let mut daily_returns = Vec::new();

        let mut dd_peak = equity;
        for t in &trades {
            equity += t.pnl - t.commission;
            dd_peak = dd_peak.max(equity);
            let dd = dd_peak - equity;
            if dd > max_dd {
                max_dd = dd;
                max_drawdown_pct = if dd_peak > 0.0 { dd / dd_peak * 100.0 } else { 0.0 };
            }
            peak = peak.max(equity);
            equity_curve.push(EquityPoint {
                time: t.exit_time,
                equity,
                drawdown: dd,
            });
            daily_returns.push(t.pnl / initial_equity);
        }

        let sharpe = if daily_returns.len() < 2 {
            0.0
        } else {
            let mean = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
            let std_dev = {
                let variance = daily_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (daily_returns.len() - 1) as f64;
                variance.sqrt()
            };
            if std_dev == 0.0 { 0.0 } else { mean / std_dev * (252.0_f64).sqrt() }
        };

        let avg_bars_held = trades.iter().map(|t| t.bars_held as f64).sum::<f64>() / total_trades as f64;

        let mut consec_wins = 0usize;
        let mut consec_losses = 0usize;
        let mut max_cw = 0usize;
        let mut max_cl = 0usize;
        for t in &trades {
            if t.pnl > 0.0 {
                consec_wins += 1;
                consec_losses = 0;
                max_cw = max_cw.max(consec_wins);
            } else {
                consec_losses += 1;
                consec_wins = 0;
                max_cl = max_cl.max(consec_losses);
            }
        }

        Self {
            total_trades,
            winning_trades: winning.len(),
            losing_trades: losing.len(),
            win_rate: winning.len() as f64 / total_trades as f64,
            avg_win,
            avg_loss,
            profit_factor,
            total_pnl,
            total_commission,
            net_pnl,
            max_drawdown: max_dd,
            max_drawdown_pct,
            sharpe_ratio: sharpe,
            avg_bars_held,
            max_consecutive_wins: max_cw,
            max_consecutive_losses: max_cl,
            equity_curve,
            trade_log: trades,
        }
    }
}
