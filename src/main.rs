#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "information_provider_whu=info,tower_http=info".into()),
        )
        .init();

    if let Err(error) = information_provider_whu::run_server().await {
        tracing::error!(%error, "server startup or runtime failed");
        std::process::exit(1);
    }
}
