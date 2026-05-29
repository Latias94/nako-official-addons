use std::net::SocketAddr;

use anyhow::Context;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args()
        .nth(1)
        .is_some_and(|arg| arg == "render-drift-cases" || arg == "--render-drift-cases")
    {
        let config = nako_metadata_scraper::Config::from_env();
        let cases =
            nako_metadata_scraper::providers::browser_worker_render_drift_cases_from_env(&config);
        println!("{}", serde_json::to_string_pretty(&cases)?);
        return Ok(());
    }

    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = nako_metadata_scraper::Config::from_env();
    let addr: SocketAddr = config.listen_addr.parse().with_context(|| {
        format!(
            "invalid NAKO_METADATA_SCRAPER_LISTEN_ADDR: {}",
            config.listen_addr
        )
    })?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(%addr, "starting Nako Metadata Scraper Addon");
    axum::serve(listener, nako_metadata_scraper::router(config))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
