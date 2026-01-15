use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::AppState;
use crate::services::language_detector::{LanguageDetector, LanguageResult};

/// Request to process text input
#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    /// The code content to process
    pub code: String,
    /// Optional language hint (if not provided, will be auto-detected)
    pub language: Option<String>,
}

/// Response from text processing
#[derive(Debug, Serialize)]
pub struct ProcessResponse {
    pub success: bool,
    pub code: String,
    pub language: LanguageResult,
    pub formatted: bool,
    pub line_count: usize,
    pub character_count: usize,
}

/// Process text input with syntax highlighting and language detection
pub async fn process_text(
    State(_app_state): State<AppState>,
    Json(request): Json<ProcessRequest>,
) -> Result<Json<ProcessResponse>, (StatusCode, Json<Value>)> {
    // Validate input
    if request.code.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid request",
                "message": "Code content cannot be empty"
            }))
        ));
    }

    // Create language detector
    let language_detector = LanguageDetector::new().map_err(|e| {
        tracing::error!("Failed to create language detector: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Service initialization failed",
                "message": "Unable to initialize language detection service"
            }))
        )
    })?;

    // Detect or validate language
    let language_result = if let Some(lang) = request.language {
        // Manual language selection - validate it
        language_detector.validate_manual_selection(&lang).unwrap_or_else(|_| {
            // If validation fails, create manual override
            language_detector.create_manual_override(&lang)
        })
    } else {
        // Auto-detect language
        language_detector.detect_language(&request.code)
    };

    // Calculate statistics
    let line_count = request.code.lines().count();
    let character_count = request.code.chars().count();

    // Format the code (preserve original formatting for now)
    let formatted_code = request.code.clone();

    tracing::info!(
        "Processed text input: {} lines, {} characters, detected language: {}",
        line_count,
        character_count,
        language_result.language
    );

    Ok(Json(ProcessResponse {
        success: true,
        code: formatted_code,
        language: language_result,
        formatted: true,
        line_count,
        character_count,
    }))
}

/// Validate code syntax (basic validation)
#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub code: String,
    pub language: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub is_valid: bool,
    pub language_supported: bool,
    pub suggestions: Vec<String>,
}

/// Validate code and language
pub async fn validate_code(
    State(_app_state): State<AppState>,
    Json(request): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, (StatusCode, Json<Value>)> {
    // Create language detector
    let language_detector = LanguageDetector::new().map_err(|e| {
        tracing::error!("Failed to create language detector: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Service initialization failed",
                "message": "Unable to initialize language detection service"
            }))
        )
    })?;

    // Check if language is supported
    let supported_languages = language_detector.get_supported_languages();
    let language_supported = supported_languages
        .iter()
        .any(|lang| lang.to_lowercase() == request.language.to_lowercase());

    let mut suggestions = Vec::new();

    if !language_supported {
        // Find similar languages
        suggestions = supported_languages
            .iter()
            .filter(|lang| {
                let lang_lower = lang.to_lowercase();
                let req_lower = request.language.to_lowercase();
                lang_lower.contains(&req_lower) || req_lower.contains(&lang_lower)
            })
            .take(3)
            .cloned()
            .collect();
    }

    // Basic validation - check if code is not empty
    let is_valid = !request.code.trim().is_empty() && language_supported;

    Ok(Json(ValidateResponse {
        is_valid,
        language_supported,
        suggestions,
    }))
}

/// Get supported languages
pub async fn get_supported_languages(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let language_detector = LanguageDetector::new().map_err(|e| {
        tracing::error!("Failed to create language detector: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Service initialization failed",
                "message": "Unable to initialize language detection service"
            }))
        )
    })?;

    let languages = language_detector.get_supported_languages();
    let extensions = language_detector.get_supported_extensions();

    Ok(Json(json!({
        "languages": languages,
        "extensions": extensions,
        "total_languages": languages.len(),
        "total_extensions": extensions.len()
    })))
}