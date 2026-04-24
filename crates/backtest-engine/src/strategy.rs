use aicrypto_feature_engine::ohlcv::OhlcvCandle;
use crate::engine::StrategyAction;
use crate::position::Position;
use crate::engine::BacktestStrategy;

pub struct BtcBreakoutLong {
    lookback: usize,
    volume_ratio_threshold: f64,
    take_profit_pct: f64,
    stop_loss_pct: f64,
    max_bars: i64,
    entry_price: Option<f64>,
    bars_held: i64,
    closes: Vec<f64>,
    volumes: Vec<f64>,
}

impl BtcBreakoutLong {
    pub fn new() -> Self {
        Self {
            lookback: 20,
            volume_ratio_threshold: 1.2,
            take_profit_pct: 0.05,
            stop_loss_pct: 0.03,
            max_bars: 480,
            entry_price: None,
            bars_held: 0,
            closes: Vec::new(),
            volumes: Vec::new(),
        }
    }

    fn highest_in_lookback(&self) -> Option<f64> {
        if self.closes.len() < self.lookback {
            return None;
        }
        let start = self.closes.len().saturating_sub(self.lookback);
        Some(self.closes[start..self.closes.len() - 1].iter().cloned().fold(f64::NEG_INFINITY, f64::max))
    }

    fn volume_ratio(&self) -> f64 {
        if self.volumes.len() < 20 {
            return 1.0;
        }
        let start = self.volumes.len().saturating_sub(20);
        let avg: f64 = self.volumes[start..].iter().sum::<f64>() / 20.0;
        if avg == 0.0 { return 1.0; }
        *self.volumes.last().unwrap_or(&0.0) / avg
    }
}

impl BacktestStrategy for BtcBreakoutLong {
    fn name(&self) -> &str {
        "BTC_breakout_market_regime_long"
    }

    fn on_bar(&mut self, bar: &OhlcvCandle, _bar_index: usize, position: Option<&Position>) -> StrategyAction {
        self.closes.push(bar.close);
        self.volumes.push(bar.volume);

        if position.is_some() {
            self.bars_held += 1;
            if let Some(ep) = self.entry_price {
                let pnl_pct = (bar.close - ep) / ep;
                if pnl_pct >= self.take_profit_pct {
                    self.entry_price = None;
                    self.bars_held = 0;
                    return StrategyAction::ClosePosition {
                        reason: format!("take_profit ({:.1}%)", pnl_pct * 100.0),
                    };
                }
                if pnl_pct <= -self.stop_loss_pct {
                    self.entry_price = None;
                    self.bars_held = 0;
                    return StrategyAction::ClosePosition {
                        reason: format!("stop_loss ({:.1}%)", pnl_pct * 100.0),
                    };
                }
                if self.bars_held >= self.max_bars {
                    self.entry_price = None;
                    self.bars_held = 0;
                    return StrategyAction::ClosePosition {
                        reason: "time_exit".to_string(),
                    };
                }
            }
            return StrategyAction::None;
        }

        let highest = self.highest_in_lookback();
        let vol_ratio = self.volume_ratio();

        let should_enter = highest.map_or(false, |h| {
            bar.close > h && vol_ratio >= self.volume_ratio_threshold
        });

        if should_enter {
            self.entry_price = Some(bar.close);
            self.bars_held = 0;
            StrategyAction::OpenLong {
                quantity: 1.0,
                reason: format!("breakout above {:.2} vol_ratio {:.1}", highest.unwrap_or(0.0), vol_ratio),
            }
        } else {
            StrategyAction::None
        }
    }
}

pub struct GlobalRiskOffReduce {
    sma_period: usize,
    consecutive_down_threshold: i32,
    prev_close: f64,
    consecutive_down: i32,
    sma_sum: f64,
    sma_count: usize,
}

impl GlobalRiskOffReduce {
    pub fn new() -> Self {
        Self {
            sma_period: 50,
            consecutive_down_threshold: 3,
            prev_close: 0.0,
            consecutive_down: 0,
            sma_sum: 0.0,
            sma_count: 0,
        }
    }
}

impl BacktestStrategy for GlobalRiskOffReduce {
    fn name(&self) -> &str {
        "global_risk_off_reduce_exposure"
    }

    fn on_bar(&mut self, bar: &OhlcvCandle, _bar_index: usize, position: Option<&Position>) -> StrategyAction {
        if self.prev_close > 0.0 {
            if bar.close < self.prev_close {
                self.consecutive_down += 1;
            } else {
                self.consecutive_down = 0;
            }
        }

        self.sma_sum += bar.close;
        self.sma_count += 1;
        let sma = if self.sma_count >= self.sma_period {
            let start_idx = self.sma_count.saturating_sub(self.sma_period);
            self.sma_sum / self.sma_period.min(self.sma_count) as f64
        } else {
            bar.close
        };

        let should_close = position.is_some()
            && self.consecutive_down >= self.consecutive_down_threshold
            && bar.close < sma;

        self.prev_close = bar.close;

        if should_close {
            StrategyAction::ClosePosition {
                reason: format!("risk_off: {} consecutive down bars below SMA50", self.consecutive_down),
            }
        } else {
            StrategyAction::None
        }
    }
}
