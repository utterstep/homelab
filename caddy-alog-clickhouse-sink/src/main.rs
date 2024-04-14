use std::sync::Arc;

use eyre::{Result, WrapErr};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

mod app_state;
mod config;
mod handlers;
mod log;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    // enable tracing
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(4)
                .with_targets(true)
                .with_indent_lines(true)
                .with_bracketed_fields(true)
                .with_thread_names(false)
                .with_thread_ids(true),
        )
        .init();

    let app_state = Arc::new(app_state::AppState::new(config.clone()).await?);

    let listener = TcpListener::bind(config.bind_to())
        .await
        .wrap_err_with(|| format!("Failed to bind to address {}", config.bind_to()))?;

    loop {
        let (socket, peer) = listener.accept().await?;
        info!(peer_addr = %peer, "Accepted new connection");

        tokio::spawn(handlers::handle_stream(
            Arc::clone(&app_state),
            socket,
            peer,
        ));
    }
}
