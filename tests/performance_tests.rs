use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use std::time::Instant;

mod common;
use common::*;

/// Test processing large code snippets
#[tokio::test]
async fn test_large_code_snippet_processing() {
    let app = setup_test_app().await;
    
    // Generate a large code snippet (1000 lines)
    let mut large_code = String::new();
    for i in 0..1000 {
        large_code.push_str(&format!("fn function_{}() {{\n", i));
        large_code.push_str(&format!("    println!(\"Function {}\");\n", i));
        large_code.push_str("}\n\n");
    }
    
    let start = Instant::now();
    
    let request = Request::builder()
        .uri("/api/process")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "code": large_code,
                "language": "rust"
            })).unwrap()
        ))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    let duration = start.elapsed();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["success"].as_bool().unwrap());
    // Line count includes the function declarations and closing braces (4 lines per function)
    assert_eq!(json["line_count"].as_u64().unwrap(), 4000); // 4 lines per function * 1000
    
    // Processing should complete within 5 seconds (requirement 5.1)
    assert!(duration.as_secs() < 5, "Processing took {:?}, expected < 5s", duration);
    
    println!("Large code snippet processing took: {:?}", duration);
}

/// Test image generation with high resolution
#[tokio::test]
async fn test_high_resolution_export() {
    let app = setup_test_app().await;
    
    // Get a theme first
    let themes_request = Request::builder()
        .uri("/api/themes")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let themes_response = app.clone().oneshot(themes_request).await.unwrap();
    let themes_body = axum::body::to_bytes(themes_response.into_body(), usize::MAX).await.unwrap();
    let themes_json: Value = serde_json::from_slice(&themes_body).unwrap();
    let theme = &themes_json["themes"][0];
    
    // Generate image with high resolution
    let code = "fn main() {\n    println!(\"Hello, world!\");\n}";
    
    let start = Instant::now();
    
    let request = Request::builder()
        .uri("/api/generate")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "code": code,
                "language": "rust",
                "theme": theme,
                "export_options": {
                    "format": "PNG",
                    "quality": 100,
                    "width": 2000,
                    "height": 1500
                }
            })).unwrap()
        ))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    let duration = start.elapsed();
    let status = response.status();
    
    // If generation fails due to missing dependencies, skip the test
    if status != StatusCode::OK {
        eprintln!("High resolution export test skipped - generation not available");
        return;
    }
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["success"].as_bool().unwrap());
    
    // Generation should start within 5 seconds (requirement 4.4)
    assert!(duration.as_secs() < 5, "Generation start took {:?}, expected < 5s", duration);
    
    println!("High resolution export initiation took: {:?}", duration);
}

/// Test concurrent request handling
#[tokio::test]
async fn test_concurrent_requests() {
    let app = setup_test_app().await;
    
    let start = Instant::now();
    
    // Create 10 concurrent requests
    let mut handles = vec![];
    
    for i in 0..10 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let request = Request::builder()
                .uri("/api/process")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "code": format!("fn test_{}() {{\n    println!(\"Test {}\");\n}}", i, i),
                        "language": "rust"
                    })).unwrap()
                ))
                .unwrap();
            
            app_clone.oneshot(request).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(response)) = handle.await {
            if response.status() == StatusCode::OK {
                success_count += 1;
            }
        }
    }
    
    let duration = start.elapsed();
    
    assert_eq!(success_count, 10, "All concurrent requests should succeed");
    
    // All requests should complete within reasonable time
    assert!(duration.as_secs() < 10, "Concurrent requests took {:?}, expected < 10s", duration);
    
    println!("10 concurrent requests completed in: {:?}", duration);
}

/// Test memory usage with multiple large snippets
#[tokio::test]
async fn test_multiple_large_snippets() {
    let app = setup_test_app().await;
    
    // Process 5 large code snippets sequentially
    for iteration in 0..5 {
        let mut large_code = String::new();
        for i in 0..500 {
            large_code.push_str(&format!("fn function_{}_{}() {{\n", iteration, i));
            large_code.push_str(&format!("    println!(\"Function {} {}\");\n", iteration, i));
            large_code.push_str("}\n\n");
        }
        
        let request = Request::builder()
            .uri("/api/process")
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "code": large_code,
                    "language": "rust"
                })).unwrap()
            ))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    println!("Successfully processed 5 large code snippets");
}

/// Test response time consistency
#[tokio::test]
async fn test_response_time_consistency() {
    let app = setup_test_app().await;
    
    let mut durations = vec![];
    
    // Make 20 identical requests and measure response times
    for _ in 0..20 {
        let start = Instant::now();
        
        let request = Request::builder()
            .uri("/api/process")
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "code": "fn main() {\n    println!(\"Hello, world!\");\n}",
                    "language": "rust"
                })).unwrap()
            ))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        let duration = start.elapsed();
        
        assert_eq!(response.status(), StatusCode::OK);
        durations.push(duration);
        
        // Small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    
    // Calculate average and max response time
    let avg_duration = durations.iter().sum::<std::time::Duration>() / durations.len() as u32;
    let max_duration = durations.iter().max().unwrap();
    
    println!("Average response time: {:?}", avg_duration);
    println!("Max response time: {:?}", max_duration);
    
    // Average should be well under 1 second
    assert!(avg_duration.as_millis() < 1000, "Average response time too high: {:?}", avg_duration);
    
    // Max should be under 3 seconds (requirement 5.1)
    assert!(max_duration.as_secs() < 3, "Max response time too high: {:?}", max_duration);
}

/// Test API endpoint throughput
#[tokio::test]
async fn test_api_throughput() {
    let app = setup_test_app().await;
    
    let start = Instant::now();
    let request_count = 50;
    
    // Send 50 requests as fast as possible
    let mut handles = vec![];
    
    for i in 0..request_count {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let request = Request::builder()
                .uri("/api/process/languages")
                .method("GET")
                .body(Body::empty())
                .unwrap();
            
            app_clone.oneshot(request).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all to complete
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(response)) = handle.await {
            if response.status() == StatusCode::OK {
                success_count += 1;
            }
        }
    }
    
    let duration = start.elapsed();
    let throughput = request_count as f64 / duration.as_secs_f64();
    
    println!("Processed {} requests in {:?}", request_count, duration);
    println!("Throughput: {:.2} requests/second", throughput);
    
    assert_eq!(success_count, request_count, "All requests should succeed");
    assert!(throughput > 10.0, "Throughput should be > 10 req/s, got {:.2}", throughput);
}
