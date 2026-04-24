pub fn sma(data: &[f64], period: usize) -> Vec<Option<f64>> {
    data.windows(period)
        .map(|w| Some(w.iter().sum::<f64>() / period as f64))
        .collect()
}

pub fn ema(data: &[f64], period: usize) -> Vec<Option<f64>> {
    if data.len() < period || period == 0 {
        return vec![None; data.len()];
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());

    let first_ema: f64 = data[..period].iter().sum::<f64>() / period as f64;
    for _ in 0..period - 1 {
        result.push(None);
    }
    result.push(Some(first_ema));

    let mut prev = first_ema;
    for i in period..data.len() {
        let val = data[i] * k + prev * (1.0 - k);
        result.push(Some(val));
        prev = val;
    }
    result
}

pub fn rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    if closes.len() < period + 1 {
        return vec![None; closes.len()];
    }
    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..closes.len() {
        let diff = closes[i] - closes[i - 1];
        gains.push(if diff > 0.0 { diff } else { 0.0 });
        losses.push(if diff < 0.0 { -diff } else { 0.0 });
    }

    let mut result = Vec::with_capacity(closes.len());
    result.push(None);

    let avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    for _ in 1..period {
        result.push(None);
    }

    let mut current_avg_gain = avg_gain;
    let mut current_avg_loss = avg_loss;

    let rs = if current_avg_loss == 0.0 { 100.0 } else { current_avg_gain / current_avg_loss };
    result.push(Some(100.0 - 100.0 / (1.0 + rs)));

    for i in period..gains.len() {
        current_avg_gain = (current_avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        current_avg_loss = (current_avg_loss * (period - 1) as f64 + losses[i]) / period as f64;
        let rs = if current_avg_loss == 0.0 { 100.0 } else { current_avg_gain / current_avg_loss };
        result.push(Some(100.0 - 100.0 / (1.0 + rs)));
    }

    result
}

pub fn atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<Option<f64>> {
    if highs.len() < 2 || period == 0 {
        return vec![None; highs.len()];
    }

    let mut true_ranges = Vec::with_capacity(highs.len());
    true_ranges.push(highs[0] - lows[0]);

    for i in 1..highs.len() {
        let tr = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
        true_ranges.push(tr);
    }

    let mut result = Vec::with_capacity(highs.len());
    for _ in 0..period.min(true_ranges.len()).saturating_sub(1) {
        result.push(None);
    }

    if true_ranges.len() >= period {
        let first_atr: f64 = true_ranges[..period].iter().sum::<f64>() / period as f64;
        result.push(Some(first_atr));

        let mut prev = first_atr;
        for i in period..true_ranges.len() {
            let val = (prev * (period - 1) as f64 + true_ranges[i]) / period as f64;
            result.push(Some(val));
            prev = val;
        }
    }

    result
}

pub fn bollinger_bands(closes: &[f64], period: usize, std_dev_mult: f64) -> Vec<Option<(f64, f64, f64)>> {
    closes.windows(period).map(|w| {
        let mean = w.iter().sum::<f64>() / period as f64;
        let variance = w.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();
        Some((mean - std_dev_mult * std_dev, mean, mean + std_dev_mult * std_dev))
    }).collect()
}

pub fn macd(closes: &[f64], fast: usize, slow: usize, signal: usize) -> Vec<Option<(f64, f64, f64)>> {
    let fast_ema = ema(closes, fast);
    let slow_ema = ema(closes, slow);

    let macd_line: Vec<Option<f64>> = fast_ema.iter().zip(slow_ema.iter())
        .map(|(f, s)| match (f, s) {
            (Some(fv), Some(sv)) => Some(fv - sv),
            _ => None,
        })
        .collect();

    let macd_values: Vec<f64> = macd_line.iter()
        .filter_map(|v| *v)
        .collect();

    let signal_line = ema(&macd_values, signal);

    let mut result = Vec::with_capacity(closes.len());
    let mut sig_idx = 0;
    for ml in &macd_line {
        match ml {
            Some(v) => {
                if sig_idx < signal_line.len() {
                    match signal_line[sig_idx] {
                        Some(s) => result.push(Some((*v, s, *v - s))),
                        None => result.push(None),
                    }
                    sig_idx += 1;
                } else {
                    result.push(None);
                }
            }
            None => result.push(None),
        }
    }
    result
}

pub fn stochastic(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    k_period: usize,
    d_period: usize,
) -> Vec<Option<(f64, f64)>> {
    let raw_k: Vec<Option<f64>> = highs.windows(k_period).zip(lows.windows(k_period))
        .zip(closes.windows(k_period))
        .map(|((hw, lw), cw)| {
            let highest = hw.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let lowest = lw.iter().cloned().fold(f64::INFINITY, f64::min);
            let range = highest - lowest;
            if range == 0.0 {
                Some(50.0)
            } else {
                Some((cw.last()? - lowest) / range * 100.0)
            }
        })
        .collect();

    let d_values: Vec<f64> = raw_k.iter().filter_map(|v| *v).collect();
    let d_line = sma(&d_values, d_period);

    let mut result = Vec::with_capacity(closes.len());
    let mut d_idx = 0;
    for k in &raw_k {
        match k {
            Some(kv) => {
                if d_idx < d_line.len() {
                    match d_line[d_idx] {
                        Some(dv) => result.push(Some((*kv, dv))),
                        None => result.push(None),
                    }
                    d_idx += 1;
                } else {
                    result.push(Some((*kv, *kv)));
                }
            }
            None => result.push(None),
        }
    }
    result
}

pub fn volume_sma(volumes: &[f64], period: usize) -> Vec<Option<f64>> {
    sma(volumes, period)
}

pub fn volume_ratio(volumes: &[f64], period: usize) -> Vec<Option<f64>> {
    let avg = sma(volumes, period);
    volumes.iter().enumerate().map(|(i, &v)| {
        match avg.get(i).and_then(|a| *a) {
            Some(a) if a > 0.0 => Some(v / a),
            _ => None,
        }
    }).collect()
}

pub fn obv(closes: &[f64], volumes: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0; closes.len()];
    if closes.is_empty() { return result; }
    result[0] = volumes[0];
    for i in 1..closes.len() {
        if closes[i] > closes[i - 1] {
            result[i] = result[i - 1] + volumes[i];
        } else if closes[i] < closes[i - 1] {
            result[i] = result[i - 1] - volumes[i];
        } else {
            result[i] = result[i - 1];
        }
    }
    result
}

pub fn rolling_max(data: &[f64], period: usize) -> Vec<Option<f64>> {
    data.windows(period)
        .map(|w| Some(w.iter().cloned().fold(f64::NEG_INFINITY, f64::max)))
        .collect()
}

pub fn rolling_min(data: &[f64], period: usize) -> Vec<Option<f64>> {
    data.windows(period)
        .map(|w| Some(w.iter().cloned().fold(f64::INFINITY, f64::min)))
        .collect()
}

pub fn historical_volatility(returns: &[f64], period: usize) -> Vec<Option<f64>> {
    returns.windows(period).map(|w| {
        let mean = w.iter().sum::<f64>() / w.len() as f64;
        let variance = w.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (w.len() - 1) as f64;
        Some(variance.sqrt() * (252.0_f64).sqrt())
    }).collect()
}

pub fn slope(data: &[f64], period: usize) -> Vec<Option<f64>> {
    data.windows(period).map(|w| {
        let n = w.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = w.iter().sum::<f64>() / n;
        let mut num = 0.0;
        let mut den = 0.0;
        for (i, &y) in w.iter().enumerate() {
            let x = i as f64;
            num += (x - x_mean) * (y - y_mean);
            den += (x - x_mean).powi(2);
        }
        if den == 0.0 { None } else { Some(num / den) }
    }).collect()
}
