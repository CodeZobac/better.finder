use crate::types::SearchResult;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

/// Cache entry with timestamp for TTL
#[derive(Clone)]
struct CacheEntry {
    results: Vec<SearchResult>,
    timestamp: Instant,
}

/// LRU cache for search results with TTL support
pub struct ResultCache {
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    ttl: Duration,
}

impl ResultCache {
    /// Creates a new ResultCache with specified capacity and TTL
    pub fn new(capacity: usize, ttl_seconds: u64) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap());
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Gets cached results for a query if they exist and are not expired
    pub async fn get(&self, query: &str) -> Option<Vec<SearchResult>> {
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get(query) {
            // Check if entry is still valid (not expired)
            if entry.timestamp.elapsed() < self.ttl {
                debug!("Cache hit for query: '{}'", query);
                return Some(entry.results.clone());
            } else {
                debug!("Cache entry expired for query: '{}'", query);
                // Remove expired entry
                cache.pop(query);
            }
        }
        
        debug!("Cache miss for query: '{}'", query);
        None
    }

    /// Stores search results in the cache
    pub async fn put(&self, query: String, results: Vec<SearchResult>) {
        let mut cache = self.cache.write().await;
        
        let entry = CacheEntry {
            results,
            timestamp: Instant::now(),
        };
        
        cache.put(query.clone(), entry);
        debug!("Cached results for query: '{}'", query);
    }

    /// Invalidates all cached entries
    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("Cache invalidated");
    }

    /// Invalidates a specific query from the cache
    pub async fn invalidate(&self, query: &str) {
        let mut cache = self.cache.write().await;
        cache.pop(query);
        debug!("Invalidated cache for query: '{}'", query);
    }

    /// Returns the number of entries currently in the cache
    pub async fn len(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Returns whether the cache is empty
    pub async fn is_empty(&self) -> bool {
        let cache = self.cache.read().await;
        cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ResultAction, ResultType};
    use std::collections::HashMap;

    fn create_test_result(id: &str, title: &str) -> SearchResult {
        SearchResult {
            id: id.to_string(),
            title: title.to_string(),
            subtitle: "test".to_string(),
            icon: None,
            result_type: ResultType::File,
            score: 1.0,
            metadata: HashMap::new(),
            action: ResultAction::OpenFile {
                path: "/test".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_cache_put_and_get() {
        let cache = ResultCache::new(10, 5);
        let results = vec![create_test_result("1", "test")];
        
        cache.put("query".to_string(), results.clone()).await;
        
        let cached = cache.get("query").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = ResultCache::new(10, 5);
        
        let cached = cache.get("nonexistent").await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = ResultCache::new(10, 1); // 1 second TTL
        let results = vec![create_test_result("1", "test")];
        
        cache.put("query".to_string(), results).await;
        
        // Should be cached immediately
        assert!(cache.get("query").await.is_some());
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Should be expired
        assert!(cache.get("query").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let cache = ResultCache::new(10, 5);
        let results = vec![create_test_result("1", "test")];
        
        cache.put("query".to_string(), results).await;
        assert!(cache.get("query").await.is_some());
        
        cache.invalidate("query").await;
        assert!(cache.get("query").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate_all() {
        let cache = ResultCache::new(10, 5);
        
        cache.put("query1".to_string(), vec![create_test_result("1", "test1")]).await;
        cache.put("query2".to_string(), vec![create_test_result("2", "test2")]).await;
        
        assert_eq!(cache.len().await, 2);
        
        cache.invalidate_all().await;
        
        assert_eq!(cache.len().await, 0);
        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = ResultCache::new(2, 5); // Only 2 entries
        
        cache.put("query1".to_string(), vec![create_test_result("1", "test1")]).await;
        cache.put("query2".to_string(), vec![create_test_result("2", "test2")]).await;
        cache.put("query3".to_string(), vec![create_test_result("3", "test3")]).await;
        
        // query1 should be evicted (LRU)
        assert!(cache.get("query1").await.is_none());
        assert!(cache.get("query2").await.is_some());
        assert!(cache.get("query3").await.is_some());
    }
}
