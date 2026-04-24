use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use tracing;

#[derive(Debug, Clone)]
pub struct UserStreamClient {
    client: Client,
    base_url: String,
    api_key: String,
    listen_key: Option<String>,
}

impl UserStreamClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            listen_key: None,
        }
    }

    pub async fn create_listen_key(&mut self) -> Result<String> {
        let url = format!("{}/fapi/v1/listenKey", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await
            .context("failed to create listenKey")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("create listenKey failed: status={}, body={}", status, body);
        }

        let data: serde_json::Value = resp.json().await?;
        let listen_key = data["listenKey"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("listenKey not found in response"))?
            .to_string();

        tracing::info!(listen_key = %listen_key, "listenKey created");
        self.listen_key = Some(listen_key.clone());
        Ok(listen_key)
    }

    pub async fn keepalive(&self) -> Result<()> {
        let listen_key = self.listen_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("no listenKey available"))?;

        let url = format!("{}/fapi/v1/listenKey", self.base_url);
        let resp = self
            .client
            .put(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&[("listenKey", listen_key)])
            .send()
            .await
            .context("failed to keepalive listenKey")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("listenKey keepalive failed: status={}, body={}", status, body);
        }

        tracing::debug!("listenKey keepalive sent");
        Ok(())
    }

    pub fn get_ws_url(&self, ws_base_url: &str) -> Option<String> {
        self.listen_key
            .as_ref()
            .map(|lk| format!("{}/{}", ws_base_url.trim_end_matches('/'), lk))
    }
}
