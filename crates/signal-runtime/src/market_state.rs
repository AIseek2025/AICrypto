use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketState {
    BullTrend,
    BullEuphoria,
    RangeNeutral,
    RiskOff,
    PanicSell,
    ShortSqueeze,
    EventDriven,
}

impl fmt::Display for MarketState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketState::BullTrend => write!(f, "BULL_TREND"),
            MarketState::BullEuphoria => write!(f, "BULL_EUPHORIA"),
            MarketState::RangeNeutral => write!(f, "RANGE_NEUTRAL"),
            MarketState::RiskOff => write!(f, "RISK_OFF"),
            MarketState::PanicSell => write!(f, "PANIC_SELL"),
            MarketState::ShortSqueeze => write!(f, "SHORT_SQUEEZE"),
            MarketState::EventDriven => write!(f, "EVENT_DRIVEN"),
        }
    }
}

pub fn classify_market_state(features: &serde_json::Map<String, serde_json::Value>) -> MarketState {
    let ma20_slope = get_f64(features, "ma20_slope");
    let rsi_14 = get_f64(features, "rsi_14");
    let atr_pct = get_f64(features, "atr_pct");
    let bb_position = get_f64(features, "bb_position");
    let volume_ratio_5 = get_f64(features, "volume_ratio_5");
    let hist_vol_20 = get_f64(features, "hist_vol_20");
    let bb_width = get_f64(features, "bb_width");
    let price_above_sma20 = get_bool(features, "price_above_sma20");
    let price_above_sma50 = get_bool(features, "price_above_sma50");
    let price_return = get_f64(features, "price_return");

    if atr_pct > 0.05 && hist_vol_20 > 1.2 {
        return MarketState::EventDriven;
    }

    if rsi_14 > 80.0 && bb_position > 0.95 && volume_ratio_5 > 3.0 && price_above_sma20 && price_above_sma50 {
        return MarketState::BullEuphoria;
    }

    if price_return < -0.03 && volume_ratio_5 > 3.0 && rsi_14 < 25.0 {
        return MarketState::PanicSell;
    }

    if price_return > 0.05 && volume_ratio_5 > 4.0 && rsi_14 < 40.0 {
        return MarketState::ShortSqueeze;
    }

    if ma20_slope < -0.001 && !price_above_sma20 && rsi_14 < 40.0 {
        return MarketState::RiskOff;
    }

    if bb_width < atr_pct * 5.0 && ma20_slope.abs() < 0.0005 && rsi_14 > 40.0 && rsi_14 < 60.0 {
        return MarketState::RangeNeutral;
    }

    if ma20_slope > 0.001 && price_above_sma20 && price_above_sma50 && rsi_14 > 50.0 {
        return MarketState::BullTrend;
    }

    if ma20_slope < -0.001 && !price_above_sma20 {
        return MarketState::RiskOff;
    }

    MarketState::RangeNeutral
}

fn get_f64(features: &serde_json::Map<String, serde_json::Value>, key: &str) -> f64 {
    features
        .get(key)
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

fn get_bool(features: &serde_json::Map<String, serde_json::Value>, key: &str) -> bool {
    features
        .get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_features(pairs: &[(&str, serde_json::Value)]) -> serde_json::Map<String, serde_json::Value> {
        let mut m = serde_json::Map::new();
        for (k, v) in pairs {
            m.insert(k.to_string(), v.clone());
        }
        m
    }

    #[test]
    fn test_bull_trend() {
        let features = make_features(&[
            ("ma20_slope", json!(0.005)),
            ("rsi_14", json!(62.0)),
            ("atr_pct", json!(0.015)),
            ("bb_position", json!(0.7)),
            ("volume_ratio_5", json!(1.2)),
            ("hist_vol_20", json!(0.3)),
            ("bb_width", json!(0.05)),
            ("price_above_sma20", json!(true)),
            ("price_above_sma50", json!(true)),
            ("price_return", json!(0.01)),
        ]);
        assert_eq!(classify_market_state(&features), MarketState::BullTrend);
    }

    #[test]
    fn test_bull_euphoria() {
        let features = make_features(&[
            ("ma20_slope", json!(0.01)),
            ("rsi_14", json!(88.0)),
            ("atr_pct", json!(0.02)),
            ("bb_position", json!(0.98)),
            ("volume_ratio_5", json!(5.0)),
            ("hist_vol_20", json!(0.5)),
            ("bb_width", json!(0.08)),
            ("price_above_sma20", json!(true)),
            ("price_above_sma50", json!(true)),
            ("price_return", json!(0.02)),
        ]);
        assert_eq!(classify_market_state(&features), MarketState::BullEuphoria);
    }

    #[test]
    fn test_panic_sell() {
        let features = make_features(&[
            ("ma20_slope", json!(-0.008)),
            ("rsi_14", json!(18.0)),
            ("atr_pct", json!(0.04)),
            ("bb_position", json!(0.05)),
            ("volume_ratio_5", json!(5.0)),
            ("hist_vol_20", json!(0.6)),
            ("bb_width", json!(0.1)),
            ("price_above_sma20", json!(false)),
            ("price_above_sma50", json!(false)),
            ("price_return", json!(-0.05)),
        ]);
        assert_eq!(classify_market_state(&features), MarketState::PanicSell);
    }

    #[test]
    fn test_risk_off() {
        let features = make_features(&[
            ("ma20_slope", json!(-0.005)),
            ("rsi_14", json!(35.0)),
            ("atr_pct", json!(0.02)),
            ("bb_position", json!(0.2)),
            ("volume_ratio_5", json!(1.5)),
            ("hist_vol_20", json!(0.4)),
            ("bb_width", json!(0.06)),
            ("price_above_sma20", json!(false)),
            ("price_above_sma50", json!(false)),
            ("price_return", json!(-0.01)),
        ]);
        assert_eq!(classify_market_state(&features), MarketState::RiskOff);
    }

    #[test]
    fn test_range_neutral() {
        let features = make_features(&[
            ("ma20_slope", json!(0.0001)),
            ("rsi_14", json!(50.0)),
            ("atr_pct", json!(0.008)),
            ("bb_position", json!(0.5)),
            ("volume_ratio_5", json!(0.9)),
            ("hist_vol_20", json!(0.15)),
            ("bb_width", json!(0.02)),
            ("price_above_sma20", json!(true)),
            ("price_above_sma50", json!(true)),
            ("price_return", json!(0.002)),
        ]);
        assert_eq!(classify_market_state(&features), MarketState::RangeNeutral);
    }

    #[test]
    fn test_short_squeeze() {
        let features = make_features(&[
            ("ma20_slope", json!(0.002)),
            ("rsi_14", json!(35.0)),
            ("atr_pct", json!(0.03)),
            ("bb_position", json!(0.6)),
            ("volume_ratio_5", json!(6.0)),
            ("hist_vol_20", json!(0.7)),
            ("bb_width", json!(0.09)),
            ("price_above_sma20", json!(true)),
            ("price_above_sma50", json!(false)),
            ("price_return", json!(0.08)),
        ]);
        assert_eq!(classify_market_state(&features), MarketState::ShortSqueeze);
    }

    #[test]
    fn test_event_driven() {
        let features = make_features(&[
            ("ma20_slope", json!(0.001)),
            ("rsi_14", json!(55.0)),
            ("atr_pct", json!(0.06)),
            ("bb_position", json!(0.5)),
            ("volume_ratio_5", json!(2.0)),
            ("hist_vol_20", json!(1.5)),
            ("bb_width", json!(0.12)),
            ("price_above_sma20", json!(true)),
            ("price_above_sma50", json!(true)),
            ("price_return", json!(0.015)),
        ]);
        assert_eq!(classify_market_state(&features), MarketState::EventDriven);
    }
}
