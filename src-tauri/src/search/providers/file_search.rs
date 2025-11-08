/// File search provider using Everything SDK
///
/// This provider searches for files on the system using the Everything SDK
/// for ultra-fast file indexing and search.

use crate::error::{LauncherError, Result};
use crate::search::providers::everything::{EverythingClient, EverythingFile};
use crate::search::SearchProvider;
use crate::types::{ResultAction, ResultType, SearchResult};
use crate::utils::IconCache;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

const MAX_RESULTS: u32 = 20;

/// File search provider
pub struct FileSearchProvider {
    everything_client: Option<EverythingClient>,
    icon_cache: Arc<IconCache>,
}

impl FileSearchProvider {
    /// Creates a new FileSearchProvider
    pub fn new() -> Result<Self> {
        info!("Initializing FileSearchProvider");

        // Try to initialize Everything client
        let everything_client = match EverythingClient::new() {
            Ok(client) => {
                info!("Everything SDK initialized successfully");
                Some(client)
            }
            Err(e) => {
                warn!("Everything SDK not available: {}. File search will be limited.", e);
                None
            }
        };

        Ok(Self {
            everything_client,
            icon_cache: Arc::new(IconCache::new()),
        })
    }

    /// Gets file icon using the centralized icon cache
    async fn get_file_icon(&self, path: &Path) -> Option<String> {
        // Use generic icon based on extension for better performance
        Some(IconCache::get_generic_icon(path))
    }

    /// Converts EverythingFile to SearchResult
    async fn convert_to_search_result(&self, file: EverythingFile, score: f64) -> SearchResult {
        let icon = self.get_file_icon(&file.full_path).await;

        let mut metadata = HashMap::new();
        metadata.insert("size".to_string(), serde_json::json!(file.size));
        metadata.insert("modified".to_string(), serde_json::json!(file.modified));
        metadata.insert("path".to_string(), serde_json::json!(file.path));

        SearchResult {
            id: format!("file:{}", file.full_path.display()),
            title: file.name.clone(),
            subtitle: file.path.clone(),
            icon,
            result_type: ResultType::File,
            score,
            metadata,
            action: ResultAction::OpenFile {
                path: file.full_path.to_string_lossy().to_string(),
            },
        }
    }

    /// Calculates relevance score for a file based on query
    fn calculate_score(file: &EverythingFile, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let name_lower = file.name.to_lowercase();

        let mut score = 50.0; // Base score

        // Exact match
        if name_lower == query_lower {
            score += 100.0;
        }

        // Starts with query
        if name_lower.starts_with(&query_lower) {
            score += 50.0;
        }

        // Contains query
        if name_lower.contains(&query_lower) {
            score += 25.0;
        }

        // Boost recently modified files
        let now = chrono::Utc::now().timestamp();
        let age_days = (now - file.modified) / 86400;
        if age_days < 7 {
            score += 10.0;
        } else if age_days < 30 {
            score += 5.0;
        }

        // Penalize very large files (might be less relevant)
        if file.size > 1_000_000_000 {
            // > 1GB
            score -= 5.0;
        }

        score
    }
}

#[async_trait]
impl SearchProvider for FileSearchProvider {
    fn name(&self) -> &str {
        "FileSearch"
    }

    fn priority(&self) -> u8 {
        90 // High priority for file search
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Check if Everything is available
        let client = match &self.everything_client {
            Some(client) => client,
            None => {
                debug!("Everything SDK not available, skipping file search");
                return Ok(Vec::new());
            }
        };

        debug!("Searching files for query: '{}'", query);

        // Perform search using Everything SDK
        let files = client.search(query, MAX_RESULTS).map_err(|e| {
            error!("File search failed: {}", e);
            LauncherError::SearchError(format!("File search failed: {}", e))
        })?;

        debug!("Found {} files", files.len());

        // Convert to search results
        let mut results = Vec::new();
        for file in files {
            let score = Self::calculate_score(&file, query);
            let result = self.convert_to_search_result(file, score).await;
            results.push(result);
        }

        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    async fn execute(&self, result: &SearchResult) -> Result<()> {
        if result.result_type != ResultType::File {
            return Err(LauncherError::ExecutionError(
                "Not a file result".to_string(),
            ));
        }

        match &result.action {
            ResultAction::OpenFile { path } => {
                info!("Opening file: {}", path);

                // Verify file exists before attempting to open
                let file_path = Path::new(path);
                if !file_path.exists() {
                    error!("File not found: {}", path);
                    return Err(LauncherError::NotFound(format!(
                        "File does not exist: {}",
                        path
                    )));
                }

                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;

                    // Use Windows ShellExecute via cmd to open file with default application
                    let result = std::process::Command::new("cmd")
                        .args(["/C", "start", "", path])
                        .creation_flags(CREATE_NO_WINDOW)
                        .spawn();

                    match result {
                        Ok(_) => {
                            info!("Successfully opened file: {}", path);
                            Ok(())
                        }
                        Err(e) => {
                            error!("Failed to open file '{}': {}", path, e);
                            Err(LauncherError::ExecutionError(format!(
                                "Failed to open file: {}",
                                e
                            )))
                        }
                    }
                }

                #[cfg(not(windows))]
                {
                    Err(LauncherError::ExecutionError(
                        "File opening not implemented for this platform".to_string(),
                    ))
                }
            }
            _ => Err(LauncherError::ExecutionError(
                "Invalid action for file result".to_string(),
            )),
        }
    }

    fn is_enabled(&self) -> bool {
        self.everything_client.is_some()
    }
}

impl Default for FileSearchProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            everything_client: None,
            icon_cache: Arc::new(IconCache::new()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_file_search_provider_creation() {
        let provider = FileSearchProvider::new();
        match provider {
            Ok(p) => {
                assert_eq!(p.name(), "FileSearch");
                assert_eq!(p.priority(), 90);
            }
            Err(e) => {
                println!("Provider creation failed (expected if Everything not installed): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_file_search() {
        if let Ok(provider) = FileSearchProvider::new() {
            if provider.is_enabled() {
                let results = provider.search("*.txt").await;
                match results {
                    Ok(files) => {
                        println!("Found {} files", files.len());
                        for file in files.iter().take(3) {
                            println!("  - {}: {}", file.title, file.subtitle);
                        }
                    }
                    Err(e) => {
                        println!("Search failed: {}", e);
                    }
                }
            } else {
                println!("Provider not enabled - test skipped");
            }
        }
    }

    #[tokio::test]
    async fn test_score_calculation() {
        let file = EverythingFile {
            name: "test.txt".to_string(),
            path: "C:\\Users\\Test".to_string(),
            full_path: PathBuf::from("C:\\Users\\Test\\test.txt"),
            size: 1024,
            modified: chrono::Utc::now().timestamp(),
        };

        let score = FileSearchProvider::calculate_score(&file, "test");
        assert!(score > 50.0, "Score should be greater than base score");

        let exact_score = FileSearchProvider::calculate_score(&file, "test.txt");
        assert!(exact_score > score, "Exact match should have higher score");
    }
}
