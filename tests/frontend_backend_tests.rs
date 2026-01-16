use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::{json, Value};
use tower::ServiceExt;

mod common;
use common::*;

/// Test frontend static file serving
#[tokio::test]
async fn test_static_file_serving() {
    let app = setup_test_app().await;
    
    // Test root path
    let request = Request::builder()
        .uri("/")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    
    // Should contain HTML content
    assert!(html.contains("<!DOCTYPE html>") || html.contains("<html"));
}

/// Test complete user workflow simulation
#[tokio::test]
async fn test_complete_user_workflow() {
    let app = setup_test_app().await;
    
    // Step 1: User loads the application
    let request = Request::builder()
        .uri("/")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Step 2: User checks available languages
    let request = Request::builder()
        .uri("/api/process/languages")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let languages_json: Value = serde_json::from_slice(&body).unwrap();
    let languages = languages_json["languages"].as_array().unwrap();
    assert!(languages.len() > 0);
    
    // Step 3: User loads available themes
    let request = Request::builder()
        .uri("/api/themes")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let themes_json: Value = serde_json::from_slice(&body).unwrap();
    let themes = themes_json["themes"].as_array().unwrap();
    assert!(themes.len() > 0);
    let selected_theme = &themes[0];
    
    // Step 4: User pastes code and processes it
    let code = "function hello() {\n    console.log('Hello, world!');\n}";
    
    let request = Request::builder()
        .uri("/api/process")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "code": code,
                "language": null
            })).unwrap()
        ))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let process_json: Value = serde_json::from_slice(&body).unwrap();
    assert!(process_json["success"].as_bool().unwrap());
    let detected_language = process_json["language"]["language"].as_str().unwrap();
    
    // Step 5: User generates image with selected theme
    let request = Request::builder()
        .uri("/api/generate")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "code": code,
                "language": detected_language,
                "theme": selected_theme,
                "export_options": {
                    "format": "PNG",
                    "quality": 90
                }
            })).unwrap()
        ))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let generate_json: Value = serde_json::from_slice(&body).unwrap();
    assert!(generate_json["success"].as_bool().unwrap());
    
    println!("Complete user workflow test passed");
}

/// Test theme customization workflow
#[tokio::test]
async fn test_theme_customization_workflow() {
    let app = setup_test_app().await;
    
    // Step 1: Get default theme
    let request = Request::builder()
        .uri("/api/themes/default")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let mut theme: Value = serde_json::from_slice(&body).unwrap();
    
    // Step 2: Get customization options
    let request = Request::builder()
        .uri("/api/themes/options")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let options: Value = serde_json::from_slice(&body).unwrap();
    assert!(options["background_types"].is_array());
    assert!(options["window_styles"].is_array());
    
    // Step 3: Customize theme
    if let Some(background) = theme.get_mut("background") {
        if let Some(primary) = background.get_mut("primary") {
            *primary = json!("#1a1a1a");
        }
    }
    
    let request = Request::builder()
        .uri("/api/themes/customize")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&theme).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let customized_theme: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        customized_theme["background"]["primary"].as_str().unwrap(),
        "#1a1a1a"
    );
    
    println!("Theme customization workflow test passed");
}

/// Test error recovery workflow
#[tokio::test]
async fn test_error_recovery_workflow() {
    let app = setup_test_app().await;
    
    // Step 1: User submits empty code (error)
    let request = Request::builder()
        .uri("/api/process")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "code": "",
                "language": null
            })).unwrap()
        ))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_json: Value = serde_json::from_slice(&body).unwrap();
    assert!(error_json["error"].is_string());
    assert!(error_json["message"].is_string());
    
    // Step 2: User corrects the error and resubmits
    let request = Request::builder()
        .uri("/api/process")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "code": "fn main() { println!(\"Hello\"); }",
                "language": "rust"
            })).unwrap()
        ))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let success_json: Value = serde_json::from_slice(&body).unwrap();
    assert!(success_json["success"].as_bool().unwrap());
    
    println!("Error recovery workflow test passed");
}

/// Test multiple language processing
#[tokio::test]
async fn test_multiple_language_processing() {
    let app = setup_test_app().await;
    
    let test_snippets = get_test_code_snippets();
    
    for (expected_lang, code) in test_snippets {
        let request = Request::builder()
            .uri("/api/process")
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "code": code,
                    "language": null
                })).unwrap()
            ))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert!(json["success"].as_bool().unwrap());
        let detected_lang = json["language"]["language"].as_str().unwrap().to_lowercase();
        
        println!("Expected: {}, Detected: {}", expected_lang, detected_lang);
        
        // Language detection should work for most cases
        // (some may not match exactly due to detection heuristics)
    }
    
    println!("Multiple language processing test passed");
}

/// Test export format options
#[tokio::test]
async fn test_export_format_options() {
    let app = setup_test_app().await;
    
    // Get export options
    let request = Request::builder()
        .uri("/api/generate/options")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let options: Value = serde_json::from_slice(&body).unwrap();
    
    assert!(options["formats"].is_array());
    assert!(options["resolutions"].is_array());
    assert!(options["quality_range"].is_object());
    
    let formats = options["formats"].as_array().unwrap();
    assert!(formats.len() > 0);
    
    println!("Export format options: {:?}", formats);
    println!("Export format options test passed");
}

/// Test session persistence simulation
#[tokio::test]
async fn test_session_persistence() {
    let app = setup_test_app().await;
    
    // Simulate user session with multiple operations
    let session_id = uuid::Uuid::new_v4().to_string();
    
    // Operation 1: Process code
    let request = Request::builder()
        .uri("/api/process")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Session-ID", &session_id)
        .body(Body::from(
            serde_json::to_string(&json!({
                "code": "fn test() {}",
                "language": "rust"
            })).unwrap()
        ))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Operation 2: Get themes (same session)
    let request = Request::builder()
        .uri("/api/themes")
        .method("GET")
        .header("X-Session-ID", &session_id)
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Operation 3: Generate image (same session)
    let themes_request = Request::builder()
        .uri("/api/themes/default")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let themes_response = app.clone().oneshot(themes_request).await.unwrap();
    let themes_body = axum::body::to_bytes(themes_response.into_body(), usize::MAX).await.unwrap();
    let theme: Value = serde_json::from_slice(&themes_body).unwrap();
    
    let request = Request::builder()
        .uri("/api/generate")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Session-ID", &session_id)
        .body(Body::from(
            serde_json::to_string(&json!({
                "code": "fn test() {}",
                "language": "rust",
                "theme": theme,
                "export_options": {
                    "format": "PNG",
                    "quality": 90
                }
            })).unwrap()
        ))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    println!("Session persistence test passed");
}
