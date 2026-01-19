use axum::{
    http::Method,
    response::{Html, IntoResponse},
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

use code_snippet_designer::{handlers, services, utils, AppState};

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
    let config = utils::config::AppConfig::from_env();
    tracing::info!("Configuration loaded: {:?}", config);

    // Initialize file storage service
    let storage_service = services::file_storage::FileStorageService::new(&config.temp_dir)
        .map_err(|e| {
            tracing::error!("Failed to initialize file storage: {}", e);
            e
        })?;

    // Initialize session manager (1 hour expiry)
    let session_manager = services::session_manager::SessionManager::with_expiry(
        std::time::Duration::from_secs(3600),
    );

    // Initialize cache manager (30 minute TTL, max 1000 items)
    let cache_manager = services::cache_manager::CacheManager::with_ttl_and_max_size(
        std::time::Duration::from_secs(1800),
        1000,
    );

    // Initialize rate limiter (100 requests per minute)
    let rate_limiter =
        services::rate_limiter::RateLimiter::with_config(services::rate_limiter::RateLimitConfig {
            max_requests: 100,
            window_duration: std::time::Duration::from_secs(60),
        });

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

    // Start cleanup task for expired sessions
    let cleanup_session_manager = session_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600)); // Run every 10 minutes
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_session_manager.cleanup_expired_sessions().await {
                tracing::error!("Failed to cleanup expired sessions: {}", e);
            }
        }
    });

    // Start cleanup task for expired cache items
    let cleanup_cache_manager = cache_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Run every 5 minutes
        loop {
            interval.tick().await;
            cleanup_cache_manager.cleanup_expired().await;
        }
    });

    // Start cleanup task for expired rate limit records
    let cleanup_rate_limiter = rate_limiter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Run every 5 minutes
        loop {
            interval.tick().await;
            cleanup_rate_limiter.cleanup_expired().await;
        }
    });

    // Start cleanup task for expired downloads
    // (moved) — the cleanup will run using the shared `DownloadService` created below

    // Create shared state, including a shared ExportService and DownloadService
    // Initialize ExportService once and create a DownloadService that will be reused by handlers.
    let export_service = std::sync::Arc::new(services::export_service::ExportService::new()?);

    // The download service requires an Arc<FileStorageService>. We clone the concrete storage service
    // and wrap it in an Arc for the DownloadService constructor.
    let download_service = std::sync::Arc::new(services::download_service::DownloadService::new(
        Arc::clone(&export_service),
        Arc::new(storage_service.clone()),
    ));

    let app_state = AppState {
        config: Arc::new(config.clone()),
        storage: Arc::new(storage_service),
        session_manager: Arc::new(session_manager),
        cache_manager: Arc::new(cache_manager),
        rate_limiter: Arc::new(rate_limiter),
        download_service: Arc::clone(&download_service),
    };

    // Spawn the cleanup task for expired downloads using the shared DownloadService.
    {
        let cleanup_download_service = download_service.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1800)); // Run every 30 minutes
            loop {
                interval.tick().await;
                if let Err(e) = cleanup_download_service.cleanup_expired_downloads().await {
                    tracing::error!("Failed to cleanup expired downloads: {}", e);
                }
            }
        });
    }

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    // Build the application router
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(handlers::health::health_check))
        .route("/api/health", get(handlers::health::health_check))
        // File upload endpoint
        .route(
            "/api/upload",
            axum::routing::post(
                |state: axum::extract::State<AppState>,
                 req: axum::http::Request<axum::body::Body>| async move {
                    handlers::upload::upload_image(state, req).await
                },
            ),
        )
        // Text processing endpoints
        .route(
            "/api/process",
            axum::routing::post(handlers::process::process_text),
        )
        .route(
            "/api/process/validate",
            axum::routing::post(handlers::process::validate_code),
        )
        .route(
            "/api/process/languages",
            get(handlers::process::get_supported_languages),
        )
        // Theme endpoints
        .route("/api/themes", get(handlers::themes::list_themes))
        .route("/api/themes/info", get(handlers::themes::list_theme_info))
        .route(
            "/api/themes/default",
            get(handlers::themes::get_default_theme),
        )
        .route(
            "/api/themes/options",
            get(handlers::themes::get_customization_options),
        )
        .route("/api/themes/:theme_id", get(handlers::themes::get_theme))
        .route(
            "/api/themes/type/:theme_type",
            get(handlers::themes::get_themes_by_type),
        )
        .route(
            "/api/themes/customize",
            axum::routing::post(handlers::themes::customize_theme),
        )
        .route(
            "/api/themes/validate",
            axum::routing::post(handlers::themes::validate_theme),
        )
        // Image generation and download endpoints
        .route(
            "/api/generate",
            axum::routing::post(handlers::generate::generate_image),
        )
        .route(
            "/api/generate/progress/:download_id",
            get(handlers::generate::check_progress),
        )
        .route(
            "/api/generate/download/:download_id",
            get(handlers::generate::download_file),
        )
        .route(
            "/api/generate/options",
            get(handlers::generate::get_export_options),
        )
        .route(
            "/api/generate/stats",
            get(handlers::generate::get_download_stats),
        )
        // Serve static files
        .nest_service("/static", ServeDir::new("static"))
        // Serve the main frontend
        .route("/", get(serve_frontend))
        // Fallback route for SPA routing
        .fallback(serve_frontend)
        // Add shared state
        .with_state(app_state)
        // Add middleware layers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(
                    config.request_timeout_seconds,
                )))
                .layer(cors),
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

async fn serve_frontend(
    req: axum::http::Request<axum::body::Body>,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    // If the incoming request is for an API path, return a JSON 404 instead of serving the SPA HTML.
    // This prevents accidental HTML responses being returned to API clients.
    let path = req.uri().path();
    if path.starts_with("/api/") {
        let body = r#"{"error":"Not Found","message":"API endpoint not found"}"#;
        let resp = axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_FOUND)
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(body))
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(resp);
    }

    // Otherwise serve the SPA frontend (either the static index.html or a fallback page).
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(content) => Ok(Html(content).into_response()),
        Err(_) => {
            // Fallback HTML if file doesn't exist
            let fallback_html = r#"
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
            "#;
            Ok(Html(fallback_html).into_response())
        }
    }
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
