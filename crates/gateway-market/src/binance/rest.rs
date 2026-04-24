use crate::binance::models::*;
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use tracing;

#[derive(Debug, Clone)]
pub struct BinanceRestClient {
    client: Client,
    base_url: String,
}

impl BinanceRestClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn get_exchange_info(&self) -> Result<ExchangeInfoResponse> {
        let url = format!("{}/fapi/v1/exchangeInfo", self.base_url);
        tracing::debug!(url = %url, "fetching exchangeInfo");
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("failed to send exchangeInfo request")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("exchangeInfo request failed: status={}, body={}", status, body);
        }
        let data: ExchangeInfoResponse = resp
            .json()
            .await
            .context("failed to parse exchangeInfo response")?;
        tracing::info!(symbol_count = data.symbols.len(), "exchangeInfo fetched");
        Ok(data)
    }

    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<BinanceKline>> {
        let url = format!("{}/fapi/v1/klines", self.base_url);
        let mut params = vec![
            ("symbol".to_string(), symbol.to_string()),
            ("interval".to_string(), interval.to_string()),
        ];
        if let Some(st) = start_time {
            params.push(("startTime".to_string(), st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime".to_string(), et.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit".to_string(), l.to_string()));
        }

        tracing::debug!(symbol = %symbol, interval = %interval, "fetching klines");
        let resp = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("failed to send klines request")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("klines request failed: status={}, body={}", status, body);
        }

        let raw: Vec<Vec<serde_json::Value>> = resp
            .json()
            .await
            .context("failed to parse klines response")?;

        let klines: Vec<BinanceKline> = raw
            .iter()
            .filter_map(|r| BinanceKline::from_raw(r))
            .collect();

        tracing::debug!(symbol = %symbol, interval = %interval, count = klines.len(), "klines fetched");
        Ok(klines)
    }

    pub async fn get_mark_price(&self, symbol: Option<&str>) -> Result<Vec<MarkPriceResponse>> {
        let url = format!("{}/fapi/v1/premiumIndex", self.base_url);
        let mut req = self.client.get(&url);
        if let Some(s) = symbol {
            req = req.query(&[("symbol", s)]);
        }
        let resp = req
            .send()
            .await
            .context("failed to send markPrice request")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("markPrice request failed: status={}, body={}", status, body);
        }

        let body = resp.text().await.context("failed to read markPrice body")?;
        let data: Vec<MarkPriceResponse> = if symbol.is_some() {
            let single: MarkPriceResponse = serde_json::from_str(&body)
                .context(format!("failed to parse markPrice response: {}", &body[..body.len().min(200)]))?;
            vec![single]
        } else {
            serde_json::from_str(&body)
                .context(format!("failed to parse markPrice response: {}", &body[..body.len().min(200)]))?
        };
        Ok(data)
    }

    pub async fn get_funding_rate(
        &self,
        symbol: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRateResponse>> {
        let url = format!("{}/fapi/v1/fundingRate", self.base_url);
        let mut params = vec![("symbol".to_string(), symbol.to_string())];
        if let Some(st) = start_time {
            params.push(("startTime".to_string(), st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime".to_string(), et.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit".to_string(), l.to_string()));
        }

        let resp = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("failed to send fundingRate request")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("fundingRate request failed: status={}, body={}", status, body);
        }

        let data: Vec<FundingRateResponse> = resp
            .json()
            .await
            .context("failed to parse fundingRate response")?;
        Ok(data)
    }

    pub async fn get_open_interest(&self, symbol: &str) -> Result<OpenInterestResponse> {
        let url = format!("{}/fapi/v1/openInterest", self.base_url);
        let resp = self
            .client
            .get(&url)
            .query(&[("symbol", symbol)])
            .send()
            .await
            .context("failed to send openInterest request")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("openInterest request failed: status={}, body={}", status, body);
        }

        let data: OpenInterestResponse = resp
            .json()
            .await
            .context("failed to parse openInterest response")?;
        Ok(data)
    }

    pub async fn get_24hr_ticker(&self, symbol: Option<&str>) -> Result<Vec<Ticker24hrResponse>> {
        let url = format!("{}/fapi/v1/ticker/24hr", self.base_url);
        let mut req = self.client.get(&url);
        if let Some(s) = symbol {
            req = req.query(&[("symbol", s)]);
        }
        let resp = req
            .send()
            .await
            .context("failed to send 24hr ticker request")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("24hr ticker request failed: status={}, body={}", status, body);
        }

        let body = resp.text().await.context("failed to read ticker body")?;
        let data: Vec<Ticker24hrResponse> = if symbol.is_some() {
            let single: Ticker24hrResponse = serde_json::from_str(&body)
                .context(format!("failed to parse 24hr ticker response: {}", &body[..body.len().min(200)]))?;
            vec![single]
        } else {
            serde_json::from_str(&body)
                .context(format!("failed to parse 24hr ticker response: {}", &body[..body.len().min(200)]))?
        };
        Ok(data)
    }
}
