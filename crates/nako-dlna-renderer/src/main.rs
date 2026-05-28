use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

use nako_dlna_renderer::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let config = Config::from_env();
    let addr = config.listen_addr.clone();
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "starting Nako DLNA Renderer Addon");

    axum::serve(listener, nako_dlna_renderer::router(config)).await?;

    Ok(())
}
