use axum::Router;
use std::sync::Arc;
use tempfile::TempDir;

// Re-export the main app modules for testing
use code_snippet_designer::{AppState, handlers, services, utils};

/// Setup a test application with temporary storage
pub async fn setup_test_app() -> Router {
    // Create temporary directory for test storage
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap().to_string();
    
    // Create test configuration
    let config = utils::config::AppConfig {
        host: "127.0.0.1".to_string(),
        port: 0, // Use random port for testing
        temp_dir: temp_path.clone(),
        max_file_size: 10 * 1024 * 1024, // 10MB
        cors_origins: vec!["*".to_string()],
        request_timeout_seconds: 30,
    };
    
    // Initialize services
    let storage_service = services::file_storage::FileStorageService::new(&temp_path)
        .expect("Failed to create storage service");
    
    let session_manager = services::session_manager::SessionManager::with_expiry(
        std::time::Duration::from_secs(3600)
    );
    
    let cache_manager = services::cache_manager::CacheManager::with_ttl_and_max_size(
        std::time::Duration::from_secs(1800),
        1000
    );
    
    let rate_limiter = services::rate_limiter::RateLimiter::with_config(
        services::rate_limiter::RateLimitConfig {
            max_requests: 100,
            window_duration: std::time::Duration::from_secs(60),
        }
    );
    
    // Create app state
    let app_state = AppState {
        config: Arc::new(config),
        storage: Arc::new(storage_service),
        session_manager: Arc::new(session_manager),
        cache_manager: Arc::new(cache_manager),
        rate_limiter: Arc::new(rate_limiter),
    };
    
    // Build router (simplified version without middleware for testing)
    Router::new()
        .route("/health", axum::routing::get(handlers::health::health_check))
        .route("/api/health", axum::routing::get(handlers::health::health_check))
        .route("/api/upload", axum::routing::post(handlers::upload::upload_image))
        .route("/api/process", axum::routing::post(handlers::process::process_text))
        .route("/api/process/validate", axum::routing::post(handlers::process::validate_code))
        .route("/api/process/languages", axum::routing::get(handlers::process::get_supported_languages))
        .route("/api/themes", axum::routing::get(handlers::themes::list_themes))
        .route("/api/themes/info", axum::routing::get(handlers::themes::list_theme_info))
        .route("/api/themes/default", axum::routing::get(handlers::themes::get_default_theme))
        .route("/api/themes/options", axum::routing::get(handlers::themes::get_customization_options))
        .route("/api/themes/:theme_id", axum::routing::get(handlers::themes::get_theme))
        .route("/api/themes/type/:theme_type", axum::routing::get(handlers::themes::get_themes_by_type))
        .route("/api/themes/customize", axum::routing::post(handlers::themes::customize_theme))
        .route("/api/themes/validate", axum::routing::post(handlers::themes::validate_theme))
        .route("/api/generate", axum::routing::post(handlers::generate::generate_image))
        .route("/api/generate/progress/:download_id", axum::routing::get(handlers::generate::check_progress))
        .route("/api/generate/download/:download_id", axum::routing::get(handlers::generate::download_file))
        .route("/api/generate/options", axum::routing::get(handlers::generate::get_export_options))
        .route("/api/generate/stats", axum::routing::get(handlers::generate::get_download_stats))
        .with_state(app_state)
}

/// Create a test PNG image (1x1 pixel)
pub fn create_test_image() -> Vec<u8> {
    // PNG magic bytes + minimal valid PNG structure
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // Width: 1
        0x00, 0x00, 0x00, 0x01, // Height: 1
        0x08, 0x02, 0x00, 0x00, 0x00, // Bit depth, color type, etc.
        0x90, 0x77, 0x53, 0xDE, // CRC
        0x00, 0x00, 0x00, 0x0C, // IDAT chunk length
        0x49, 0x44, 0x41, 0x54, // IDAT
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, // Compressed data
        0x03, 0x01, 0x01, 0x00,
        0x18, 0xDD, 0x8D, 0xB4, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk length
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ]
}

/// Create a multipart form body for file upload
pub fn create_multipart_body(file_data: &[u8], filename: &str, boundary: &str) -> String {
    let mut body = String::new();
    
    body.push_str(&format!("--{}\r\n", boundary));
    body.push_str(&format!("Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n", filename));
    body.push_str("Content-Type: image/png\r\n");
    body.push_str("\r\n");
    
    // Convert binary data to string (this is a simplification for testing)
    // In real multipart, binary data would be included as-is
    let base64_data = base64_encode(file_data);
    body.push_str(&base64_data);
    
    body.push_str("\r\n");
    body.push_str(&format!("--{}--\r\n", boundary));
    
    body
}

/// Simple base64 encoding for test data
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);
        
        result.push(CHARS[(b1 >> 2) as usize] as char);
        result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
        
        if chunk.len() > 1 {
            result.push(CHARS[(((b2 & 0x0F) << 2) | (b3 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(CHARS[(b3 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    
    result
}

/// Create test code snippets for various languages
pub fn get_test_code_snippets() -> Vec<(&'static str, &'static str)> {
    vec![
        ("rust", "fn main() {\n    println!(\"Hello, world!\");\n}"),
        ("python", "def hello():\n    print('Hello, world!')"),
        ("javascript", "function hello() {\n    console.log('Hello, world!');\n}"),
        ("java", "public class Hello {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, world!\");\n    }\n}"),
        ("go", "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello, world!\")\n}"),
        ("cpp", "#include <iostream>\n\nint main() {\n    std::cout << \"Hello, world!\" << std::endl;\n    return 0;\n}"),
    ]
}
