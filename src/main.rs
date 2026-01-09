use axum::{
    http::Method,
    response::Html,
    routing::get,
    Router,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod services;
mod models;
mod utils;

use handlers::{health, upload};
use services::file_storage::FileStorageService;
use utils::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub storage: Arc<FileStorageService>,
}

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

    // Load configuration
    let config = AppConfig::from_env();
    tracing::info!("Configuration loaded: {:?}", config);

    // Initialize file storage service
    let storage_service = FileStorageService::new(&config.temp_dir)
        .map_err(|e| {
            tracing::error!("Failed to initialize file storage: {}", e);
            e
        })?;

    // Start cleanup task for temporary files
    let cleanup_storage = storage_service.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Run every hour
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_storage.cleanup_temp_files().await {
                tracing::error!("Failed to cleanup temporary files: {}", e);
            }
        }
    });

    // Create shared state
    let app_state = AppState {
        config: Arc::new(config.clone()),
        storage: Arc::new(storage_service),
    };

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    // Build the application router
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health::health_check))
        .route("/api/health", get(health::health_check))
        
        // File upload endpoint
        .route("/api/upload", axum::routing::post(upload::upload_image))
        
        // Fallback route for the frontend
        .fallback(fallback_handler)
        
        // Serve static files
        .nest_service("/static", ServeDir::new("static"))
        
        // Add shared state
        .with_state(app_state)
        
        // Add middleware layers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(config.request_timeout_seconds)))
                .layer(cors)
        );

    // Parse the bind address
    let addr: SocketAddr = config.bind_address().parse()?;
    tracing::info!("Server listening on {}", addr);

    // Create the server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

async fn fallback_handler() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Code Snippet Designer</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background: #f5f5f5;
        }
        .container {
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            text-align: center;
        }
        .status {
            text-align: center;
            color: #666;
            margin-top: 1rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎨 Code Snippet Designer</h1>
        <p class="status">Server is running! API endpoints are available.</p>
        <p class="status">Frontend interface coming soon...</p>
    </div>
</body>
</html>
    "#)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, starting graceful shutdown");
        },
    }
}