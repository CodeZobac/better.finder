/// Windows Search fallback provider
///
/// This provider uses Windows Search API as a fallback when Everything SDK is not available.
/// It provides basic file search functionality using the built-in Windows indexing service.

use crate::error::{LauncherError, Result};
use crate::search::SearchProvider;
use crate::types::{ResultAction, ResultType, SearchResult};
use crate::utils::IconCache;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

const MAX_RESULTS: usize = 20;

/// Windows Search fallback provider
pub struct WindowsSearchProvider {
    icon_cache: Arc<IconCache>,
    enabled: bool,
}

impl WindowsSearchProvider {
    /// Creates a new WindowsSearchProvider
    pub fn new() -> Result<Self> {
        info!("Initializing WindowsSearchProvider as fallback");
        
        Ok(Self {
            icon_cache: Arc::new(IconCache::new()),
            enabled: true,
        })
    }

    /// Search files using Windows Search API
    #[cfg(windows)]
    fn search_windows(&self, query: &str) -> Result<Vec<SearchResult>> {
        use std::process::Command;
        use tracing::{debug, warn};
        
        // Use PowerShell to query Windows Search
        // This is a simplified implementation - a full implementation would use COM APIs
        let ps_script = format!(
            r#"Get-ChildItem -Path "$env:USERPROFILE" -Recurse -Filter "*{}*" -ErrorAction SilentlyContinue | Select-Object -First {} | ForEach-Object {{ $_.FullName }}"#,
            query.replace("\"", "\\\""),
            MAX_RESULTS
        );
        
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut results = Vec::new();
                
                for (idx, line) in stdout.lines().enumerate() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    
                    let path = Path::new(line);
                    if !path.exists() {
                        continue;
                    }
                    
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    
                    let parent_path = path
                        .parent()
                        .and_then(|p| p.to_str())
                        .unwrap_or("")
                        .to_string();
                    
                    let icon = Some(IconCache::get_generic_icon(path));
                    
                    let mut metadata = HashMap::new();
                    metadata.insert("path".to_string(), serde_json::json!(line));
                    
                    // Calculate score based on position (earlier results are more relevant)
                    let score = 50.0 - (idx as f64 * 2.0);
                    
                    results.push(SearchResult {
                        id: format!("windows_search:{}", line),
                        title: file_name,
                        subtitle: parent_path,
                        icon,
                        result_type: ResultType::File,
                        score,
                        metadata,
                        action: ResultAction::OpenFile {
                            path: line.to_string(),
                        },
                    });
                }
                
                debug!("Windows Search found {} results", results.len());
                Ok(results)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Windows Search failed: {}", stderr);
                Ok(Vec::new())
            }
            Err(e) => {
                warn!("Failed to execute Windows Search: {}", e);
                Ok(Vec::new())
            }
        }
    }

    #[cfg(not(windows))]
    fn search_windows(&self, _query: &str) -> Result<Vec<SearchResult>> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl SearchProvider for WindowsSearchProvider {
    fn name(&self) -> &str {
        "WindowsSearch"
    }

    fn priority(&self) -> u8 {
        85 // Slightly lower priority than Everything
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        self.search_windows(query)
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

                let file_path = Path::new(path);
                if !file_path.exists() {
                    return Err(LauncherError::NotFound(format!(
                        "File does not exist: {}",
                        path
                    )));
                }

                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;

                    std::process::Command::new("cmd")
                        .args(["/C", "start", "", path])
                        .creation_flags(CREATE_NO_WINDOW)
                        .spawn()
                        .map_err(|e| {
                            LauncherError::ExecutionError(format!("Failed to open file: {}", e))
                        })?;

                    Ok(())
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
        self.enabled
    }
}

impl Default for WindowsSearchProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            icon_cache: Arc::new(IconCache::new()),
            enabled: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_windows_search_provider_creation() {
        let provider = WindowsSearchProvider::new();
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "WindowsSearch");
        assert_eq!(provider.priority(), 85);
        assert!(provider.is_enabled());
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_windows_search() {
        if let Ok(provider) = WindowsSearchProvider::new() {
            let results = provider.search("test").await;
            match results {
                Ok(files) => {
                    println!("Found {} files with Windows Search", files.len());
                    for file in files.iter().take(3) {
                        println!("  - {}: {}", file.title, file.subtitle);
                    }
                }
                Err(e) => {
                    println!("Search failed: {}", e);
                }
            }
        }
    }
}
