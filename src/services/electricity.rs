mod api;
mod client;
mod config;
mod crypto;
mod error;
mod models;
mod ports;
mod room_service;
mod scheduler;
mod store;
mod token;
mod transport;
mod whu_service;

pub use client::WhuApiClient;
pub use error::{Error, Result};
pub use models::{
    ArchitectureSummary, AreaSummary, DayUsageItem, ReserveInfo, RoomMeterInfo, RoomSummary,
};
pub use room_service::RoomService;
pub use whu_service::WhuApiService;

/// Configuration for the WHU electricity API client.
#[derive(Clone, Debug)]
pub struct WhuApiOptions {
    pub base_url: String,
    pub account_id: String,
    pub account_pass: String,
    pub sys_id: String,
    pub sm2_public_key: String,
}

impl Default for WhuApiOptions {
    fn default() -> Self {
        Self {
            base_url: "http://zwhqbsd.whu.edu.cn/ICBS_V2_Server".into(),
            account_id: "ph".into(),
            account_pass: "phAPI".into(),
            sys_id: "1".into(),
            sm2_public_key: "04b238b7d42c87a25e1a4eaddca81e8f33fd95773ccd471408e4195db62aa4085f6e0a3b695cad8d60acece1af348f534b7f72d312f2966b144248c9c590c930fc".into(),
        }
    }
}

pub(crate) async fn run() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::sync::Arc;
    use tokio::net::TcpListener;

    let config = config::AppConfig::from_env();
    let store = store::Store::connect(&config.database_url).await?;
    let client = WhuApiClient::new(config.whu.clone())?;
    let whu = WhuApiService::new(client.clone());
    let rooms = RoomService::new(client);
    let state = api::AppState::new(store, Arc::new(whu), Arc::new(rooms), config.clone());

    api::restart_pending_backfills(state.clone()).await;
    tokio::spawn(scheduler::run(state.clone()));

    let listener = TcpListener::bind(&config.bind_address).await?;
    tracing::info!(address = %config.bind_address, "Infrastructure service listening");
    axum::serve(listener, api::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to register SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
