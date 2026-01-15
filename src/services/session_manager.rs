use crate::models::errors::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Session data stored for each user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session_id: String,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub data: HashMap<String, serde_json::Value>,
}

impl SessionData {
    /// Creates a new session
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            created_at: now,
            last_accessed: now,
            data: HashMap::new(),
        }
    }

    /// Updates the last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
    }

    /// Checks if the session has expired
    pub fn is_expired(&self, expiry_duration: Duration) -> bool {
        if let Ok(elapsed) = self.last_accessed.elapsed() {
            elapsed > expiry_duration
        } else {
            true
        }
    }

    /// Sets a value in the session data
    pub fn set(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
        self.touch();
    }

    /// Gets a value from the session data
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// Removes a value from the session data
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        let result = self.data.remove(key);
        self.touch();
        result
    }

    /// Clears all session data
    pub fn clear(&mut self) {
        self.data.clear();
        self.touch();
    }
}

/// Session manager for handling user sessions
#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    expiry_duration: Duration,
}

impl SessionManager {
    /// Creates a new SessionManager with default expiry (1 hour)
    pub fn new() -> Self {
        Self::with_expiry(Duration::from_secs(3600))
    }

    /// Creates a new SessionManager with custom expiry duration
    pub fn with_expiry(expiry_duration: Duration) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            expiry_duration,
        }
    }

    /// Creates a new session and returns the session ID
    pub async fn create_session(&self) -> String {
        let session = SessionData::new();
        let session_id = session.session_id.clone();
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        tracing::debug!("Created new session: {}", session_id);
        session_id
    }

    /// Gets a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<SessionData> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Updates a session's last accessed time
    pub async fn touch_session(&self, session_id: &str) -> Result<(), AppError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.touch();
            Ok(())
        } else {
            Err(AppError::SessionError {
                message: format!("Session not found: {}", session_id),
            })
        }
    }

    /// Sets a value in a session
    pub async fn set_session_data(
        &self,
        session_id: &str,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), AppError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.set(key, value);
            Ok(())
        } else {
            Err(AppError::SessionError {
                message: format!("Session not found: {}", session_id),
            })
        }
    }

    /// Gets a value from a session
    pub async fn get_session_data(
        &self,
        session_id: &str,
        key: &str,
    ) -> Result<Option<serde_json::Value>, AppError> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            Ok(session.get(key).cloned())
        } else {
            Err(AppError::SessionError {
                message: format!("Session not found: {}", session_id),
            })
        }
    }

    /// Removes a value from a session
    pub async fn remove_session_data(
        &self,
        session_id: &str,
        key: &str,
    ) -> Result<Option<serde_json::Value>, AppError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            Ok(session.remove(key))
        } else {
            Err(AppError::SessionError {
                message: format!("Session not found: {}", session_id),
            })
        }
    }

    /// Destroys a session
    pub async fn destroy_session(&self, session_id: &str) -> Result<(), AppError> {
        let mut sessions = self.sessions.write().await;
        
        if sessions.remove(session_id).is_some() {
            tracing::debug!("Destroyed session: {}", session_id);
            Ok(())
        } else {
            Err(AppError::SessionError {
                message: format!("Session not found: {}", session_id),
            })
        }
    }

    /// Cleans up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<usize, AppError> {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        
        sessions.retain(|_, session| !session.is_expired(self.expiry_duration));
        
        let removed_count = initial_count - sessions.len();
        
        if removed_count > 0 {
            tracing::info!("Cleaned up {} expired sessions", removed_count);
        }
        
        Ok(removed_count)
    }

    /// Gets the number of active sessions
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Gets session statistics
    pub async fn get_stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;
        let total_sessions = sessions.len();
        
        let expired_sessions = sessions
            .values()
            .filter(|s| s.is_expired(self.expiry_duration))
            .count();
        
        let active_sessions = total_sessions - expired_sessions;
        
        SessionStats {
            total_sessions,
            active_sessions,
            expired_sessions,
            expiry_duration_seconds: self.expiry_duration.as_secs(),
        }
    }

    /// Validates if a session exists and is not expired
    pub async fn validate_session(&self, session_id: &str) -> bool {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            !session.is_expired(self.expiry_duration)
        } else {
            false
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub expired_sessions: usize,
    pub expiry_duration_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let manager = SessionManager::new();
        let session_id = manager.create_session().await;
        
        assert!(!session_id.is_empty());
        assert!(manager.validate_session(&session_id).await);
    }

    #[tokio::test]
    async fn test_session_data() {
        let manager = SessionManager::new();
        let session_id = manager.create_session().await;
        
        // Set data
        manager
            .set_session_data(&session_id, "key1".to_string(), serde_json::json!("value1"))
            .await
            .unwrap();
        
        // Get data
        let value = manager.get_session_data(&session_id, "key1").await.unwrap();
        assert_eq!(value, Some(serde_json::json!("value1")));
        
        // Remove data
        let removed = manager.remove_session_data(&session_id, "key1").await.unwrap();
        assert_eq!(removed, Some(serde_json::json!("value1")));
        
        // Verify removed
        let value = manager.get_session_data(&session_id, "key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_destroy_session() {
        let manager = SessionManager::new();
        let session_id = manager.create_session().await;
        
        assert!(manager.validate_session(&session_id).await);
        
        manager.destroy_session(&session_id).await.unwrap();
        
        assert!(!manager.validate_session(&session_id).await);
    }

    #[tokio::test]
    async fn test_session_expiry() {
        let manager = SessionManager::with_expiry(Duration::from_millis(100));
        let session_id = manager.create_session().await;
        
        assert!(manager.validate_session(&session_id).await);
        
        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        assert!(!manager.validate_session(&session_id).await);
    }

    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let manager = SessionManager::with_expiry(Duration::from_millis(100));
        
        // Create multiple sessions
        for _ in 0..5 {
            manager.create_session().await;
        }
        
        assert_eq!(manager.session_count().await, 5);
        
        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Cleanup
        let removed = manager.cleanup_expired_sessions().await.unwrap();
        assert_eq!(removed, 5);
        assert_eq!(manager.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_stats() {
        let manager = SessionManager::new();
        
        // Create sessions
        for _ in 0..3 {
            manager.create_session().await;
        }
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_sessions, 3);
        assert_eq!(stats.active_sessions, 3);
        assert_eq!(stats.expired_sessions, 0);
    }
}
