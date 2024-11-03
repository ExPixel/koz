use std::net::SocketAddr;

use anyhow::Context;
use axum::Router;
use koz_storage::Storage;

pub async fn run(config: WebConfig, _storage: Storage) -> anyhow::Result<()> {
    let app = Router::new().route("/", axum::routing::get(|| async { "Listening..." }));

    let address = config.address;
    tracing::info!("listening on {address}");
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .with_context(|| format!("error while binding to address: {address}"))?;

    axum::serve(listener, app)
        .await
        .context("error while serving")?;

    Ok(())
}

pub struct WebConfig {
    pub address: SocketAddr,
}
