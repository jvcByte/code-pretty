use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Json, Response},
    body::Body,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::AppState;
use crate::services::download_service::{DownloadService, DownloadRequest, DownloadProgress};
use crate::services::export_service::{ExportService, EnhancedExportOptions};
use crate::models::theme::Theme;

/// Request to generate and download a code snippet image
#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub code: String,
    pub language: String,
    pub theme: Theme,
    #[serde(default)]
    pub export_options: EnhancedExportOptions,
}

/// Response for starting a download
#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub success: bool,
    pub download_id: String,
    pub message: String,
    pub progress_url: String,
    pub download_url: String,
}

/// Query parameters for checking download progress
#[derive(Debug, Deserialize)]
pub struct ProgressQuery {
    pub download_id: String,
}

/// Start image generation and return download ID
pub async fn generate_image(
    State(app_state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, (StatusCode, Json<Value>)> {
    // Validate the request
    if request.code.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid request",
                "message": "Code content cannot be empty"
            }))
        ));
    }

    if request.language.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid request", 
                "message": "Programming language must be specified"
            }))
        ));
    }

    // Create download service
    let export_service = std::sync::Arc::new(
        ExportService::new().map_err(|e| {
            tracing::error!("Failed to create export service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Service initialization failed",
                    "message": "Unable to initialize image generation service"
                }))
            )
        })?
    );

    let download_service = DownloadService::new(export_service, app_state.storage.clone());

    // Create download request
    let download_request = DownloadRequest {
        code: request.code,
        language: request.language,
        theme: request.theme,
        export_options: request.export_options,
    };

    // Start the download process
    let download_id = download_service.start_download(download_request).await.map_err(|e| {
        tracing::error!("Failed to start download: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Download failed to start",
                "message": e.to_string()
            }))
        )
    })?;

    // Store download service in app state for later use
    // Note: In a real application, you'd want to store this in the AppState
    // For now, we'll create it fresh for each request

    let response = GenerateResponse {
        success: true,
        download_id: download_id.clone(),
        message: "Image generation started".to_string(),
        progress_url: format!("/api/generate/progress/{}", download_id),
        download_url: format!("/api/generate/download/{}", download_id),
    };

    tracing::info!("Started image generation with download ID: {}", download_id);

    Ok(Json(response))
}

/// Check download progress
pub async fn check_progress(
    State(app_state): State<AppState>,
    Path(download_id): Path<String>,
) -> Result<Json<DownloadProgress>, (StatusCode, Json<Value>)> {
    // Create download service (in real app, this would be from AppState)
    let export_service = std::sync::Arc::new(
        ExportService::new().map_err(|e| {
            tracing::error!("Failed to create export service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Service initialization failed",
                    "message": "Unable to initialize service"
                }))
            )
        })?
    );

    let download_service = DownloadService::new(export_service, app_state.storage.clone());

    // Get progress
    let progress = download_service.get_progress(&download_id).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Download not found",
                "message": format!("No download found with ID: {}", download_id)
            }))
        )
    })?;

    Ok(Json(progress))
}

/// Download the generated file
pub async fn download_file(
    State(app_state): State<AppState>,
    Path(download_id): Path<String>,
) -> Result<Response<Body>, (StatusCode, Json<Value>)> {
    // Create download service (in real app, this would be from AppState)
    let export_service = std::sync::Arc::new(
        ExportService::new().map_err(|e| {
            tracing::error!("Failed to create export service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Service initialization failed",
                    "message": "Unable to initialize service"
                }))
            )
        })?
    );

    let download_service = DownloadService::new(export_service, app_state.storage.clone());

    // Get the file
    let (file_data, metadata) = download_service.get_download_file(&download_id).await.map_err(|e| {
        let status = match e.to_string().as_str() {
            s if s.contains("not found") => StatusCode::NOT_FOUND,
            s if s.contains("expired") => StatusCode::GONE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            Json(json!({
                "error": "Download failed",
                "message": e.to_string()
            }))
        )
    })?;

    // Create response with appropriate headers
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, metadata.content_type)
        .header(header::CONTENT_LENGTH, metadata.file_size.to_string())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", metadata.original_filename)
        )
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(Body::from(file_data))
        .map_err(|e| {
            tracing::error!("Failed to build response: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Response building failed",
                    "message": "Unable to create download response"
                }))
            )
        })?;

    tracing::info!("Serving download for ID: {} (size: {} bytes)", download_id, metadata.file_size);

    Ok(response)
}

/// Get available export formats and options
pub async fn get_export_options() -> Json<Value> {
    let formats = ExportService::supported_formats();
    let resolutions = ExportService::supported_resolutions();

    Json(json!({
        "formats": formats,
        "resolutions": resolutions,
        "quality_range": {
            "min": 1,
            "max": 100,
            "default": 90
        },
        "compression_levels": {
            "png": {
                "min": 0,
                "max": 9,
                "default": 6
            }
        },
        "dimension_limits": {
            "width": {
                "min": 100,
                "max": 8000
            },
            "height": {
                "min": 100,
                "max": 8000
            }
        }
    }))
}

/// Get download statistics (admin endpoint)
pub async fn get_download_stats(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Create download service
    let export_service = std::sync::Arc::new(
        ExportService::new().map_err(|e| {
            tracing::error!("Failed to create export service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Service initialization failed",
                    "message": "Unable to initialize service"
                }))
            )
        })?
    );

    let download_service = DownloadService::new(export_service, app_state.storage.clone());

    let stats = download_service.get_stats().await;

    Ok(Json(json!({
        "stats": stats,
        "timestamp": chrono::Utc::now().timestamp()
    })))
}