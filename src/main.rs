use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod services;
mod models;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "code_snippet_designer=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Code Snippet Designer server");

    // TODO: Initialize and start the web server
    tracing::info!("Server setup complete");

    Ok(())
}