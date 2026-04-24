use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvCandle {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub quote_volume: f64,
    pub trades: i64,
}

impl OhlcvCandle {
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    pub fn body(&self) -> f64 {
        (self.close - self.open).abs()
    }

    pub fn upper_shadow(&self) -> f64 {
        self.high - self.open.max(self.close)
    }

    pub fn lower_shadow(&self) -> f64 {
        self.open.min(self.close) - self.low
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn return_pct(&self) -> f64 {
        if self.open == 0.0 { return 0.0; }
        (self.close - self.open) / self.open
    }

    pub fn close_return(&self, prev_close: f64) -> f64 {
        if prev_close == 0.0 { return 0.0; }
        (self.close - prev_close) / prev_close
    }

    pub fn vwap(&self) -> f64 {
        if self.volume == 0.0 { return self.close; }
        self.quote_volume / self.volume
    }
}

#[derive(Debug, Clone)]
pub struct OhlcvSeries {
    pub symbol: String,
    pub interval: String,
    pub candles: Vec<OhlcvCandle>,
}

impl OhlcvSeries {
    pub fn new(symbol: &str, interval: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            candles: Vec::new(),
        }
    }

    pub fn closes(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.close).collect()
    }

    pub fn highs(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.high).collect()
    }

    pub fn lows(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.low).collect()
    }

    pub fn volumes(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.volume).collect()
    }

    pub fn quote_volumes(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.quote_volume).collect()
    }

    pub fn returns(&self) -> Vec<f64> {
        self.closes().windows(2).map(|w| {
            if w[0] == 0.0 { 0.0 } else { (w[1] - w[0]) / w[0] }
        }).collect()
    }

    pub fn typical_prices(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.typical_price()).collect()
    }

    pub fn last(&self) -> Option<&OhlcvCandle> {
        self.candles.last()
    }

    pub fn len(&self) -> usize {
        self.candles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    pub fn tail(&self, n: usize) -> &[OhlcvCandle] {
        let start = self.candles.len().saturating_sub(n);
        &self.candles[start..]
    }
}
