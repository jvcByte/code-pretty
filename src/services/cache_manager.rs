use crate::models::errors::AppError;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Cached item with expiration
#[derive(Debug, Clone)]
struct CachedItem<V> {
    value: V,
    created_at: SystemTime,
    expires_at: Option<SystemTime>,
    access_count: usize,
    last_accessed: SystemTime,
}

impl<V> CachedItem<V> {
    fn new(value: V, ttl: Option<Duration>) -> Self {
        let now = SystemTime::now();
        let expires_at = ttl.map(|duration| now + duration);
        
        Self {
            value,
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }

    fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
        self.access_count += 1;
    }
}

/// Generic cache manager with TTL support
#[derive(Clone)]
pub struct CacheManager<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    cache: Arc<RwLock<HashMap<K, CachedItem<V>>>>,
    default_ttl: Option<Duration>,
    max_size: Option<usize>,
}

impl<K, V> CacheManager<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Creates a new cache manager with no TTL and no size limit
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: None,
            max_size: None,
        }
    }

    /// Creates a new cache manager with default TTL
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Some(ttl),
            max_size: None,
        }
    }

    /// Creates a new cache manager with max size
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: None,
            max_size: Some(max_size),
        }
    }

    /// Creates a new cache manager with both TTL and max size
    pub fn with_ttl_and_max_size(ttl: Duration, max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Some(ttl),
            max_size: Some(max_size),
        }
    }

    /// Inserts a value into the cache with default TTL
    pub async fn insert(&self, key: K, value: V) -> Result<(), AppError> {
        self.insert_with_ttl(key, value, self.default_ttl).await
    }

    /// Inserts a value into the cache with custom TTL
    pub async fn insert_with_ttl(
        &self,
        key: K,
        value: V,
        ttl: Option<Duration>,
    ) -> Result<(), AppError> {
        let mut cache = self.cache.write().await;
        
        // Check if we need to evict items due to size limit
        if let Some(max_size) = self.max_size {
            if cache.len() >= max_size && !cache.contains_key(&key) {
                self.evict_lru(&mut cache);
            }
        }
        
        let item = CachedItem::new(value, ttl);
        cache.insert(key, item);
        
        Ok(())
    }

    /// Gets a value from the cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;
        
        if let Some(item) = cache.get_mut(key) {
            if item.is_expired() {
                cache.remove(key);
                None
            } else {
                item.touch();
                Some(item.value.clone())
            }
        } else {
            None
        }
    }

    /// Checks if a key exists in the cache (without updating access time)
    pub async fn contains_key(&self, key: &K) -> bool {
        let cache = self.cache.read().await;
        
        if let Some(item) = cache.get(key) {
            !item.is_expired()
        } else {
            false
        }
    }

    /// Removes a value from the cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;
        cache.remove(key).map(|item| item.value)
    }

    /// Clears all items from the cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Removes expired items from the cache
    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.cache.write().await;
        let initial_count = cache.len();
        
        cache.retain(|_, item| !item.is_expired());
        
        let removed_count = initial_count - cache.len();
        
        if removed_count > 0 {
            tracing::debug!("Cleaned up {} expired cache items", removed_count);
        }
        
        removed_count
    }

    /// Gets the number of items in the cache
    pub async fn len(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Checks if the cache is empty
    pub async fn is_empty(&self) -> bool {
        let cache = self.cache.read().await;
        cache.is_empty()
    }

    /// Gets cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        
        let total_items = cache.len();
        let expired_items = cache.values().filter(|item| item.is_expired()).count();
        let active_items = total_items - expired_items;
        
        let total_accesses: usize = cache.values().map(|item| item.access_count).sum();
        let avg_accesses = if total_items > 0 {
            total_accesses as f64 / total_items as f64
        } else {
            0.0
        };
        
        CacheStats {
            total_items,
            active_items,
            expired_items,
            total_accesses,
            avg_accesses,
            max_size: self.max_size,
            default_ttl_seconds: self.default_ttl.map(|d| d.as_secs()),
        }
    }

    /// Evicts the least recently used item
    fn evict_lru(&self, cache: &mut HashMap<K, CachedItem<V>>) {
        if let Some((key_to_remove, _)) = cache
            .iter()
            .min_by_key(|(_, item)| item.last_accessed)
        {
            let key_to_remove = key_to_remove.clone();
            cache.remove(&key_to_remove);
            tracing::debug!("Evicted LRU cache item");
        }
    }
}

impl<K, V> Default for CacheManager<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_items: usize,
    pub active_items: usize,
    pub expired_items: usize,
    pub total_accesses: usize,
    pub avg_accesses: f64,
    pub max_size: Option<usize>,
    pub default_ttl_seconds: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache: CacheManager<String, String> = CacheManager::new();
        
        cache.insert("key1".to_string(), "value1".to_string()).await.unwrap();
        
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        let cache: CacheManager<String, String> = 
            CacheManager::with_ttl(Duration::from_millis(100));
        
        cache.insert("key1".to_string(), "value1".to_string()).await.unwrap();
        
        // Should exist immediately
        assert!(cache.contains_key(&"key1".to_string()).await);
        
        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should be expired
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_max_size() {
        let cache: CacheManager<String, String> = CacheManager::with_max_size(3);
        
        cache.insert("key1".to_string(), "value1".to_string()).await.unwrap();
        cache.insert("key2".to_string(), "value2".to_string()).await.unwrap();
        cache.insert("key3".to_string(), "value3".to_string()).await.unwrap();
        
        assert_eq!(cache.len().await, 3);
        
        // This should evict the LRU item
        cache.insert("key4".to_string(), "value4".to_string()).await.unwrap();
        
        assert_eq!(cache.len().await, 3);
    }

    #[tokio::test]
    async fn test_cache_remove() {
        let cache: CacheManager<String, String> = CacheManager::new();
        
        cache.insert("key1".to_string(), "value1".to_string()).await.unwrap();
        
        let removed = cache.remove(&"key1".to_string()).await;
        assert_eq!(removed, Some("value1".to_string()));
        
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache: CacheManager<String, String> = CacheManager::new();
        
        cache.insert("key1".to_string(), "value1".to_string()).await.unwrap();
        cache.insert("key2".to_string(), "value2".to_string()).await.unwrap();
        
        assert_eq!(cache.len().await, 2);
        
        cache.clear().await;
        
        assert_eq!(cache.len().await, 0);
    }

    #[tokio::test]
    async fn test_cache_cleanup_expired() {
        let cache: CacheManager<String, String> = 
            CacheManager::with_ttl(Duration::from_millis(100));
        
        cache.insert("key1".to_string(), "value1".to_string()).await.unwrap();
        cache.insert("key2".to_string(), "value2".to_string()).await.unwrap();
        
        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        let removed = cache.cleanup_expired().await;
        assert_eq!(removed, 2);
        assert_eq!(cache.len().await, 0);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache: CacheManager<String, String> = CacheManager::new();
        
        cache.insert("key1".to_string(), "value1".to_string()).await.unwrap();
        cache.insert("key2".to_string(), "value2".to_string()).await.unwrap();
        
        // Access key1 multiple times
        cache.get(&"key1".to_string()).await;
        cache.get(&"key1".to_string()).await;
        cache.get(&"key2".to_string()).await;
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.total_items, 2);
        assert_eq!(stats.active_items, 2);
        assert_eq!(stats.total_accesses, 3);
    }
}
