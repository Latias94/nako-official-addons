use std::net::SocketAddr;

use anyhow::Context;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = nako_resource_search::Config::from_env();
    let addr: SocketAddr = config.listen_addr.parse().with_context(|| {
        format!(
            "invalid NAKO_RESOURCE_SEARCH_LISTEN_ADDR: {}",
            config.listen_addr
        )
    })?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(%addr, "starting Nako Resource Search Addon");
    axum::serve(listener, nako_resource_search::router(config))
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
