use crate::binance::models::*;
use aicrypto_protocols::canonical_event::{CanonicalEvent, SourceType};
use aicrypto_protocols::market_snapshot::MarketSnapshot;
use chrono::Utc;
use serde_json::json;

pub fn kline_to_event(kline: &BinanceKline, symbol: &str) -> CanonicalEvent {
    CanonicalEvent::new(SourceType::Exchange, "binance", "kline", json!({
        "open_time": kline.open_time,
        "open": kline.open,
        "high": kline.high,
        "low": kline.low,
        "close": kline.close,
        "volume": kline.volume,
        "close_time": kline.close_time,
        "quote_volume": kline.quote_volume,
        "trades": kline.trades,
        "taker_buy_volume": kline.taker_buy_volume,
    }))
    .with_symbol(symbol.to_string())
}

pub fn ws_kline_to_event(kline: &WsKlineEvent) -> CanonicalEvent {
    CanonicalEvent::new(SourceType::Exchange, "binance", "kline", json!({
        "interval": kline.kline.interval,
        "open_time": kline.kline.start_time,
        "open": kline.kline.open,
        "high": kline.kline.high,
        "low": kline.kline.low,
        "close": kline.kline.close,
        "volume": kline.kline.volume,
        "close_time": kline.kline.close_time,
        "quote_volume": kline.kline.quote_volume,
        "trades": kline.kline.trades,
        "is_closed": kline.kline.is_closed,
    }))
    .with_symbol(kline.symbol.clone())
}

pub fn mark_price_to_event(mp: &MarkPriceResponse) -> CanonicalEvent {
    CanonicalEvent::new(SourceType::Exchange, "binance", "mark_price", json!({
        "mark_price": mp.mark_price,
        "index_price": mp.index_price,
        "last_funding_rate": mp.last_funding_rate,
        "next_funding_time": mp.next_funding_time,
    }))
    .with_symbol(mp.symbol.clone())
}

pub fn ws_mark_price_to_event(mp: &WsMarkPriceEvent) -> CanonicalEvent {
    CanonicalEvent::new(SourceType::Exchange, "binance", "mark_price", json!({
        "mark_price": mp.mark_price,
        "index_price": mp.index_price,
        "funding_rate": mp.funding_rate,
        "next_funding_time": mp.next_funding_time,
    }))
    .with_symbol(mp.symbol.clone())
}

pub fn funding_rate_to_event(fr: &FundingRateResponse) -> CanonicalEvent {
    CanonicalEvent::new(SourceType::Exchange, "binance", "funding_rate", json!({
        "funding_rate": fr.funding_rate,
        "funding_time": fr.funding_time,
        "mark_price": fr.mark_price,
    }))
    .with_symbol(fr.symbol.clone())
}

pub fn open_interest_to_event(oi: &OpenInterestResponse) -> CanonicalEvent {
    let mut payload = json!({
        "open_interest": oi.open_interest,
    });
    if let Some(t) = oi.time {
        payload["time"] = json!(t);
    }
    CanonicalEvent::new(SourceType::Exchange, "binance", "open_interest", payload)
        .with_symbol(oi.symbol.clone().unwrap_or_default())
}

pub fn ticker_to_event(ticker: &Ticker24hrResponse) -> CanonicalEvent {
    CanonicalEvent::new(SourceType::Exchange, "binance", "ticker_24hr", json!({
        "price_change": ticker.price_change,
        "price_change_percent": ticker.price_change_percent,
        "last_price": ticker.last_price,
        "volume": ticker.volume,
        "quote_volume": ticker.quote_volume,
        "high_price": ticker.high_price,
        "low_price": ticker.low_price,
        "trades": ticker.count,
    }))
    .with_symbol(ticker.symbol.clone())
}

pub fn build_market_snapshot(
    symbol: &str,
    ticker: Option<&Ticker24hrResponse>,
    mark_price: Option<&MarkPriceResponse>,
    oi: Option<&OpenInterestResponse>,
) -> MarketSnapshot {
    let mut snapshot = MarketSnapshot::new(symbol, "binance");
    if let Some(t) = ticker {
        snapshot.last_price = t.last_price.clone();
        snapshot.volume_24h = Some(t.quote_volume.clone());
        snapshot.ts_snapshot = Utc::now().timestamp_millis();
    }
    if let Some(mp) = mark_price {
        snapshot.mark_price = Some(mp.mark_price.clone());
        snapshot.index_price = Some(mp.index_price.clone());
        snapshot.funding_rate = mp.last_funding_rate.clone();
        snapshot.ts_snapshot = Utc::now().timestamp_millis();
    }
    if let Some(o) = oi {
        snapshot.open_interest = Some(o.open_interest.clone());
    }
    snapshot
}
