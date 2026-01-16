// Library exports for testing and external use

pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;

use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<utils::config::AppConfig>,
    pub storage: Arc<services::file_storage::FileStorageService>,
    pub session_manager: Arc<services::session_manager::SessionManager>,
    pub cache_manager: Arc<services::cache_manager::CacheManager<String, Vec<u8>>>,
    pub rate_limiter: Arc<services::rate_limiter::RateLimiter>,
}
