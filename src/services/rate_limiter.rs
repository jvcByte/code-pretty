use crate::models::errors::AppError;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the time window
    pub max_requests: usize,
    /// Time window for rate limiting
    pub window_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_duration: Duration::from_secs(60),
        }
    }
}

/// Request record for tracking
#[derive(Debug, Clone)]
struct RequestRecord {
    timestamps: Vec<SystemTime>,
}

impl RequestRecord {
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
        }
    }

    /// Adds a new request timestamp
    fn add_request(&mut self, now: SystemTime) {
        self.timestamps.push(now);
    }

    /// Removes expired timestamps outside the window
    fn cleanup(&mut self, window_duration: Duration) {
        let now = SystemTime::now();
        self.timestamps.retain(|&timestamp| {
            if let Ok(elapsed) = now.duration_since(timestamp) {
                elapsed < window_duration
            } else {
                false
            }
        });
    }

    /// Gets the number of requests in the current window
    fn request_count(&self) -> usize {
        self.timestamps.len()
    }
}

/// Rate limiter service
#[derive(Clone)]
pub struct RateLimiter {
    records: Arc<RwLock<HashMap<String, RequestRecord>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Creates a new rate limiter with default configuration
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    /// Creates a new rate limiter with custom configuration
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Checks if a request from the given identifier is allowed
    pub async fn check_rate_limit(&self, identifier: &str) -> Result<(), AppError> {
        let mut records = self.records.write().await;
        
        // Get or create record for this identifier
        let record = records.entry(identifier.to_string()).or_insert_with(RequestRecord::new);
        
        // Cleanup old timestamps
        record.cleanup(self.config.window_duration);
        
        // Check if limit is exceeded
        if record.request_count() >= self.config.max_requests {
            return Err(AppError::rate_limited(format!(
                "Rate limit exceeded: {} requests per {} seconds",
                self.config.max_requests,
                self.config.window_duration.as_secs()
            )));
        }
        
        // Record this request
        record.add_request(SystemTime::now());
        
        Ok(())
    }

    /// Gets the current request count for an identifier
    pub async fn get_request_count(&self, identifier: &str) -> usize {
        let mut records = self.records.write().await;
        
        if let Some(record) = records.get_mut(identifier) {
            record.cleanup(self.config.window_duration);
            record.request_count()
        } else {
            0
        }
    }

    /// Gets the remaining requests for an identifier
    pub async fn get_remaining_requests(&self, identifier: &str) -> usize {
        let count = self.get_request_count(identifier).await;
        self.config.max_requests.saturating_sub(count)
    }

    /// Resets the rate limit for an identifier
    pub async fn reset(&self, identifier: &str) {
        let mut records = self.records.write().await;
        records.remove(identifier);
    }

    /// Cleans up all expired records
    pub async fn cleanup_expired(&self) -> usize {
        let mut records = self.records.write().await;
        let initial_count = records.len();
        
        // Cleanup each record
        for record in records.values_mut() {
            record.cleanup(self.config.window_duration);
        }
        
        // Remove empty records
        records.retain(|_, record| record.request_count() > 0);
        
        let removed_count = initial_count - records.len();
        
        if removed_count > 0 {
            tracing::debug!("Cleaned up {} expired rate limit records", removed_count);
        }
        
        removed_count
    }

    /// Gets rate limiter statistics
    pub async fn get_stats(&self) -> RateLimiterStats {
        let records = self.records.read().await;
        
        let total_identifiers = records.len();
        let total_requests: usize = records.values().map(|r| r.request_count()).sum();
        
        let avg_requests = if total_identifiers > 0 {
            total_requests as f64 / total_identifiers as f64
        } else {
            0.0
        };
        
        RateLimiterStats {
            total_identifiers,
            total_requests,
            avg_requests,
            max_requests: self.config.max_requests,
            window_duration_seconds: self.config.window_duration.as_secs(),
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiter statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimiterStats {
    pub total_identifiers: usize,
    pub total_requests: usize,
    pub avg_requests: f64,
    pub max_requests: usize,
    pub window_duration_seconds: u64,
}

/// Helper function to extract identifier from IP address
pub fn identifier_from_ip(ip: IpAddr) -> String {
    ip.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_requests() {
        let config = RateLimitConfig {
            max_requests: 3,
            window_duration: Duration::from_secs(60),
        };
        let limiter = RateLimiter::with_config(config);
        
        // First 3 requests should be allowed
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        
        // 4th request should be denied
        assert!(limiter.check_rate_limit("user1").await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_different_identifiers() {
        let config = RateLimitConfig {
            max_requests: 2,
            window_duration: Duration::from_secs(60),
        };
        let limiter = RateLimiter::with_config(config);
        
        // Different identifiers should have separate limits
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        assert!(limiter.check_rate_limit("user2").await.is_ok());
        assert!(limiter.check_rate_limit("user2").await.is_ok());
        
        // Both should be rate limited now
        assert!(limiter.check_rate_limit("user1").await.is_err());
        assert!(limiter.check_rate_limit("user2").await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_window_expiry() {
        let config = RateLimitConfig {
            max_requests: 2,
            window_duration: Duration::from_millis(100),
        };
        let limiter = RateLimiter::with_config(config);
        
        // Use up the limit
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        assert!(limiter.check_rate_limit("user1").await.is_err());
        
        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should be allowed again
        assert!(limiter.check_rate_limit("user1").await.is_ok());
    }

    #[tokio::test]
    async fn test_get_remaining_requests() {
        let config = RateLimitConfig {
            max_requests: 5,
            window_duration: Duration::from_secs(60),
        };
        let limiter = RateLimiter::with_config(config);
        
        assert_eq!(limiter.get_remaining_requests("user1").await, 5);
        
        limiter.check_rate_limit("user1").await.unwrap();
        assert_eq!(limiter.get_remaining_requests("user1").await, 4);
        
        limiter.check_rate_limit("user1").await.unwrap();
        assert_eq!(limiter.get_remaining_requests("user1").await, 3);
    }

    #[tokio::test]
    async fn test_reset() {
        let config = RateLimitConfig {
            max_requests: 2,
            window_duration: Duration::from_secs(60),
        };
        let limiter = RateLimiter::with_config(config);
        
        // Use up the limit
        limiter.check_rate_limit("user1").await.unwrap();
        limiter.check_rate_limit("user1").await.unwrap();
        assert!(limiter.check_rate_limit("user1").await.is_err());
        
        // Reset
        limiter.reset("user1").await;
        
        // Should be allowed again
        assert!(limiter.check_rate_limit("user1").await.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let config = RateLimitConfig {
            max_requests: 5,
            window_duration: Duration::from_millis(100),
        };
        let limiter = RateLimiter::with_config(config);
        
        // Create some records
        limiter.check_rate_limit("user1").await.unwrap();
        limiter.check_rate_limit("user2").await.unwrap();
        limiter.check_rate_limit("user3").await.unwrap();
        
        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Cleanup
        let removed = limiter.cleanup_expired().await;
        assert_eq!(removed, 3);
    }

    #[tokio::test]
    async fn test_stats() {
        let config = RateLimitConfig {
            max_requests: 10,
            window_duration: Duration::from_secs(60),
        };
        let limiter = RateLimiter::with_config(config);
        
        limiter.check_rate_limit("user1").await.unwrap();
        limiter.check_rate_limit("user1").await.unwrap();
        limiter.check_rate_limit("user2").await.unwrap();
        
        let stats = limiter.get_stats().await;
        assert_eq!(stats.total_identifiers, 2);
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.max_requests, 10);
    }
}
