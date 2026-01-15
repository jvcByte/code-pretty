use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::services::theme_manager::{ThemeManager, ThemeCustomization};
use crate::models::theme::Theme;

/// Get all available themes
pub async fn list_themes(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let theme_manager = ThemeManager::new();
    let themes = theme_manager.list_themes().await;

    Ok(Json(json!({
        "success": true,
        "themes": themes,
        "total": themes.len()
    })))
}

/// Get a specific theme by ID
pub async fn get_theme(
    State(_app_state): State<AppState>,
    Path(theme_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let theme_manager = ThemeManager::new();
    
    match theme_manager.get_theme(&theme_id).await {
        Some(theme) => Ok(Json(json!({
            "success": true,
            "theme": theme
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Theme not found",
                "message": format!("No theme found with ID: {}", theme_id)
            }))
        ))
    }
}

/// Get theme information (IDs and names only)
pub async fn list_theme_info(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let theme_manager = ThemeManager::new();
    let theme_info = theme_manager.list_theme_info().await;

    let themes: Vec<_> = theme_info
        .into_iter()
        .map(|(id, name)| json!({ "id": id, "name": name }))
        .collect();

    Ok(Json(json!({
        "success": true,
        "themes": themes,
        "total": themes.len()
    })))
}

/// Get themes by type (dark, light, high-contrast)
#[derive(Debug, Deserialize)]
pub struct ThemeTypeQuery {
    #[serde(rename = "type")]
    pub theme_type: String,
}

pub async fn get_themes_by_type(
    State(_app_state): State<AppState>,
    Path(theme_type): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let theme_manager = ThemeManager::new();
    
    let theme_type_enum = match theme_type.to_lowercase().as_str() {
        "dark" => crate::services::theme_manager::ThemeType::Dark,
        "light" => crate::services::theme_manager::ThemeType::Light,
        "high-contrast" | "highcontrast" => crate::services::theme_manager::ThemeType::HighContrast,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid theme type",
                    "message": "Theme type must be 'dark', 'light', or 'high-contrast'",
                    "valid_types": ["dark", "light", "high-contrast"]
                }))
            ));
        }
    };

    let themes = theme_manager.get_themes_by_type(theme_type_enum).await;

    Ok(Json(json!({
        "success": true,
        "type": theme_type,
        "themes": themes,
        "total": themes.len()
    })))
}

/// Create a customized theme
#[derive(Debug, Deserialize)]
pub struct CustomizeThemeRequest {
    pub base_theme_id: String,
    pub customization: ThemeCustomization,
}

pub async fn customize_theme(
    State(_app_state): State<AppState>,
    Json(request): Json<CustomizeThemeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let theme_manager = ThemeManager::new();

    match theme_manager.customize_theme(&request.base_theme_id, request.customization).await {
        Ok(customized_theme) => Ok(Json(json!({
            "success": true,
            "theme": customized_theme,
            "message": "Theme customized successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Theme customization failed",
                "message": e.to_string()
            }))
        ))
    }
}

/// Validate a theme configuration
pub async fn validate_theme(
    State(_app_state): State<AppState>,
    Json(theme): Json<Theme>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let theme_manager = ThemeManager::new();

    match theme_manager.validate_theme(&theme) {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "is_valid": true,
            "message": "Theme configuration is valid"
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "is_valid": false,
            "message": e.to_string(),
            "errors": [e.to_string()]
        })))
    }
}

/// Get default theme
pub async fn get_default_theme(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let theme_manager = ThemeManager::new();
    let default_theme = theme_manager.get_default_theme().await;

    Ok(Json(json!({
        "success": true,
        "theme": default_theme
    })))
}

/// Get available customization options
pub async fn get_customization_options(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let theme_manager = ThemeManager::new();

    let fonts = theme_manager.get_available_fonts();
    let window_styles = theme_manager.get_available_window_styles();
    let background_types = theme_manager.get_available_background_types();

    Ok(Json(json!({
        "success": true,
        "options": {
            "fonts": fonts,
            "window_styles": window_styles,
            "background_types": background_types,
            "color_format": "Hex color codes (#RRGGBB or #RGB)",
            "opacity_range": {
                "min": 0.0,
                "max": 1.0
            },
            "font_size_range": {
                "min": 8.0,
                "max": 32.0,
                "default": 14.0
            },
            "line_height_range": {
                "min": 1.0,
                "max": 3.0,
                "default": 1.5
            }
        }
    })))
}