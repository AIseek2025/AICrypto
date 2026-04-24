use tracing_subscriber::EnvFilter;

pub fn init_tracing(service_name: &str) {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .init();
    tracing::info!("tracing initialized for {}", service_name);
}
