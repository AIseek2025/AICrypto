mod handlers;

use axum::http;
use aicrypto_foundation::config::AppConfig;
use aicrypto_foundation::observability;
use handlers::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init_tracing("api-gateway");
    let config = AppConfig::from_env()?;

    tracing::info!(env = ?config.environment, "api-gateway starting");

    let project_root = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let skills_dir = project_root.join("skills");
    let state = Arc::new(RwLock::new(AppState::new(&skills_dir)?));

    let allowed_origin = std::env::var("CORS_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let cors = CorsLayer::new()
        .allow_origin(allowed_origin.parse::<http::HeaderValue>().unwrap_or_else(|_| {
            http::HeaderValue::from_static("http://localhost:3000")
        }))
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = handlers::router(state.clone()).layer(cors);

    let port: u16 = std::env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    #[cfg(feature = "server")]
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    #[cfg(feature = "server")]
    {
        tracing::info!("api-gateway listening on http://0.0.0.0:{}", port);
        axum::serve(listener, app).await?;
    }

    #[cfg(not(feature = "server"))]
    {
        tracing::info!("api-gateway initialized on port {} (demo mode — not serving)", port);
    }

    Ok(())
}
