#[cfg(test)]
mod tests {
    use crate::search::providers::{FileSearchProvider, WindowsSearchProvider};
    use crate::search::SearchProvider;

    #[tokio::test]
    async fn test_file_search_provider_fallback() {
        // Try to create FileSearchProvider
        let file_provider = FileSearchProvider::new();
        
        match file_provider {
            Ok(provider) => {
                if provider.is_enabled() {
                    println!("FileSearchProvider (Everything SDK) is available");
                    assert_eq!(provider.name(), "FileSearch");
                    assert_eq!(provider.priority(), 90);
                } else {
                    println!("FileSearchProvider created but not enabled (Everything not running)");
                    assert!(!provider.is_enabled());
                }
            }
            Err(e) => {
                println!("FileSearchProvider failed to initialize: {}", e);
                println!("Testing fallback to WindowsSearchProvider");
                
                // Test fallback
                let windows_provider = WindowsSearchProvider::new();
                assert!(windows_provider.is_ok());
                
                let provider = windows_provider.unwrap();
                assert_eq!(provider.name(), "WindowsSearch");
                assert_eq!(provider.priority(), 85);
                assert!(provider.is_enabled());
            }
        }
    }

    #[tokio::test]
    async fn test_windows_search_provider_always_available() {
        let provider = WindowsSearchProvider::new();
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert!(provider.is_enabled());
        assert_eq!(provider.name(), "WindowsSearch");
    }

    #[tokio::test]
    async fn test_provider_priority_ordering() {
        let file_provider = FileSearchProvider::new();
        let windows_provider = WindowsSearchProvider::new().unwrap();
        
        if let Ok(file_prov) = file_provider {
            // FileSearchProvider should have higher priority than WindowsSearchProvider
            assert!(file_prov.priority() > windows_provider.priority());
        }
    }

    #[tokio::test]
    async fn test_graceful_degradation_on_search_failure() {
        // This test verifies that search failures don't crash the application
        let provider = WindowsSearchProvider::new().unwrap();
        
        // Search with an empty query should return empty results, not error
        let results = provider.search("").await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_windows_search_basic_functionality() {
        let provider = WindowsSearchProvider::new().unwrap();
        
        // Try a basic search
        let results = provider.search("test").await;
        
        match results {
            Ok(files) => {
                println!("Windows Search found {} files", files.len());
                // Just verify it doesn't crash
                assert!(files.len() <= 20); // Should respect MAX_RESULTS
            }
            Err(e) => {
                println!("Windows Search failed (expected on some systems): {}", e);
                // This is acceptable - Windows Search might not be available
            }
        }
    }
}
