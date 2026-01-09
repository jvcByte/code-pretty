use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::Json,
};
use futures_util::TryStreamExt;
use multer::Multipart;
use serde_json::{json, Value};
use crate::AppState;

/// Handle multipart file upload for images
pub async fn upload_image(
    State(app_state): State<AppState>,
    request: Request,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let boundary = request
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| multer::parse_boundary(ct).ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid content type",
                    "message": "Missing or invalid multipart boundary"
                }))
            )
        })?;

    // Convert the request body to a stream
    let stream = request.into_body().into_data_stream().map_err(|err| {
        std::io::Error::new(std::io::ErrorKind::Other, err)
    });

    let mut multipart = Multipart::new(stream, boundary);
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to read multipart field: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid multipart data",
                "message": "Failed to parse uploaded file"
            }))
        )
    })? {
        
        let name = field.name().map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string());
        let filename = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|s| s.to_string());

        tracing::debug!("Processing field: {} (filename: {:?}, content_type: {:?})", 
                       name, filename, content_type);

        // Only process file fields
        if name == "file" || name == "image" {
            let data = field.bytes().await.map_err(|e| {
                tracing::error!("Failed to read file data: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Failed to read file data",
                        "message": e.to_string()
                    }))
                )
            })?;

            // Validate file size
            if data.len() > app_state.config.max_file_size {
                return Err((
                    StatusCode::PAYLOAD_TOO_LARGE,
                    Json(json!({
                        "error": "File too large",
                        "message": format!("File size {} bytes exceeds maximum of {} bytes", 
                                         data.len(), app_state.config.max_file_size),
                        "max_size": app_state.config.max_file_size
                    }))
                ));
            }

            // Validate file format based on content type and magic bytes
            let (is_valid, extension) = validate_image_format(&data, content_type.as_deref())?;
            
            if !is_valid {
                return Err((
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    Json(json!({
                        "error": "Unsupported file format",
                        "message": "Only PNG, JPG, and JPEG images are supported",
                        "supported_formats": ["image/png", "image/jpeg", "image/jpg"]
                    }))
                ));
            }

            // Store the file
            let file_id = app_state.storage.store_temp_file(&data, &extension).await.map_err(|e| {
                tracing::error!("Failed to store uploaded file: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Storage failed",
                        "message": "Failed to save uploaded file"
                    }))
                )
            })?;

            uploaded_files.push(json!({
                "file_id": file_id,
                "filename": filename,
                "size": data.len(),
                "content_type": content_type,
                "extension": extension
            }));

            tracing::info!("Successfully uploaded file: {} (size: {} bytes, type: {:?})", 
                          file_id, data.len(), content_type);
        }
    }

    if uploaded_files.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "No files uploaded",
                "message": "Please select an image file to upload"
            }))
        ));
    }

    Ok(Json(json!({
        "success": true,
        "message": "Files uploaded successfully",
        "files": uploaded_files
    })))
}

/// Validate image format based on magic bytes and content type
fn validate_image_format(data: &[u8], content_type: Option<&str>) -> Result<(bool, String), (StatusCode, Json<Value>)> {
    if data.len() < 8 {
        return Ok((false, String::new()));
    }

    // Check magic bytes for common image formats
    let is_png = data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    let is_jpeg = data.starts_with(&[0xFF, 0xD8, 0xFF]);
    
    // Also check content type as secondary validation
    let content_type_valid = content_type.map_or(false, |ct| {
        ct == "image/png" || ct == "image/jpeg" || ct == "image/jpg"
    });

    if is_png {
        Ok((true, "png".to_string()))
    } else if is_jpeg {
        Ok((true, "jpg".to_string()))
    } else if content_type_valid {
        // Fallback to content type if magic bytes don't match but content type is valid
        let extension = match content_type.unwrap() {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpg",
            _ => return Ok((false, String::new())),
        };
        Ok((true, extension.to_string()))
    } else {
        Ok((false, String::new()))
    }
}