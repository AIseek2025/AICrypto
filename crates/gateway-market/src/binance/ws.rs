use crate::binance::models::*;
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing;

#[derive(Debug, Clone)]
pub enum WsMessage {
    Kline(WsKlineEvent),
    MarkPrice(WsMarkPriceEvent),
    AggTrade(WsAggTradeEvent),
    Ticker(WsTickerEvent),
    Depth(WsDepthEvent),
    Raw(String),
}

pub struct WsMarketStream {
    ws_url: String,
    reconnect_delay: Duration,
    max_reconnect_delay: Duration,
    tx: mpsc::Sender<WsMessage>,
}

impl WsMarketStream {
    pub fn new(ws_url: &str, tx: mpsc::Sender<WsMessage>) -> Self {
        Self {
            ws_url: ws_url.trim_end_matches('/').to_string(),
            reconnect_delay: Duration::from_secs(1),
            max_reconnect_delay: Duration::from_secs(60),
            tx,
        }
    }

    pub async fn run(&self, subscriptions: Vec<String>) -> Result<()> {
        let stream_url = format!("{}/{}", self.ws_url, subscriptions.join("/"));
        let mut delay = self.reconnect_delay;

        loop {
            match self.connect_and_listen(&stream_url).await {
                Ok(()) => {
                    tracing::warn!("WebSocket disconnected, reconnecting...");
                }
                Err(e) => {
                    tracing::error!(error = %e, "WebSocket error, reconnecting...");
                }
            }

            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(self.max_reconnect_delay);
            tracing::info!(delay_secs = delay.as_secs(), "reconnecting WebSocket");
        }
    }

    async fn connect_and_listen(&self, url: &str) -> Result<()> {
        tracing::info!(url = %url, "connecting to WebSocket");

        let (ws_stream, _) = connect_async(url)
            .await
            .context("failed to connect WebSocket")?;

        tracing::info!("WebSocket connected");
        let (_, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_text(&text).await {
                        tracing::debug!(error = %e, "failed to handle WS message");
                    }
                }
                Ok(Message::Ping(data)) => {
                    tracing::trace!("received ping");
                }
                Ok(Message::Close(reason)) => {
                    tracing::warn!(?reason, "WebSocket closed by server");
                    return Ok(());
                }
                Err(e) => {
                    tracing::error!(error = %e, "WebSocket read error");
                    return Err(e.into());
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn handle_text(&self, text: &str) -> Result<()> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        let event_type = value.get("e").and_then(|v| v.as_str()).unwrap_or("");

        let msg = match event_type {
            "kline" => {
                let data: WsKlineEvent = serde_json::from_value(value)?;
                WsMessage::Kline(data)
            }
            "markPriceUpdate" => {
                let data: WsMarkPriceEvent = serde_json::from_value(value)?;
                WsMessage::MarkPrice(data)
            }
            "aggTrade" => {
                let data: WsAggTradeEvent = serde_json::from_value(value)?;
                WsMessage::AggTrade(data)
            }
            "24hrTicker" => {
                let data: WsTickerEvent = serde_json::from_value(value)?;
                WsMessage::Ticker(data)
            }
            "depthUpdate" => {
                let data: WsDepthEvent = serde_json::from_value(value)?;
                WsMessage::Depth(data)
            }
            _ => {
                WsMessage::Raw(text.to_string())
            }
        };

        if self.tx.send(msg).await.is_err() {
            tracing::warn!("WS message receiver dropped");
        }
        Ok(())
    }

    pub fn build_kline_stream(symbol: &str, interval: &str) -> String {
        format!("{}@kline_{}", symbol.to_lowercase(), interval)
    }

    pub fn build_mark_price_stream(symbol: &str) -> String {
        format!("{}@markPrice@1s", symbol.to_lowercase())
    }

    pub fn build_agg_trade_stream(symbol: &str) -> String {
        format!("{}@aggTrade", symbol.to_lowercase())
    }

    pub fn build_ticker_stream(symbol: &str) -> String {
        format!("{}@ticker", symbol.to_lowercase())
    }

    pub fn build_depth_stream(symbol: &str, speed: &str) -> String {
        format!("{}@depth@{}", symbol.to_lowercase(), speed)
    }

    pub fn build_all_tickers_stream() -> String {
        "!ticker@arr".to_string()
    }
}
