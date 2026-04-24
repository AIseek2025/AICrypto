use aicrypto_backtest_engine::engine::{BacktestConfig, BacktestEngine};
use aicrypto_backtest_engine::strategy::{BtcBreakoutLong, GlobalRiskOffReduce};
use aicrypto_foundation::observability;
use anyhow::Result;

fn main() -> Result<()> {
    observability::init_tracing("backtest-engine");

    println!("=== AICrypto Backtest Engine ===\n");

    let candles = generate_sample_candles();

    let config = BacktestConfig {
        initial_equity: 10000.0,
        commission_rate: 0.0004,
        slippage_bps: 1.0,
        leverage: 3.0,
    };
    let engine = BacktestEngine::new(config);

    println!("--- Strategy 1: BTC Breakout Long ---");
    let mut strategy1 = BtcBreakoutLong::new();
    let report1 = engine.run(&mut strategy1, &candles, "BTCUSDT", "1h");
    println!("{}\n", report1.to_summary());

    println!("--- Strategy 2: Global Risk-Off Reduce ---");
    let mut strategy2 = GlobalRiskOffReduce::new();
    let report2 = engine.run(&mut strategy2, &candles, "BTCUSDT", "1h");
    println!("{}\n", report2.to_summary());

    let json = report1.to_json();
    println!("Report 1 JSON (first 500 chars):");
    println!("{}\n", &json[..json.len().min(500)]);

    println!("=== Backtest Complete ===");
    Ok(())
}

fn generate_sample_candles() -> Vec<aicrypto_feature_engine::ohlcv::OhlcvCandle> {
    use aicrypto_feature_engine::ohlcv::OhlcvCandle;
    let mut candles = Vec::new();
    let mut price = 40000.0_f64;
    let base_time = 1700000000000_i64;
    let hour_ms = 3600000_i64;

    let trend = vec![
        (0.005, 0.01), (0.003, 0.008), (-0.002, 0.006), (0.008, 0.012),
        (0.006, 0.009), (0.01, 0.015), (0.002, 0.007), (-0.005, 0.01),
        (0.007, 0.011), (0.009, 0.013), (0.015, 0.02), (0.003, 0.008),
        (-0.001, 0.006), (0.004, 0.01), (0.006, 0.009), (0.008, 0.012),
        (0.012, 0.018), (0.001, 0.005), (-0.003, 0.008), (0.007, 0.011),
    ];

    for i in 0..500 {
        let phase = trend[i % trend.len()];
        let noise = (i as f64 * 0.1).sin() * 0.003 ;
        let ret = phase.0 + noise;
        let vol_factor = phase.1;

        let open = price;
        let change = price * ret;
        let close = price + change;
        let high = open.max(close) + price * vol_factor * 0.5;
        let low = open.min(close) - price * vol_factor * 0.3;
        let volume = 1000.0 + vol_factor * 5000.0 + (i as f64 * 0.3).sin().abs() * 3000.0;

        candles.push(OhlcvCandle {
            time: base_time + i as i64 * hour_ms,
            open,
            high,
            low: low.max(0.0),
            close,
            volume,
            quote_volume: volume * (open + close) / 2.0,
            trades: (volume / 10.0) as i64,
        });

        price = close;
    }

    candles
}
