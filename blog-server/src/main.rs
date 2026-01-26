mod server;
mod handlers;
mod domain;
mod application;
mod data;
mod infrastructure;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting blog server...");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Start server
    server::run().await?;

    Ok(())
}
