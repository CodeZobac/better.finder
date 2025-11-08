#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::error::Result;
    use crate::types::{ResultAction, ResultType, SearchResult};
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock search provider for testing
    struct MockProvider {
        name: String,
        priority: u8,
        results: Vec<SearchResult>,
        enabled: bool,
        should_fail: bool,
    }

    impl MockProvider {
        fn new(name: &str, priority: u8, result_count: usize) -> Self {
            let results = (0..result_count)
                .map(|i| SearchResult {
                    id: format!("{}-{}", name, i),
                    title: format!("Result {} from {}", i, name),
                    subtitle: format!("Subtitle {}", i),
                    icon: None,
                    result_type: ResultType::File,
                    score: (result_count - i) as f64,
                    metadata: HashMap::new(),
                    action: ResultAction::OpenFile {
                        path: format!("/path/to/file{}", i),
                    },
                })
                .collect();

            Self {
                name: name.to_string(),
                priority,
                results,
                enabled: true,
                should_fail: false,
            }
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }

        fn disabled(mut self) -> Self {
            self.enabled = false;
            self
        }
    }

    #[async_trait]
    impl SearchProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn priority(&self) -> u8 {
            self.priority
        }

        async fn search(&self, _query: &str) -> Result<Vec<SearchResult>> {
            if self.should_fail {
                return Err(crate::error::LauncherError::SearchError(
                    "Mock provider failure".to_string(),
                ));
            }
            Ok(self.results.clone())
        }

        async fn execute(&self, _result: &SearchResult) -> Result<()> {
            Ok(())
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    #[tokio::test]
    async fn test_provider_registration() {
        let engine = SearchEngine::new();
        
        assert_eq!(engine.provider_count().await, 0);

        let provider1 = Box::new(MockProvider::new("provider1", 50, 3));
        engine.register_provider(provider1).await;
        
        assert_eq!(engine.provider_count().await, 1);

        let provider2 = Box::new(MockProvider::new("provider2", 100, 3));
        engine.register_provider(provider2).await;
        
        assert_eq!(engine.provider_count().await, 2);

        let names = engine.provider_names().await;
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"provider1".to_string()));
        assert!(names.contains(&"provider2".to_string()));
    }

    #[tokio::test]
    async fn test_provider_priority_ordering() {
        let engine = SearchEngine::new();

        // Register providers in reverse priority order
        let provider_low = Box::new(MockProvider::new("low_priority", 10, 2));
        let provider_high = Box::new(MockProvider::new("high_priority", 100, 2));
        let provider_mid = Box::new(MockProvider::new("mid_priority", 50, 2));

        engine.register_provider(provider_low).await;
        engine.register_provider(provider_high).await;
        engine.register_provider(provider_mid).await;

        let names = engine.provider_names().await;
        
        // Providers should be ordered by priority (highest first)
        assert_eq!(names[0], "high_priority");
        assert_eq!(names[1], "mid_priority");
        assert_eq!(names[2], "low_priority");
    }

    #[tokio::test]
    async fn test_parallel_search_execution() {
        let engine = SearchEngine::new();

        let provider1 = Box::new(MockProvider::new("provider1", 50, 3));
        let provider2 = Box::new(MockProvider::new("provider2", 60, 4));
        let provider3 = Box::new(MockProvider::new("provider3", 70, 2));

        engine.register_provider(provider1).await;
        engine.register_provider(provider2).await;
        engine.register_provider(provider3).await;

        let results = engine.search("test query").await;

        // Should get results from all providers (3 + 4 + 2 = 9)
        assert_eq!(results.len(), 9);
    }

    #[tokio::test]
    async fn test_result_merging_and_ranking() {
        let engine = SearchEngine::new();

        // Create providers with different result counts
        let provider1 = Box::new(MockProvider::new("provider1", 50, 5));
        let provider2 = Box::new(MockProvider::new("provider2", 60, 3));

        engine.register_provider(provider1).await;
        engine.register_provider(provider2).await;

        let results = engine.search("test").await;

        // Should merge results from both providers
        assert_eq!(results.len(), 8);

        // Results should be sorted by score (descending)
        for i in 0..results.len() - 1 {
            assert!(results[i].score >= results[i + 1].score);
        }
    }

    #[tokio::test]
    async fn test_error_handling_graceful_degradation() {
        let engine = SearchEngine::new();

        // One provider that works, one that fails
        let good_provider = Box::new(MockProvider::new("good", 50, 3));
        let bad_provider = Box::new(MockProvider::new("bad", 60, 0).with_failure());

        engine.register_provider(good_provider).await;
        engine.register_provider(bad_provider).await;

        let results = engine.search("test").await;

        // Should still get results from the good provider despite one failing
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.id.starts_with("good")));
    }

    #[tokio::test]
    async fn test_disabled_provider_skipped() {
        let engine = SearchEngine::new();

        let enabled_provider = Box::new(MockProvider::new("enabled", 50, 3));
        let disabled_provider = Box::new(MockProvider::new("disabled", 60, 5).disabled());

        engine.register_provider(enabled_provider).await;
        engine.register_provider(disabled_provider).await;

        let results = engine.search("test").await;

        // Should only get results from enabled provider
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.id.starts_with("enabled")));
    }

    #[tokio::test]
    async fn test_empty_query_returns_no_results() {
        let engine = SearchEngine::new();

        let provider = Box::new(MockProvider::new("provider", 50, 5));
        engine.register_provider(provider).await;

        let results = engine.search("").await;
        assert_eq!(results.len(), 0);

        let results = engine.search("   ").await;
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_query_sanitization() {
        let engine = SearchEngine::new();

        let provider = Box::new(MockProvider::new("provider", 50, 3));
        engine.register_provider(provider).await;

        // Query with control characters should be sanitized
        let results = engine.search("test\x00query\x01").await;
        
        // Should still return results (query was sanitized, not rejected)
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_result_limit_per_provider() {
        let engine = SearchEngine::new();

        // Create provider with more than MAX_RESULTS_PER_PROVIDER results
        let provider = Box::new(MockProvider::new("provider", 50, 30));
        engine.register_provider(provider).await;

        let results = engine.search("test").await;

        // Should be limited to MAX_RESULTS_PER_PROVIDER (20)
        assert!(results.len() <= 20);
    }

    #[tokio::test]
    async fn test_total_result_limit() {
        let engine = SearchEngine::new();

        // Register multiple providers with many results each
        for i in 0..5 {
            let provider = Box::new(MockProvider::new(&format!("provider{}", i), 50 + i, 20));
            engine.register_provider(provider).await;
        }

        let results = engine.search("test").await;

        // Should be limited to MAX_TOTAL_RESULTS (50)
        assert!(results.len() <= 50);
    }

    #[tokio::test]
    async fn test_all_providers_integration() {
        // This test verifies that all providers can be registered together
        // and work in harmony without conflicts
        let engine = SearchEngine::new();

        // Register all providers in the order they would be in production
        
        // RecentFilesProvider
        if let Ok(provider) = crate::search::providers::RecentFilesProvider::new() {
            engine.register_provider(Box::new(provider)).await;
        }

        // FileSearchProvider (with fallback to WindowsSearch)
        match crate::search::providers::FileSearchProvider::new() {
            Ok(provider) => {
                if provider.is_enabled() {
                    engine.register_provider(Box::new(provider)).await;
                } else if let Ok(fallback) = crate::search::providers::WindowsSearchProvider::new() {
                    engine.register_provider(Box::new(fallback)).await;
                }
            }
            Err(_) => {
                if let Ok(fallback) = crate::search::providers::WindowsSearchProvider::new() {
                    engine.register_provider(Box::new(fallback)).await;
                }
            }
        }

        // CalculatorProvider
        if let Ok(provider) = crate::search::providers::CalculatorProvider::new() {
            engine.register_provider(Box::new(provider)).await;
        }

        // QuickActionProvider
        if let Ok(provider) = crate::search::providers::QuickActionProvider::new() {
            engine.register_provider(Box::new(provider)).await;
        }

        // AppSearchProvider
        if let Ok(provider) = crate::search::providers::AppSearchProvider::new() {
            engine.register_provider(Box::new(provider)).await;
        }

        // BookmarkProvider
        if let Ok(provider) = crate::search::providers::BookmarkProvider::new() {
            engine.register_provider(Box::new(provider)).await;
        }

        // ClipboardHistoryProvider
        if let Ok(provider) = crate::search::providers::ClipboardHistoryProvider::new() {
            engine.register_provider(Box::new(provider)).await;
        }

        // WebSearchProvider
        if let Ok(provider) = crate::search::providers::WebSearchProvider::new() {
            engine.register_provider(Box::new(provider)).await;
        }

        // Verify providers are registered
        let provider_count = engine.provider_count().await;
        assert!(provider_count >= 5, "Expected at least 5 providers, got {}", provider_count);

        let provider_names = engine.provider_names().await;
        println!("Registered providers: {:?}", provider_names);

        // Verify priority ordering
        // Calculator and RecentFiles should be high priority (90)
        // Apps should be 85
        // QuickActions should be 80
        // Bookmarks should be 50
        // Clipboard should be 60
        // WebSearch should be lowest (1)
        
        // Test a calculator query
        let calc_results = engine.search("2+2").await;
        if !calc_results.is_empty() {
            // Calculator should be in the results
            assert!(calc_results.iter().any(|r| r.result_type == ResultType::Calculator));
        }

        // Test a quick action query
        let action_results = engine.search("shutdown").await;
        if !action_results.is_empty() {
            // Quick action should be in the results
            assert!(action_results.iter().any(|r| r.result_type == ResultType::QuickAction));
        }

        println!("All providers integration test passed!");
    }

    #[tokio::test]
    async fn test_result_grouping_by_type() {
        let engine = SearchEngine::new();

        // Create mock providers that return different result types
        let file_results = vec![SearchResult {
            id: "file1".to_string(),
            title: "test.txt".to_string(),
            subtitle: "C:\\test.txt".to_string(),
            icon: None,
            result_type: ResultType::File,
            score: 80.0,
            metadata: HashMap::new(),
            action: ResultAction::OpenFile {
                path: "C:\\test.txt".to_string(),
            },
        }];

        let app_results = vec![SearchResult {
            id: "app1".to_string(),
            title: "Test App".to_string(),
            subtitle: "Application".to_string(),
            icon: None,
            result_type: ResultType::Application,
            score: 75.0,
            metadata: HashMap::new(),
            action: ResultAction::LaunchApp {
                path: "C:\\app.exe".to_string(),
            },
        }];

        struct TypedMockProvider {
            name: String,
            priority: u8,
            results: Vec<SearchResult>,
        }

        #[async_trait]
        impl SearchProvider for TypedMockProvider {
            fn name(&self) -> &str {
                &self.name
            }

            fn priority(&self) -> u8 {
                self.priority
            }

            async fn search(&self, _query: &str) -> Result<Vec<SearchResult>> {
                Ok(self.results.clone())
            }

            async fn execute(&self, _result: &SearchResult) -> Result<()> {
                Ok(())
            }

            fn is_enabled(&self) -> bool {
                true
            }
        }

        let file_provider = Box::new(TypedMockProvider {
            name: "files".to_string(),
            priority: 90,
            results: file_results,
        });

        let app_provider = Box::new(TypedMockProvider {
            name: "apps".to_string(),
            priority: 85,
            results: app_results,
        });

        engine.register_provider(file_provider).await;
        engine.register_provider(app_provider).await;

        let results = engine.search("test").await;

        // Verify we have both types
        assert_eq!(results.len(), 2);
        
        let file_count = results.iter().filter(|r| r.result_type == ResultType::File).count();
        let app_count = results.iter().filter(|r| r.result_type == ResultType::Application).count();
        
        assert_eq!(file_count, 1);
        assert_eq!(app_count, 1);

        // File should come first due to higher score
        assert_eq!(results[0].result_type, ResultType::File);
        assert_eq!(results[1].result_type, ResultType::Application);
    }
}
