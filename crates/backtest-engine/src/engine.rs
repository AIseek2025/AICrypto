#![allow(unused_assignments)]
use crate::metrics::{BacktestMetrics, TradeRecord};
use crate::position::{Position, PositionSide};
use crate::report::BacktestReport;
use aicrypto_feature_engine::ohlcv::OhlcvCandle;
use serde::{Deserialize, Serialize};

pub trait BacktestStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn on_bar(&mut self, bar: &OhlcvCandle, bar_index: usize, position: Option<&Position>) -> StrategyAction;
}

#[derive(Debug, Clone)]
pub enum StrategyAction {
    None,
    OpenLong { quantity: f64, reason: String },
    OpenShort { quantity: f64, reason: String },
    ClosePosition { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub initial_equity: f64,
    pub commission_rate: f64,
    pub slippage_bps: f64,
    pub leverage: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_equity: 10000.0,
            commission_rate: 0.0004,
            slippage_bps: 1.0,
            leverage: 1.0,
        }
    }
}

pub struct BacktestEngine {
    config: BacktestConfig,
}

impl BacktestEngine {
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    pub fn run(
        &self,
        strategy: &mut dyn BacktestStrategy,
        candles: &[OhlcvCandle],
        symbol: &str,
        interval: &str,
    ) -> BacktestReport {
        #[allow(unused_assignments)]
        let mut equity = self.config.initial_equity;
        let mut position: Option<Position> = None;
        let mut trades: Vec<TradeRecord> = Vec::new();
        let mut trade_id = 0usize;

        for (i, bar) in candles.iter().enumerate() {
            if let Some(ref mut pos) = position {
                pos.update_unrealized(bar.close);
                pos.bars_held += 1;
            }

            let action = strategy.on_bar(bar, i, position.as_ref());

            match action {
                StrategyAction::None => {}
                StrategyAction::OpenLong { quantity, reason } if position.is_none() => {
                    let fill_price = self.apply_slippage(bar.close, true);
                    let commission = fill_price * quantity * self.config.commission_rate;
                    let qty = quantity.min(self.position_size(equity, fill_price));
                    if qty > 0.0 {
                        position = Some(Position::new_long(symbol, qty, fill_price, bar.time));
                        equity -= commission;
                    }
                }
                StrategyAction::OpenShort { quantity, reason } if position.is_none() => {
                    let fill_price = self.apply_slippage(bar.close, false);
                    let commission = fill_price * quantity * self.config.commission_rate;
                    let qty = quantity.min(self.position_size(equity, fill_price));
                    if qty > 0.0 {
                        position = Some(Position::new_short(symbol, qty, fill_price, bar.time));
                        equity -= commission;
                    }
                }
                StrategyAction::ClosePosition { reason } if position.is_some() => {
                    let mut pos = position.take().unwrap();
                    let fill_price = self.apply_slippage(bar.close, matches!(pos.side, PositionSide::Long));
                    let commission = fill_price * pos.quantity * self.config.commission_rate;
                    let pnl = pos.close(fill_price, commission);

                    trade_id += 1;
                    let entry_price = pos.entry_price;
                    let pnl_pct = if entry_price > 0.0 { pnl / (entry_price * pos.quantity) } else { 0.0 };

                    trades.push(TradeRecord {
                        trade_id,
                        symbol: symbol.to_string(),
                        side: match pos.side {
                            PositionSide::Long => "LONG".to_string(),
                            PositionSide::Short => "SHORT".to_string(),
                        },
                        entry_time: pos.open_time,
                        exit_time: bar.time,
                        entry_price: pos.entry_price,
                        exit_price: fill_price,
                        quantity: pos.quantity,
                        pnl,
                        pnl_pct,
                        commission,
                        bars_held: pos.bars_held,
                        exit_reason: reason,
                    });

                    equity += pnl;
                }
                _ => {}
            }

            if let Some(ref pos) = position {
                if pos.quantity <= 0.0 {
                    position = None;
                }
            }
        }

        if let Some(mut pos) = position.take() {
            if let Some(last_bar) = candles.last() {
                let commission = last_bar.close * pos.quantity * self.config.commission_rate;
                let pnl = pos.close(last_bar.close, commission);
                trade_id += 1;
                trades.push(TradeRecord {
                    trade_id,
                    symbol: symbol.to_string(),
                    side: match pos.side {
                        PositionSide::Long => "LONG".to_string(),
                        PositionSide::Short => "SHORT".to_string(),
                    },
                    entry_time: pos.open_time,
                    exit_time: last_bar.time,
                    entry_price: pos.entry_price,
                    exit_price: last_bar.close,
                    quantity: pos.quantity,
                    pnl,
                    pnl_pct: if pos.entry_price > 0.0 { pnl / (pos.entry_price * pos.quantity) } else { 0.0 },
                    commission,
                    bars_held: pos.bars_held,
                    exit_reason: "end_of_data".to_string(),
                });
                equity += pnl;
            }
        }

        let metrics = BacktestMetrics::compute(trades, self.config.initial_equity);

        BacktestReport {
            strategy_name: strategy.name().to_string(),
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            start_time: candles.first().map(|c| c.time).unwrap_or(0),
            end_time: candles.last().map(|c| c.time).unwrap_or(0),
            initial_equity: self.config.initial_equity,
            commission_rate: self.config.commission_rate,
            slippage_bps: self.config.slippage_bps,
            total_bars: candles.len(),
            metrics,
        }
    }

    fn apply_slippage(&self, price: f64, is_buy: bool) -> f64 {
        let slip = price * self.config.slippage_bps / 10000.0;
        if is_buy { price + slip } else { price - slip }
    }

    fn position_size(&self, equity: f64, price: f64) -> f64 {
        if price <= 0.0 { return 0.0; }
        let notional = equity * self.config.leverage;
        notional / price
    }
}
