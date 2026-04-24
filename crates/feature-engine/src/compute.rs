use crate::feature_set::{basic_trend_features, momentum_features, volatility_features, volume_features, structure_features};
use crate::indicators;
use crate::ohlcv::OhlcvSeries;
use aicrypto_protocols::feature_vector::FeatureVector;
use serde_json::json;
use std::collections::HashMap;

pub fn compute_all_features(series: &OhlcvSeries) -> Option<FeatureVector> {
    if series.candles.len() < 50 {
        return None;
    }
    let closes = series.closes();
    let highs = series.highs();
    let lows = series.lows();
    let volumes = series.volumes();
    let returns = series.returns();

    let mut features = serde_json::Map::new();

    let last_close = *closes.last()?;
    let last_high = *highs.last()?;
    let last_low = *lows.last()?;

    let sma10 = indicators::sma(&closes, 10);
    let sma20 = indicators::sma(&closes, 20);
    let sma50 = indicators::sma(&closes, 50);
    let ema12 = indicators::ema(&closes, 12);
    let ema26 = indicators::ema(&closes, 26);
    let ma20_slope = indicators::slope(&closes, 20);
    let rsi14 = indicators::rsi(&closes, 14);
    let atr14 = indicators::atr(&highs, &lows, &closes, 14);
    let bb = indicators::bollinger_bands(&closes, 20, 2.0);
    let macd = indicators::macd(&closes, 12, 26, 9);
    let stoch = indicators::stochastic(&highs, &lows, &closes, 14, 3);
    let vol_ratio_5 = indicators::volume_ratio(&volumes, 5);
    let vol_ratio_20 = indicators::volume_ratio(&volumes, 20);
    let obv = indicators::obv(&closes, &volumes);
    let obv_slope = indicators::slope(&obv, 10);
    let hist_vol = indicators::historical_volatility(&returns, 20);
    let high_20 = indicators::rolling_max(&highs, 20);
    let low_20 = indicators::rolling_min(&lows, 20);

    let n = series.candles.len();
    let last_idx = n - 1;

    if let Some(Some(v)) = sma10.last() { features.insert("sma_10".into(), json!(v)); }
    if let Some(Some(v)) = sma20.last() { features.insert("sma_20".into(), json!(v)); }
    if let Some(Some(v)) = sma50.last() { features.insert("sma_50".into(), json!(v)); }
    if let Some(Some(v)) = ema12.last() { features.insert("ema_12".into(), json!(v)); }
    if let Some(Some(v)) = ema26.last() { features.insert("ema_26".into(), json!(v)); }

    if let Some(Some(v)) = ma20_slope.last() {
        features.insert("ma20_slope".into(), json!(v));
        let sma20_val = features.get("sma_20").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let sma50_val = features.get("sma_50").and_then(|v| v.as_f64()).unwrap_or(0.0);
        features.insert("price_above_sma20".into(), json!(last_close > sma20_val));
        features.insert("price_above_sma50".into(), json!(last_close > sma50_val));
    }

    if let (Some(Some(sma10_v)), Some(Some(sma20_v))) = (sma10.last(), sma20.last()) {
        features.insert("sma_cross_signal".into(), json!(if sma10_v > sma20_v { 1.0 } else { -1.0 }));
    }

    if let Some(Some(v)) = rsi14.last() { features.insert("rsi_14".into(), json!(v)); }

    if let Some(Some((macd_l, sig_l, hist_l))) = macd.last() {
        features.insert("macd_line".into(), json!(macd_l));
        features.insert("macd_signal".into(), json!(sig_l));
        features.insert("macd_hist".into(), json!(hist_l));
    }

    if let Some(Some((k, d))) = stoch.last() {
        features.insert("stoch_k".into(), json!(k));
        features.insert("stoch_d".into(), json!(d));
    }

    if let Some(Some(v)) = atr14.last() {
        features.insert("atr_14".into(), json!(v));
        features.insert("atr_pct".into(), json!(if last_close > 0.0 { v / last_close } else { 0.0 }));
    }

    if let Some(Some((bb_lower, bb_mid, bb_upper))) = bb.last() {
        features.insert("bb_upper".into(), json!(bb_upper));
        features.insert("bb_lower".into(), json!(bb_lower));
        let bb_width = bb_upper - bb_lower;
        features.insert("bb_width".into(), json!(bb_width));
        let bb_position = if bb_width > 0.0 { (last_close - bb_lower) / bb_width } else { 0.5 };
        features.insert("bb_position".into(), json!(bb_position));
    }

    if let Some(Some(v)) = hist_vol.last() { features.insert("hist_vol_20".into(), json!(v)); }
    if let Some(Some(v)) = vol_ratio_5.last() { features.insert("volume_ratio_5".into(), json!(v)); }
    if let Some(Some(v)) = vol_ratio_20.last() { features.insert("volume_ratio_20".into(), json!(v)); }
    if let Some(Some(v)) = obv_slope.last() { features.insert("obv_slope_10".into(), json!(v)); }

    if let Some(Some(h20)) = high_20.last() {
        features.insert("high_20d".into(), json!(h20));
        features.insert("pct_from_high_20d".into(), json!(if *h20 > 0.0 { (last_close - h20) / h20 } else { 0.0 }));
        features.insert("breakout_high_20d".into(), json!(last_close >= *h20));
    }
    if let Some(Some(l20)) = low_20.last() {
        features.insert("low_20d".into(), json!(l20));
        features.insert("pct_from_low_20d".into(), json!(if *l20 > 0.0 { (last_close - l20) / l20 } else { 0.0 }));
        features.insert("breakdown_low_20d".into(), json!(last_close <= *l20));
    }

    if n >= 2 {
        let prev_close = closes[n - 2];
        if prev_close > 0.0 {
            features.insert("price_return".into(), json!((last_close - prev_close) / prev_close));
        }
    }

    let candle = series.candles.last()?;
    let fv = FeatureVector {
        schema_name: "feature_vector".to_string(),
        schema_version: "v1".to_string(),
        feature_set: "all".to_string(),
        feature_version: "v1".to_string(),
        symbol: series.symbol.clone(),
        window: series.interval.clone(),
        ts_feature: candle.time,
        features,
        source_refs: Vec::new(),
    };

    Some(fv)
}

pub fn compute_features_for_candles(
    candles: &[crate::ohlcv::OhlcvCandle],
    symbol: &str,
    interval: &str,
    min_history: usize,
) -> Vec<FeatureVector> {
    if candles.len() < min_history {
        return Vec::new();
    }

    let mut results = Vec::new();
    for end in min_history..=candles.len() {
        let sub_candles = &candles[..end];
        let mut series = OhlcvSeries::new(symbol, interval);
        series.candles = sub_candles.to_vec();
        if let Some(fv) = compute_all_features(&series) {
            results.push(fv);
        }
    }
    results
}
