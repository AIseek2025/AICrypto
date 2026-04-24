use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

pub const SUBJECT_CANONICAL_EVENT: &str = "aicrypto.events.canonical";
pub const SUBJECT_FEATURE_VECTOR: &str = "aicrypto.features.vector";
pub const SUBJECT_SIGNAL_EVENT: &str = "aicrypto.signals.event";
pub const SUBJECT_ORDER_INTENT: &str = "aicrypto.orders.intent";
pub const SUBJECT_RISK_DECISION: &str = "aicrypto.risk.decision";
pub const SUBJECT_EXECUTION_REPORT: &str = "aicrypto.execution.report";

pub struct BusClient {
    nats_url: String,
}

impl BusClient {
    pub fn new(nats_url: impl Into<String>) -> Self {
        Self {
            nats_url: nats_url.into(),
        }
    }

    pub async fn connect(&self) -> Result<async_nats::Client> {
        let client = async_nats::connect(&self.nats_url).await?;
        Ok(client)
    }
}

pub async fn publish<T: Serialize>(
    client: &async_nats::Client,
    subject: &str,
    payload: &T,
) -> Result<()> {
    let data = serde_json::to_vec(payload)?;
    client
        .publish(subject.to_string(), data.into())
        .await?;
    tracing::debug!(subject = subject, "published message");
    Ok(())
}

pub async fn subscribe(
    client: &async_nats::Client,
    subject: &str,
) -> Result<async_nats::Subscriber> {
    let subscriber = client.subscribe(subject.to_string()).await?;
    tracing::info!(subject = subject, "subscribed to subject");
    Ok(subscriber)
}

pub fn deserialize_message<T: DeserializeOwned>(
    msg: &async_nats::Message,
) -> Result<T> {
    let data: T = serde_json::from_slice(&msg.payload)?;
    Ok(data)
}
