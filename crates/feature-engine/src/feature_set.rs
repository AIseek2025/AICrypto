use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSetDef {
    pub name: String,
    pub version: String,
    pub description: String,
    pub features: Vec<FeatureDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDef {
    pub name: String,
    pub category: FeatureCategory,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureCategory {
    Price,
    Volume,
    Volatility,
    Momentum,
    Trend,
    Derivatives,
    Structure,
}

pub fn basic_trend_features() -> FeatureSetDef {
    FeatureSetDef {
        name: "basic_trend".to_string(),
        version: "v1".to_string(),
        description: "基础趋势特征集".to_string(),
        features: vec![
            FeatureDef { name: "price_return".into(), category: FeatureCategory::Price, description: "收益率".into() },
            FeatureDef { name: "sma_10".into(), category: FeatureCategory::Trend, description: "10周期均线".into() },
            FeatureDef { name: "sma_20".into(), category: FeatureCategory::Trend, description: "20周期均线".into() },
            FeatureDef { name: "sma_50".into(), category: FeatureCategory::Trend, description: "50周期均线".into() },
            FeatureDef { name: "ema_12".into(), category: FeatureCategory::Trend, description: "12周期指数均线".into() },
            FeatureDef { name: "ema_26".into(), category: FeatureCategory::Trend, description: "26周期指数均线".into() },
            FeatureDef { name: "ma20_slope".into(), category: FeatureCategory::Trend, description: "20均线斜率".into() },
            FeatureDef { name: "price_above_sma20".into(), category: FeatureCategory::Trend, description: "价格是否在20均线上方".into() },
            FeatureDef { name: "price_above_sma50".into(), category: FeatureCategory::Trend, description: "价格是否在50均线上方".into() },
            FeatureDef { name: "sma_cross_signal".into(), category: FeatureCategory::Trend, description: "均线交叉信号".into() },
        ],
    }
}

pub fn momentum_features() -> FeatureSetDef {
    FeatureSetDef {
        name: "momentum".to_string(),
        version: "v1".to_string(),
        description: "动量特征集".to_string(),
        features: vec![
            FeatureDef { name: "rsi_14".into(), category: FeatureCategory::Momentum, description: "14周期RSI".into() },
            FeatureDef { name: "macd_line".into(), category: FeatureCategory::Momentum, description: "MACD线".into() },
            FeatureDef { name: "macd_signal".into(), category: FeatureCategory::Momentum, description: "MACD信号线".into() },
            FeatureDef { name: "macd_hist".into(), category: FeatureCategory::Momentum, description: "MACD柱状图".into() },
            FeatureDef { name: "stoch_k".into(), category: FeatureCategory::Momentum, description: "随机指标K".into() },
            FeatureDef { name: "stoch_d".into(), category: FeatureCategory::Momentum, description: "随机指标D".into() },
        ],
    }
}

pub fn volatility_features() -> FeatureSetDef {
    FeatureSetDef {
        name: "volatility".to_string(),
        version: "v1".to_string(),
        description: "波动率特征集".to_string(),
        features: vec![
            FeatureDef { name: "atr_14".into(), category: FeatureCategory::Volatility, description: "14周期ATR".into() },
            FeatureDef { name: "atr_pct".into(), category: FeatureCategory::Volatility, description: "ATR占价格百分比".into() },
            FeatureDef { name: "bb_upper".into(), category: FeatureCategory::Volatility, description: "布林带上轨".into() },
            FeatureDef { name: "bb_lower".into(), category: FeatureCategory::Volatility, description: "布林带下轨".into() },
            FeatureDef { name: "bb_width".into(), category: FeatureCategory::Volatility, description: "布林带宽度".into() },
            FeatureDef { name: "bb_position".into(), category: FeatureCategory::Volatility, description: "价格在布林带中位置".into() },
            FeatureDef { name: "hist_vol_20".into(), category: FeatureCategory::Volatility, description: "20周期历史波动率".into() },
        ],
    }
}

pub fn volume_features() -> FeatureSetDef {
    FeatureSetDef {
        name: "volume".to_string(),
        version: "v1".to_string(),
        description: "成交量特征集".to_string(),
        features: vec![
            FeatureDef { name: "volume_ratio_5".into(), category: FeatureCategory::Volume, description: "5周期量比".into() },
            FeatureDef { name: "volume_ratio_20".into(), category: FeatureCategory::Volume, description: "20周期量比".into() },
            FeatureDef { name: "obv_slope_10".into(), category: FeatureCategory::Volume, description: "OBV 10周期斜率".into() },
        ],
    }
}

pub fn structure_features() -> FeatureSetDef {
    FeatureSetDef {
        name: "structure".to_string(),
        version: "v1".to_string(),
        description: "价格结构特征集".to_string(),
        features: vec![
            FeatureDef { name: "high_20d".into(), category: FeatureCategory::Structure, description: "20周期最高价".into() },
            FeatureDef { name: "low_20d".into(), category: FeatureCategory::Structure, description: "20周期最低价".into() },
            FeatureDef { name: "pct_from_high_20d".into(), category: FeatureCategory::Structure, description: "距20高点百分比".into() },
            FeatureDef { name: "pct_from_low_20d".into(), category: FeatureCategory::Structure, description: "距20低点百分比".into() },
            FeatureDef { name: "breakout_high_20d".into(), category: FeatureCategory::Structure, description: "是否突破20高点".into() },
            FeatureDef { name: "breakdown_low_20d".into(), category: FeatureCategory::Structure, description: "是否跌破20低点".into() },
        ],
    }
}

pub fn all_feature_sets() -> Vec<FeatureSetDef> {
    vec![
        basic_trend_features(),
        momentum_features(),
        volatility_features(),
        volume_features(),
        structure_features(),
    ]
}
