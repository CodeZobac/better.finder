/// Web search fallback provider
///
/// This provider detects queries that should trigger a web search:
/// - Queries with no local results
/// - Queries containing question words (how, what, why, when, where, who)
/// - Natural language queries

use crate::error::{LauncherError, Result};
use crate::search::SearchProvider;
use crate::types::{ResultAction, ResultType, SearchResult};
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, info};

/// Web search provider for fallback searches
pub struct WebSearchProvider {
    /// Whether the provider is enabled
    enabled: bool,
    /// Regex for detecting question words
    question_pattern: Regex,
}

impl WebSearchProvider {
    /// Creates a new WebSearchProvider
    pub fn new() -> Result<Self> {
        info!("Initializing WebSearchProvider");

        // Pattern to detect question words at the start of queries
        // Matches: how, what, why, when, where, who (case-insensitive)
        let question_pattern = Regex::new(r"(?i)^\s*(how|what|why|when|where|who)\b")
            .map_err(|e| LauncherError::ExecutionError(format!("Failed to compile regex: {}", e)))?;

        Ok(Self {
            enabled: true,
            question_pattern,
        })
    }

    /// Checks if a query contains question words
    pub fn has_question_words(&self, query: &str) -> bool {
        self.question_pattern.is_match(query)
    }

    /// Classifies whether a query should trigger a web search
    /// 
    /// Returns true if:
    /// - Query contains question words (how, what, why, when, where, who)
    /// - Query is a natural language phrase (contains multiple words)
    pub fn should_trigger_web_search(&self, query: &str, has_local_results: bool) -> bool {
        let trimmed = query.trim();
        
        // Don't trigger on empty queries
        if trimmed.is_empty() {
            return false;
        }

        // Don't trigger on very short queries (likely file/app names)
        if trimmed.len() < 3 {
            return false;
        }

        // Check for question words
        if self.has_question_words(trimmed) {
            debug!("Query contains question words, triggering web search");
            return true;
        }

        // If there are no local results and query looks like a search phrase
        // (contains spaces and is reasonably long), suggest web search
        if !has_local_results && trimmed.contains(' ') && trimmed.len() > 5 {
            debug!("No local results for multi-word query, suggesting web search");
            return true;
        }

        false
    }

    /// Creates a web search result for the given query
    fn create_web_search_result(&self, query: &str) -> SearchResult {
        let mut metadata = HashMap::new();
        metadata.insert("query".to_string(), serde_json::json!(query));
        metadata.insert("search_engine".to_string(), serde_json::json!("Google"));

        SearchResult {
            id: format!("web_search:{}", query),
            title: format!("Search Google for \"{}\"", query),
            subtitle: "Press Enter to search on the web".to_string(),
            icon: Some("web".to_string()),
            result_type: ResultType::WebSearch,
            score: 10.0, // Low score so it appears at the bottom
            metadata,
            action: ResultAction::WebSearch {
                query: query.to_string(),
            },
        }
    }
}

#[async_trait]
impl SearchProvider for WebSearchProvider {
    fn name(&self) -> &str {
        "WebSearch"
    }

    fn priority(&self) -> u8 {
        1 // Lowest priority - fallback option
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let trimmed = query.trim();
        
        // For now, always return a web search option if query has question words
        // The actual decision to show this will be made by checking if other results exist
        if self.has_question_words(trimmed) {
            debug!("Creating web search result for question query: '{}'", trimmed);
            let result = self.create_web_search_result(trimmed);
            return Ok(vec![result]);
        }

        // For other queries, we'll return a web search option with very low score
        // so it only shows up when there are few other results
        if trimmed.len() >= 3 {
            debug!("Creating fallback web search result for: '{}'", trimmed);
            let result = self.create_web_search_result(trimmed);
            return Ok(vec![result]);
        }

        Ok(Vec::new())
    }

    async fn execute(&self, result: &SearchResult) -> Result<()> {
        if result.result_type != ResultType::WebSearch {
            return Err(LauncherError::ExecutionError(
                "Not a web search result".to_string(),
            ));
        }

        // Extract the query from the action
        match &result.action {
            ResultAction::WebSearch { query } => {
                info!("Executing web search for: {}", query);
                Self::open_web_search(query).await?;
                Ok(())
            }
            _ => Err(LauncherError::ExecutionError(
                "Invalid action for web search result".to_string(),
            )),
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("WebSearchProvider initialized");
        Ok(())
    }
}

impl Default for WebSearchProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            enabled: false,
            question_pattern: Regex::new(r"(?i)^\s*(how|what|why|when|where|who)\b").unwrap(),
        })
    }
}

impl WebSearchProvider {
    /// Detects the default browser from Windows registry
    #[cfg(windows)]
    fn get_default_browser() -> Result<Option<String>> {
        use windows::Win32::System::Registry::*;
        use windows::Win32::Foundation::*;
        use windows::core::HSTRING;

        unsafe {
            // Try to read the default browser from the registry
            // HKEY_CURRENT_USER\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice
            let subkey = HSTRING::from("Software\\Microsoft\\Windows\\Shell\\Associations\\UrlAssociations\\http\\UserChoice");
            let mut hkey = HKEY::default();
            
            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                &subkey,
                0,
                KEY_READ,
                &mut hkey,
            );

            if result.is_err() {
                debug!("Could not open registry key for default browser detection");
                return Ok(None);
            }

            // Read the ProgId value
            let value_name = HSTRING::from("ProgId");
            let mut buffer = vec![0u16; 256];
            let mut buffer_size = (buffer.len() * 2) as u32;
            let mut value_type = REG_VALUE_TYPE::default();

            let result = RegQueryValueExW(
                hkey,
                &value_name,
                None,
                Some(&mut value_type),
                Some(buffer.as_mut_ptr() as *mut u8),
                Some(&mut buffer_size),
            );

            let _ = RegCloseKey(hkey);

            if result.is_ok() && value_type == REG_SZ {
                // Convert the buffer to a string
                let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
                let prog_id = String::from_utf16_lossy(&buffer[..len]);
                debug!("Detected default browser ProgId: {}", prog_id);
                return Ok(Some(prog_id));
            }

            Ok(None)
        }
    }

    /// Constructs a Google search URL with encoded query
    /// This function is platform-independent
    pub fn construct_search_url(query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://www.google.com/search?q={}", encoded_query)
    }

    /// Opens a web search in the default browser
    #[cfg(windows)]
    async fn open_web_search(query: &str) -> Result<()> {
        let search_url = Self::construct_search_url(query);
        
        info!("Opening web search URL: {}", search_url);

        // Detect default browser (for logging purposes)
        if let Ok(Some(browser)) = Self::get_default_browser() {
            debug!("Default browser: {}", browser);
        }

        tokio::task::spawn_blocking(move || -> Result<()> {
            // Use Windows shell to open the URL with the default browser
            // The "start" command will use the default browser automatically
            std::process::Command::new("cmd")
                .args(["/C", "start", "", &search_url])
                .spawn()
                .map_err(|e| LauncherError::ExecutionError(format!("Failed to open web search: {}", e)))?;
            Ok(())
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn web search task: {}", e))
        })??;

        Ok(())
    }

    #[cfg(not(windows))]
    fn get_default_browser() -> Result<Option<String>> {
        Ok(None)
    }

    #[cfg(not(windows))]
    async fn open_web_search(_query: &str) -> Result<()> {
        Err(LauncherError::ExecutionError(
            "Web search not supported on this platform".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_search_provider_creation() {
        let provider = WebSearchProvider::new();
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "WebSearch");
        assert_eq!(provider.priority(), 1);
        assert!(provider.is_enabled());
    }

    #[tokio::test]
    async fn test_has_question_words() {
        let provider = WebSearchProvider::new().unwrap();

        // Queries with question words
        assert!(provider.has_question_words("how to use keyboard"));
        assert!(provider.has_question_words("what is rust"));
        assert!(provider.has_question_words("why is the sky blue"));
        assert!(provider.has_question_words("when was windows released"));
        assert!(provider.has_question_words("where is the file"));
        assert!(provider.has_question_words("who created linux"));

        // Case insensitive
        assert!(provider.has_question_words("HOW TO CODE"));
        assert!(provider.has_question_words("What Is This"));
        assert!(provider.has_question_words("WHY NOT"));

        // With leading spaces
        assert!(provider.has_question_words("  how to test"));
        assert!(provider.has_question_words("   what is this"));

        // Queries without question words
        assert!(!provider.has_question_words("search query"));
        assert!(!provider.has_question_words("file.txt"));
        assert!(!provider.has_question_words("calculator"));
        assert!(!provider.has_question_words(""));
        assert!(!provider.has_question_words("   "));

        // Question words not at the start
        assert!(!provider.has_question_words("tell me how to code"));
        assert!(!provider.has_question_words("I wonder what this is"));
    }

    #[tokio::test]
    async fn test_should_trigger_web_search_with_question_words() {
        let provider = WebSearchProvider::new().unwrap();

        // Should trigger with question words regardless of local results
        assert!(provider.should_trigger_web_search("how to use rust", true));
        assert!(provider.should_trigger_web_search("how to use rust", false));
        assert!(provider.should_trigger_web_search("what is tauri", true));
        assert!(provider.should_trigger_web_search("what is tauri", false));
    }

    #[tokio::test]
    async fn test_should_trigger_web_search_no_local_results() {
        let provider = WebSearchProvider::new().unwrap();

        // Should trigger for multi-word queries with no local results
        assert!(provider.should_trigger_web_search("search for something", false));
        assert!(provider.should_trigger_web_search("find this file", false));
        assert!(provider.should_trigger_web_search("open application", false));

        // Should NOT trigger for multi-word queries WITH local results
        assert!(!provider.should_trigger_web_search("search for something", true));
        assert!(!provider.should_trigger_web_search("find this file", true));
    }

    #[tokio::test]
    async fn test_should_trigger_web_search_short_queries() {
        let provider = WebSearchProvider::new().unwrap();

        // Should NOT trigger for very short queries
        assert!(!provider.should_trigger_web_search("ab", false));
        assert!(!provider.should_trigger_web_search("a", false));
        assert!(!provider.should_trigger_web_search("", false));

        // Should NOT trigger for short single-word queries even without results
        assert!(!provider.should_trigger_web_search("app", false));
        assert!(!provider.should_trigger_web_search("file", false));
    }

    #[tokio::test]
    async fn test_should_trigger_web_search_single_word() {
        let provider = WebSearchProvider::new().unwrap();

        // Should NOT trigger for single-word queries without question words
        assert!(!provider.should_trigger_web_search("calculator", false));
        assert!(!provider.should_trigger_web_search("notepad", false));
        assert!(!provider.should_trigger_web_search("document", false));
    }

    #[tokio::test]
    async fn test_create_web_search_result() {
        let provider = WebSearchProvider::new().unwrap();

        let result = provider.create_web_search_result("how to code");

        assert_eq!(result.id, "web_search:how to code");
        assert_eq!(result.title, "Search Google for \"how to code\"");
        assert_eq!(result.subtitle, "Press Enter to search on the web");
        assert_eq!(result.result_type, ResultType::WebSearch);
        assert_eq!(result.score, 10.0);

        // Check metadata
        assert!(result.metadata.contains_key("query"));
        assert!(result.metadata.contains_key("search_engine"));

        let query = result.metadata.get("query").unwrap().as_str().unwrap();
        assert_eq!(query, "how to code");

        let engine = result.metadata.get("search_engine").unwrap().as_str().unwrap();
        assert_eq!(engine, "Google");

        // Check action
        match &result.action {
            ResultAction::WebSearch { query } => {
                assert_eq!(query, "how to code");
            }
            _ => panic!("Expected WebSearch action"),
        }
    }

    #[tokio::test]
    async fn test_search_with_question_words() {
        let provider = WebSearchProvider::new().unwrap();

        // Should return web search result for question queries
        let results = provider.search("how to use rust").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].result_type, ResultType::WebSearch);
        assert_eq!(results[0].title, "Search Google for \"how to use rust\"");

        let results = provider.search("what is tauri").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].result_type, ResultType::WebSearch);
    }

    #[tokio::test]
    async fn test_search_without_question_words() {
        let provider = WebSearchProvider::new().unwrap();

        // Should still return web search result as fallback for longer queries
        let results = provider.search("search query").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].result_type, ResultType::WebSearch);

        let results = provider.search("calculator").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].result_type, ResultType::WebSearch);
    }

    #[tokio::test]
    async fn test_search_short_query() {
        let provider = WebSearchProvider::new().unwrap();

        // Should NOT return results for very short queries
        let results = provider.search("ab").await.unwrap();
        assert!(results.is_empty());

        let results = provider.search("a").await.unwrap();
        assert!(results.is_empty());

        let results = provider.search("").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_execute_invalid_result_type() {
        let provider = WebSearchProvider::new().unwrap();

        // Create a result with wrong type
        let invalid_result = SearchResult {
            id: "test".to_string(),
            title: "Test".to_string(),
            subtitle: "Test".to_string(),
            icon: None,
            result_type: ResultType::File, // Wrong type
            score: 100.0,
            metadata: HashMap::new(),
            action: ResultAction::WebSearch {
                query: "test".to_string(),
            },
        };

        let result = provider.execute(&invalid_result).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_provider_initialization() {
        let mut provider = WebSearchProvider::new().unwrap();
        
        let result = provider.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_result_action() {
        let provider = WebSearchProvider::new().unwrap();

        let results = provider.search("how to code").await.unwrap();
        assert_eq!(results.len(), 1);

        let result = &results[0];
        
        // Check action is WebSearch
        match &result.action {
            ResultAction::WebSearch { query } => {
                assert_eq!(query, "how to code");
            }
            _ => panic!("Expected WebSearch action"),
        }
    }

    #[tokio::test]
    async fn test_query_classification() {
        let provider = WebSearchProvider::new().unwrap();

        // Question queries - should always trigger
        assert!(provider.should_trigger_web_search("how to use keyboard", false));
        assert!(provider.should_trigger_web_search("what is rust", false));
        assert!(provider.should_trigger_web_search("why is sky blue", false));

        // Multi-word queries without results - should trigger
        assert!(provider.should_trigger_web_search("search for files", false));
        assert!(provider.should_trigger_web_search("open my document", false));

        // Multi-word queries with results - should NOT trigger
        assert!(!provider.should_trigger_web_search("search for files", true));
        assert!(!provider.should_trigger_web_search("open my document", true));

        // Short queries - should NOT trigger
        assert!(!provider.should_trigger_web_search("ab", false));
        assert!(!provider.should_trigger_web_search("app", false));

        // Single word queries - should NOT trigger
        assert!(!provider.should_trigger_web_search("calculator", false));
        assert!(!provider.should_trigger_web_search("notepad", false));
    }

    #[test]
    fn test_url_construction() {
        // Test basic query encoding
        let url = WebSearchProvider::construct_search_url("hello world");
        assert_eq!(url, "https://www.google.com/search?q=hello%20world");

        // Test special characters encoding
        let url = WebSearchProvider::construct_search_url("rust & tauri");
        assert_eq!(url, "https://www.google.com/search?q=rust%20%26%20tauri");

        // Test URL-unsafe characters
        let url = WebSearchProvider::construct_search_url("what is c++?");
        assert_eq!(url, "https://www.google.com/search?q=what%20is%20c%2B%2B%3F");

        // Test query with equals sign
        let url = WebSearchProvider::construct_search_url("2+2=4");
        assert_eq!(url, "https://www.google.com/search?q=2%2B2%3D4");

        // Test query with forward slash
        let url = WebSearchProvider::construct_search_url("path/to/file");
        assert_eq!(url, "https://www.google.com/search?q=path%2Fto%2Ffile");

        // Test query with hash
        let url = WebSearchProvider::construct_search_url("c# programming");
        assert_eq!(url, "https://www.google.com/search?q=c%23%20programming");

        // Test query with percent sign
        let url = WebSearchProvider::construct_search_url("100% complete");
        assert_eq!(url, "https://www.google.com/search?q=100%25%20complete");

        // Test empty query
        let url = WebSearchProvider::construct_search_url("");
        assert_eq!(url, "https://www.google.com/search?q=");

        // Test unicode characters
        let url = WebSearchProvider::construct_search_url("hello 世界");
        assert!(url.starts_with("https://www.google.com/search?q=hello%20"));
        assert!(url.contains("%E4%B8%96%E7%95%8C")); // UTF-8 encoded 世界
    }

    #[test]
    #[cfg(windows)]
    fn test_browser_detection() {
        // Test browser detection (may return None if registry key doesn't exist)
        let result = WebSearchProvider::get_default_browser();
        
        // Should not error
        assert!(result.is_ok());
        
        // If a browser is detected, it should be a non-empty string
        if let Ok(Some(browser)) = result {
            assert!(!browser.is_empty());
            // Common browser ProgIds
            let valid_browsers = [
                "ChromeHTML",
                "MSEdgeHTM",
                "FirefoxURL",
                "BraveHTML",
                "OperaStable",
            ];
            
            // Check if it's one of the known browsers or at least contains "HTML" or "URL"
            let is_valid = valid_browsers.iter().any(|&b| browser.contains(b))
                || browser.contains("HTML")
                || browser.contains("URL");
            
            if !is_valid {
                // Log the detected browser for debugging
                println!("Detected browser ProgId: {}", browser);
            }
        }
    }

    #[test]
    #[cfg(not(windows))]
    fn test_browser_detection_non_windows() {
        // On non-Windows platforms, should return None
        let result = WebSearchProvider::get_default_browser();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
