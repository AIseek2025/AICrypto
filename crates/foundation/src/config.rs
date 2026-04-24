use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub environment: Environment,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub nats: NatsConfig,
    pub binance: BinanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Dev,
    Backtest,
    Paper,
    Prod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceConfig {
    pub rest_base_url: String,
    pub ws_base_url: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub testnet: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        Ok(Self {
            environment: match std::env::var("AICRYPTO_ENV").unwrap_or_default().as_str() {
                "backtest" => Environment::Backtest,
                "paper" => Environment::Paper,
                "prod" => Environment::Prod,
                _ => Environment::Dev,
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://aicrypto:aicrypto@localhost:5432/aicrypto".to_string()),
                max_connections: std::env::var("DB_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
            },
            redis: RedisConfig {
                url: std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            nats: NatsConfig {
                url: std::env::var("NATS_URL")
                    .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            },
            binance: BinanceConfig {
                rest_base_url: std::env::var("BINANCE_REST_URL")
                    .unwrap_or_else(|_| "https://testnet.binancefuture.com".to_string()),
                ws_base_url: std::env::var("BINANCE_WS_URL")
                    .unwrap_or_else(|_| "wss://stream.binancefuture.com/ws".to_string()),
                api_key: std::env::var("BINANCE_API_KEY").ok(),
                api_secret: std::env::var("BINANCE_API_SECRET").ok(),
                testnet: std::env::var("BINANCE_TESTNET")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        })
    }
}
