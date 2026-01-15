use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("OCR processing failed: {message}")]
    OCRError { message: String },
    
    #[error("Image generation failed: {message}")]
    ImageGenerationError { message: String },
    
    #[error("File upload error: {message}")]
    FileUploadError { message: String },
    
    #[error("Theme not found: {theme_id}")]
    ThemeNotFound { theme_id: String },
    
    #[error("Theme error: {message}")]
    ThemeError { message: String },
    
    #[error("Language detection failed: {message}")]
    LanguageDetectionError { message: String },
    
    #[error("Syntax highlighting failed: {message}")]
    SyntaxHighlightingError { message: String },
    
    #[error("Validation error: {message}")]
    ValidationError { message: String },
    
    #[error("Storage error: {message}")]
    StorageError { message: String },
    
    #[error("Processing timeout: operation took longer than {timeout_seconds} seconds")]
    TimeoutError { timeout_seconds: u64 },
    
    #[error("Rate limit exceeded: {message}")]
    RateLimitError { message: String },
    
    #[error("Session error: {message}")]
    SessionError { message: String },
    
    #[error("Internal server error: {message}")]
    InternalError { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
    pub error_code: String,
    pub severity: ErrorSeverity,
    pub actions: Vec<ErrorAction>,
    pub retry_after: Option<u64>, // seconds
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAction {
    pub action_type: ErrorActionType,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorActionType {
    Retry,
    EditInput,
    ChangeSettings,
    ContactSupport,
    TryAlternative,
}

pub struct ErrorHandler;

impl ErrorHandler {
    /// Converts an AppError into a user-friendly ErrorResponse
    pub fn handle_error(error: AppError) -> ErrorResponse {
        match error {
            AppError::OCRError { message } => ErrorResponse {
                message: "Failed to extract text from image".to_string(),
                error_code: "OCR_FAILED".to_string(),
                severity: ErrorSeverity::Medium,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::Retry,
                        label: "Try Again".to_string(),
                        description: "Upload the image again".to_string(),
                    },
                    ErrorAction {
                        action_type: ErrorActionType::TryAlternative,
                        label: "Type Code".to_string(),
                        description: "Enter your code manually instead".to_string(),
                    },
                ],
                retry_after: Some(1),
                details: Some(message),
            },
            
            AppError::ImageGenerationError { message } => ErrorResponse {
                message: "Failed to generate code snippet image".to_string(),
                error_code: "IMAGE_GENERATION_FAILED".to_string(),
                severity: ErrorSeverity::High,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::Retry,
                        label: "Retry".to_string(),
                        description: "Try generating the image again".to_string(),
                    },
                    ErrorAction {
                        action_type: ErrorActionType::ChangeSettings,
                        label: "Change Theme".to_string(),
                        description: "Try a different theme or settings".to_string(),
                    },
                ],
                retry_after: Some(2),
                details: Some(message),
            },
            
            AppError::FileUploadError { message } => ErrorResponse {
                message: "File upload failed".to_string(),
                error_code: "UPLOAD_FAILED".to_string(),
                severity: ErrorSeverity::Medium,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::Retry,
                        label: "Try Again".to_string(),
                        description: "Upload the file again".to_string(),
                    },
                    ErrorAction {
                        action_type: ErrorActionType::EditInput,
                        label: "Check File".to_string(),
                        description: "Ensure the file is a valid image (PNG, JPG, JPEG)".to_string(),
                    },
                ],
                retry_after: Some(1),
                details: Some(message),
            },
            
            AppError::ThemeNotFound { theme_id } => ErrorResponse {
                message: "Theme not found".to_string(),
                error_code: "THEME_NOT_FOUND".to_string(),
                severity: ErrorSeverity::Low,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::ChangeSettings,
                        label: "Choose Different Theme".to_string(),
                        description: "Select from available themes".to_string(),
                    },
                ],
                retry_after: None,
                details: Some(format!("Theme '{}' does not exist", theme_id)),
            },
            
            AppError::ThemeError { message } => ErrorResponse {
                message: "Theme configuration error".to_string(),
                error_code: "THEME_ERROR".to_string(),
                severity: ErrorSeverity::Medium,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::EditInput,
                        label: "Fix Theme".to_string(),
                        description: "Correct the theme configuration".to_string(),
                    },
                    ErrorAction {
                        action_type: ErrorActionType::ChangeSettings,
                        label: "Use Default Theme".to_string(),
                        description: "Switch to a default theme".to_string(),
                    },
                ],
                retry_after: None,
                details: Some(message),
            },
            
            AppError::LanguageDetectionError { message } => ErrorResponse {
                message: "Could not detect programming language".to_string(),
                error_code: "LANGUAGE_DETECTION_FAILED".to_string(),
                severity: ErrorSeverity::Low,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::EditInput,
                        label: "Select Language".to_string(),
                        description: "Manually choose the programming language".to_string(),
                    },
                ],
                retry_after: None,
                details: Some(message),
            },
            
            AppError::SyntaxHighlightingError { message } => ErrorResponse {
                message: "Syntax highlighting failed".to_string(),
                error_code: "SYNTAX_HIGHLIGHTING_FAILED".to_string(),
                severity: ErrorSeverity::Medium,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::Retry,
                        label: "Try Again".to_string(),
                        description: "Retry with syntax highlighting".to_string(),
                    },
                    ErrorAction {
                        action_type: ErrorActionType::TryAlternative,
                        label: "Use Plain Text".to_string(),
                        description: "Continue without syntax highlighting".to_string(),
                    },
                ],
                retry_after: Some(1),
                details: Some(message),
            },
            
            AppError::ValidationError { message } => ErrorResponse {
                message: "Input validation failed".to_string(),
                error_code: "VALIDATION_FAILED".to_string(),
                severity: ErrorSeverity::Medium,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::EditInput,
                        label: "Fix Input".to_string(),
                        description: "Please correct the input and try again".to_string(),
                    },
                ],
                retry_after: None,
                details: Some(message),
            },
            
            AppError::StorageError { message } => ErrorResponse {
                message: "Storage operation failed".to_string(),
                error_code: "STORAGE_FAILED".to_string(),
                severity: ErrorSeverity::High,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::Retry,
                        label: "Try Again".to_string(),
                        description: "Retry the operation".to_string(),
                    },
                    ErrorAction {
                        action_type: ErrorActionType::ContactSupport,
                        label: "Contact Support".to_string(),
                        description: "If the problem persists, please contact support".to_string(),
                    },
                ],
                retry_after: Some(5),
                details: Some(message),
            },
            
            AppError::TimeoutError { timeout_seconds } => ErrorResponse {
                message: "Operation timed out".to_string(),
                error_code: "TIMEOUT".to_string(),
                severity: ErrorSeverity::Medium,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::Retry,
                        label: "Try Again".to_string(),
                        description: "The operation may succeed if retried".to_string(),
                    },
                ],
                retry_after: Some(3),
                details: Some(format!("Operation exceeded {} second timeout", timeout_seconds)),
            },
            
            AppError::RateLimitError { message } => ErrorResponse {
                message: "Too many requests".to_string(),
                error_code: "RATE_LIMITED".to_string(),
                severity: ErrorSeverity::Medium,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::Retry,
                        label: "Wait and Retry".to_string(),
                        description: "Please wait a moment before trying again".to_string(),
                    },
                ],
                retry_after: Some(60),
                details: Some(message),
            },
            
            AppError::SessionError { message } => ErrorResponse {
                message: "Session error".to_string(),
                error_code: "SESSION_ERROR".to_string(),
                severity: ErrorSeverity::Medium,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::Retry,
                        label: "Refresh".to_string(),
                        description: "Refresh the page to start a new session".to_string(),
                    },
                ],
                retry_after: None,
                details: Some(message),
            },
            
            AppError::InternalError { message } => ErrorResponse {
                message: "An unexpected error occurred".to_string(),
                error_code: "INTERNAL_ERROR".to_string(),
                severity: ErrorSeverity::High,
                actions: vec![
                    ErrorAction {
                        action_type: ErrorActionType::Retry,
                        label: "Try Again".to_string(),
                        description: "The error might be temporary".to_string(),
                    },
                    ErrorAction {
                        action_type: ErrorActionType::ContactSupport,
                        label: "Report Issue".to_string(),
                        description: "Report this issue if it continues".to_string(),
                    },
                ],
                retry_after: Some(5),
                details: Some(message),
            },
        }
    }

    /// Retry mechanism with exponential backoff for transient failures
    pub async fn retry_with_backoff<F, T, E>(
        operation: F,
        max_attempts: usize,
        initial_delay: Duration,
    ) -> Result<T, E>
    where
        F: Fn() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        let mut delay = initial_delay;
        
        for attempt in 1..=max_attempts {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt == max_attempts {
                        return Err(error);
                    }
                    
                    tracing::warn!(
                        "Operation failed on attempt {}/{}, retrying in {:?}: {:?}",
                        attempt,
                        max_attempts,
                        delay,
                        error
                    );
                    
                    sleep(delay).await;
                    delay = std::cmp::min(delay * 2, Duration::from_secs(30)); // Cap at 30 seconds
                }
            }
        }
        
        unreachable!("Loop should have returned or errored")
    }

    /// Async version of retry with backoff for async operations
    pub async fn retry_async_with_backoff<F, Fut, T, E>(
        operation: F,
        max_attempts: usize,
        initial_delay: Duration,
    ) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut delay = initial_delay;
        
        for attempt in 1..=max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt == max_attempts {
                        return Err(error);
                    }
                    
                    tracing::warn!(
                        "Async operation failed on attempt {}/{}, retrying in {:?}: {:?}",
                        attempt,
                        max_attempts,
                        delay,
                        error
                    );
                    
                    sleep(delay).await;
                    delay = std::cmp::min(delay * 2, Duration::from_secs(30)); // Cap at 30 seconds
                }
            }
        }
        
        unreachable!("Loop should have returned or errored")
    }

    /// Determines if an error is retryable
    pub fn is_retryable(error: &AppError) -> bool {
        matches!(
            error,
            AppError::OCRError { .. } |
            AppError::ImageGenerationError { .. } |
            AppError::FileUploadError { .. } |
            AppError::StorageError { .. } |
            AppError::TimeoutError { .. } |
            AppError::SyntaxHighlightingError { .. } |
            AppError::InternalError { .. }
        )
    }

    /// Gets the recommended retry delay for an error
    pub fn get_retry_delay(error: &AppError) -> Duration {
        match error {
            AppError::OCRError { .. } => Duration::from_secs(1),
            AppError::ImageGenerationError { .. } => Duration::from_secs(2),
            AppError::FileUploadError { .. } => Duration::from_secs(1),
            AppError::StorageError { .. } => Duration::from_secs(5),
            AppError::TimeoutError { .. } => Duration::from_secs(3),
            AppError::RateLimitError { .. } => Duration::from_secs(60),
            AppError::SyntaxHighlightingError { .. } => Duration::from_secs(1),
            AppError::InternalError { .. } => Duration::from_secs(5),
            _ => Duration::from_secs(1),
        }
    }
}

// Convenience functions for creating specific errors
impl AppError {
    pub fn ocr_failed(message: impl Into<String>) -> Self {
        AppError::OCRError { message: message.into() }
    }

    pub fn image_generation_failed(message: impl Into<String>) -> Self {
        AppError::ImageGenerationError { message: message.into() }
    }

    pub fn file_upload_failed(message: impl Into<String>) -> Self {
        AppError::FileUploadError { message: message.into() }
    }

    pub fn theme_not_found(theme_id: impl Into<String>) -> Self {
        AppError::ThemeNotFound { theme_id: theme_id.into() }
    }

    pub fn theme_error(message: impl Into<String>) -> Self {
        AppError::ThemeError { message: message.into() }
    }

    pub fn language_detection_failed(message: impl Into<String>) -> Self {
        AppError::LanguageDetectionError { message: message.into() }
    }

    pub fn syntax_highlighting_failed(message: impl Into<String>) -> Self {
        AppError::SyntaxHighlightingError { message: message.into() }
    }

    pub fn validation_failed(message: impl Into<String>) -> Self {
        AppError::ValidationError { message: message.into() }
    }

    pub fn storage_failed(message: impl Into<String>) -> Self {
        AppError::StorageError { message: message.into() }
    }

    pub fn timeout(timeout_seconds: u64) -> Self {
        AppError::TimeoutError { timeout_seconds }
    }

    pub fn rate_limited(message: impl Into<String>) -> Self {
        AppError::RateLimitError { message: message.into() }
    }

    pub fn session_error(message: impl Into<String>) -> Self {
        AppError::SessionError { message: message.into() }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        AppError::InternalError { message: message.into() }
    }
}