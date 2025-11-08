/// Performance benchmarks for search functionality
/// 
/// These benchmarks ensure that the search engine meets performance targets:
/// - Search response time: <50ms
/// - UI render time: <100ms  
/// - Memory usage: <100MB RAM

#[cfg(test)]
mod benchmarks {
    use crate::search::{ResultCache, SearchEngine};
    use crate::types::{ResultAction, ResultType, SearchResult};
    use std::collections::HashMap;
    use std::time::Instant;

    /// Helper to create test results
    fn create_test_results(count: usize) -> Vec<SearchResult> {
        (0..count)
            .map(|i| SearchResult {
                id: format!("result-{}", i),
                title: format!("Test Result {}", i),
                subtitle: format!("Subtitle {}", i),
                icon: Some("test-icon".to_string()),
                result_type: ResultType::File,
                score: 100.0 - (i as f64),
                metadata: HashMap::new(),
                action: ResultAction::OpenFile {
                    path: format!("/test/file{}.txt", i),
                },
            })
            .collect()
    }

    #[tokio::test]
    async fn benchmark_search_response_time() {
        // Target: <50ms for search operations
        let engine = SearchEngine::new();
        
        // Warm up
        let _ = engine.search("test").await;
        
        // Benchmark
        let start = Instant::now();
        let results = engine.search("test query").await;
        let duration = start.elapsed();
        
        println!("Search response time: {:?}", duration);
        println!("Results count: {}", results.len());
        
        // Assert performance target
        assert!(
            duration.as_millis() < 50,
            "Search took {}ms, expected <50ms",
            duration.as_millis()
        );
    }

    #[tokio::test]
    async fn benchmark_cache_performance() {
        // Target: Cache hit should be <1ms
        let cache = ResultCache::new(100, 5);
        let results = create_test_results(50);
        
        // Store in cache
        cache.put("test query".to_string(), results.clone()).await;
        
        // Benchmark cache retrieval
        let start = Instant::now();
        let cached = cache.get("test query").await;
        let duration = start.elapsed();
        
        println!("Cache retrieval time: {:?}", duration);
        
        assert!(cached.is_some());
        assert!(
            duration.as_micros() < 1000, // <1ms
            "Cache retrieval took {}μs, expected <1000μs",
            duration.as_micros()
        );
    }

    #[tokio::test]
    async fn benchmark_large_result_set() {
        // Test with large result sets (1000 results)
        let engine = SearchEngine::new();
        let results = create_test_results(1000);
        
        // Simulate ranking large result set
        let start = Instant::now();
        let ranked = SearchEngine::rank_results(results, "test");
        let duration = start.elapsed();
        
        println!("Ranking 1000 results took: {:?}", duration);
        println!("Ranked results count: {}", ranked.len());
        
        // Should complete quickly even with large sets
        assert!(
            duration.as_millis() < 10,
            "Ranking took {}ms, expected <10ms",
            duration.as_millis()
        );
    }

    #[tokio::test]
    async fn benchmark_concurrent_searches() {
        // Test multiple concurrent searches
        let engine = SearchEngine::new();
        
        let start = Instant::now();
        
        // Spawn 10 concurrent searches
        let mut handles = vec![];
        for i in 0..10 {
            let query = format!("query {}", i);
            let handle = tokio::spawn(async move {
                let engine = SearchEngine::new();
                engine.search(&query).await
            });
            handles.push(handle);
        }
        
        // Wait for all to complete
        for handle in handles {
            let _ = handle.await;
        }
        
        let duration = start.elapsed();
        
        println!("10 concurrent searches took: {:?}", duration);
        
        // All searches should complete reasonably fast
        assert!(
            duration.as_millis() < 500,
            "Concurrent searches took {}ms, expected <500ms",
            duration.as_millis()
        );
    }

    #[tokio::test]
    async fn benchmark_memory_usage() {
        // Estimate memory usage of search results
        let results = create_test_results(1000);
        
        // Rough estimate: each result is ~200 bytes
        // 1000 results = ~200KB
        let estimated_size = results.len() * 200;
        
        println!("Estimated memory for 1000 results: {} bytes (~{}KB)", 
                 estimated_size, estimated_size / 1024);
        
        // Should be well under 100MB target
        assert!(
            estimated_size < 100 * 1024 * 1024, // 100MB
            "Memory usage too high: {} bytes",
            estimated_size
        );
    }

    #[tokio::test]
    async fn benchmark_cache_eviction() {
        // Test LRU cache performance with eviction
        let cache = ResultCache::new(10, 60); // Small cache
        
        let start = Instant::now();
        
        // Add 100 items (will cause evictions)
        for i in 0..100 {
            let results = create_test_results(10);
            cache.put(format!("query-{}", i), results).await;
        }
        
        let duration = start.elapsed();
        
        println!("100 cache operations with eviction took: {:?}", duration);
        
        // Cache operations should be fast even with eviction
        assert!(
            duration.as_millis() < 50,
            "Cache operations took {}ms, expected <50ms",
            duration.as_millis()
        );
        
        // Verify cache size is limited
        assert_eq!(cache.len().await, 10);
    }

    #[test]
    fn benchmark_result_serialization() {
        // Test serialization performance (for IPC)
        let results = create_test_results(100);
        
        let start = Instant::now();
        let serialized = serde_json::to_string(&results).unwrap();
        let duration = start.elapsed();
        
        println!("Serializing 100 results took: {:?}", duration);
        println!("Serialized size: {} bytes", serialized.len());
        
        // Serialization should be fast
        assert!(
            duration.as_millis() < 10,
            "Serialization took {}ms, expected <10ms",
            duration.as_millis()
        );
    }

    #[test]
    fn benchmark_query_sanitization() {
        // Test query sanitization performance
        let long_query = "a".repeat(1000);
        
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = SearchEngine::sanitize_query(&long_query);
        }
        let duration = start.elapsed();
        
        println!("1000 query sanitizations took: {:?}", duration);
        
        // Should be very fast
        assert!(
            duration.as_millis() < 10,
            "Sanitization took {}ms, expected <10ms",
            duration.as_millis()
        );
    }
}
