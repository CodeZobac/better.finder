use crate::error::{LauncherError, Result};
use crate::search::{ResultCache, SearchProvider};
use crate::types::{ResultAction, ResultType, SearchResult};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Maximum number of results to return per provider
const MAX_RESULTS_PER_PROVIDER: usize = 20;

/// Maximum total results to return
const MAX_TOTAL_RESULTS: usize = 50;

/// Cache capacity (number of queries to cache)
const CACHE_CAPACITY: usize = 100;

/// Cache TTL in seconds
const CACHE_TTL_SECONDS: u64 = 5;

/// SearchEngine coordinates search across multiple providers
pub struct SearchEngine {
    providers: Arc<RwLock<Vec<Box<dyn SearchProvider>>>>,
    /// Optional callback for tracking file access
    file_access_tracker: Arc<RwLock<Option<Box<dyn Fn(&str) + Send + Sync>>>>,
    /// LRU cache for search results
    cache: ResultCache,
}

impl SearchEngine {
    /// Creates a new SearchEngine instance
    pub fn new() -> Self {
        info!("Initializing SearchEngine with result cache");
        Self {
            providers: Arc::new(RwLock::new(Vec::new())),
            file_access_tracker: Arc::new(RwLock::new(None)),
            cache: ResultCache::new(CACHE_CAPACITY, CACHE_TTL_SECONDS),
        }
    }

    /// Sets a callback for tracking file access
    pub async fn set_file_access_tracker<F>(&self, tracker: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        let mut file_tracker = self.file_access_tracker.write().await;
        *file_tracker = Some(Box::new(tracker));
        info!("File access tracker registered");
    }

    /// Registers a new search provider
    pub async fn register_provider(&self, provider: Box<dyn SearchProvider>) {
        let name = provider.name().to_string();
        let priority = provider.priority();
        
        let mut providers = self.providers.write().await;
        providers.push(provider);
        
        // Sort providers by priority (highest first)
        providers.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        // Invalidate cache when providers change
        self.cache.invalidate_all().await;
        
        info!("Registered provider '{}' with priority {}", name, priority);
    }

    /// Performs a search across all enabled providers in parallel
    pub async fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            debug!("Empty query, returning no results");
            return Vec::new();
        }

        let sanitized_query = Self::sanitize_query(query);
        debug!("Searching for: '{}'", sanitized_query);

        // Check cache first
        if let Some(cached_results) = self.cache.get(&sanitized_query).await {
            info!("Returning {} cached results for query: '{}'", cached_results.len(), sanitized_query);
            return cached_results;
        }

        let providers = self.providers.read().await;
        
        // Collect search futures from all enabled providers
        let mut search_futures = Vec::new();
        
        for provider in providers.iter() {
            if !provider.is_enabled() {
                debug!("Skipping disabled provider: {}", provider.name());
                continue;
            }

            let provider_name = provider.name().to_string();
            let query_clone = sanitized_query.clone();
            
            // Execute search and collect the future
            let search_future = async move {
                match provider.search(&query_clone).await {
                    Ok(mut results) => {
                        // Limit results per provider
                        results.truncate(MAX_RESULTS_PER_PROVIDER);
                        debug!(
                            "Provider '{}' returned {} results",
                            provider_name,
                            results.len()
                        );
                        Ok((provider_name, results))
                    }
                    Err(e) => {
                        error!("Provider '{}' search failed: {}", provider_name, e);
                        Err((provider_name, e))
                    }
                }
            };
            
            search_futures.push(search_future);
        }

        // Wait for all search futures to complete
        let task_results = futures::future::join_all(search_futures).await;

        // Collect and merge results
        let mut all_results = Vec::new();
        
        for task_result in task_results {
            match task_result {
                Ok((provider_name, results)) => {
                    debug!("Successfully collected {} results from '{}'", results.len(), provider_name);
                    all_results.extend(results);
                }
                Err((provider_name, error)) => {
                    warn!("Provider '{}' failed with error: {}", provider_name, error);
                    // Continue with other providers (graceful degradation)
                }
            }
        }

        // Rank and sort results
        let ranked_results = Self::rank_results(all_results, &sanitized_query);
        
        // Limit total results
        let final_results: Vec<SearchResult> = ranked_results
            .into_iter()
            .take(MAX_TOTAL_RESULTS)
            .collect();

        info!("Search completed: {} total results", final_results.len());
        
        // Cache the results
        self.cache.put(sanitized_query, final_results.clone()).await;
        
        final_results
    }

    /// Executes the action associated with a search result
    pub async fn execute_result(&self, result: &SearchResult) -> Result<()> {
        info!("Executing result: {} (type: {:?})", result.title, result.result_type);

        // Find the provider that can handle this result type
        let providers = self.providers.read().await;
        
        for provider in providers.iter() {
            if !provider.is_enabled() {
                continue;
            }

            // Try to execute with this provider
            match provider.execute(result).await {
                Ok(()) => {
                    info!("Result executed successfully by provider '{}'", provider.name());
                    
                    // Track file access if this is a file result
                    self.track_file_access_if_needed(result).await;
                    
                    return Ok(());
                }
                Err(e) => {
                    debug!("Provider '{}' could not execute result: {}", provider.name(), e);
                    // Try next provider
                }
            }
        }

        // If no provider could execute, try default execution based on action type
        let execution_result = Self::execute_default_action(&result.action).await;
        
        // Track file access if execution was successful
        if execution_result.is_ok() {
            self.track_file_access_if_needed(result).await;
        }
        
        execution_result
    }

    /// Tracks file access in RecentFilesProvider if the result is a file
    async fn track_file_access_if_needed(&self, result: &SearchResult) {
        // Only track file results
        if result.result_type != ResultType::File {
            return;
        }

        // Extract file path from the result
        let file_path = match &result.action {
            ResultAction::OpenFile { path } => Some(path.as_str()),
            _ => result.metadata.get("path").and_then(|v| v.as_str()),
        };

        if let Some(path_str) = file_path {
            // Call the file access tracker if registered
            let tracker = self.file_access_tracker.read().await;
            if let Some(track_fn) = tracker.as_ref() {
                debug!("Tracking file access for: {}", path_str);
                track_fn(path_str);
            }
        }
    }

    /// Sanitizes user query to prevent issues
    pub fn sanitize_query(query: &str) -> String {
        query
            .trim()
            .chars()
            .filter(|c| !c.is_control())
            .take(256) // Limit query length
            .collect()
    }

    /// Ranks and sorts results by relevance
    pub fn rank_results(mut results: Vec<SearchResult>, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        
        // Boost scores based on various factors
        for result in &mut results {
            let title_lower = result.title.to_lowercase();
            
            // Exact match bonus
            if title_lower == query_lower {
                result.score += 100.0;
            }
            
            // Starts with query bonus
            if title_lower.starts_with(&query_lower) {
                result.score += 50.0;
            }
            
            // Contains query bonus
            if title_lower.contains(&query_lower) {
                result.score += 25.0;
            }
        }

        // Sort by score (highest first)
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Default action execution when no provider handles it
    async fn execute_default_action(action: &ResultAction) -> Result<()> {
        match action {
            ResultAction::OpenFile { path } => {
                info!("Opening file: {}", path);
                #[cfg(target_os = "windows")]
                {
                    std::process::Command::new("cmd")
                        .args(["/C", "start", "", path])
                        .spawn()
                        .map_err(|e| LauncherError::ExecutionError(format!("Failed to open file: {}", e)))?;
                    Ok(())
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err(LauncherError::ExecutionError(
                        "File opening not implemented for this platform".to_string()
                    ))
                }
            }
            ResultAction::LaunchApp { path } => {
                info!("Launching application: {}", path);
                #[cfg(target_os = "windows")]
                {
                    std::process::Command::new(path)
                        .spawn()
                        .map_err(|e| LauncherError::ExecutionError(format!("Failed to launch app: {}", e)))?;
                    Ok(())
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err(LauncherError::ExecutionError(
                        "App launching not implemented for this platform".to_string()
                    ))
                }
            }
            ResultAction::ExecuteCommand { command, args } => {
                info!("Executing command: {} {:?}", command, args);
                std::process::Command::new(command)
                    .args(args)
                    .spawn()
                    .map_err(|e| LauncherError::ExecutionError(format!("Failed to execute command: {}", e)))?;
                Ok(())
            }
            ResultAction::CopyToClipboard { content } => {
                info!("Copying to clipboard: {} chars", content.len());
                // Clipboard functionality will be implemented in ClipboardProvider
                // For now, just log
                warn!("Clipboard copy not yet implemented");
                Ok(())
            }
            ResultAction::OpenUrl { url } => {
                info!("Opening URL: {}", url);
                #[cfg(target_os = "windows")]
                {
                    std::process::Command::new("cmd")
                        .args(["/C", "start", "", url])
                        .spawn()
                        .map_err(|e| LauncherError::ExecutionError(format!("Failed to open URL: {}", e)))?;
                    Ok(())
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err(LauncherError::ExecutionError(
                        "URL opening not implemented for this platform".to_string()
                    ))
                }
            }
            ResultAction::WebSearch { query } => {
                info!("Performing web search: {}", query);
                
                #[cfg(target_os = "windows")]
                {
                    let encoded_query = urlencoding::encode(query);
                    let search_url = format!("https://www.google.com/search?q={}", encoded_query);
                    std::process::Command::new("cmd")
                        .args(["/C", "start", "", &search_url])
                        .spawn()
                        .map_err(|e| LauncherError::ExecutionError(format!("Failed to open web search: {}", e)))?;
                    Ok(())
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = query; // Suppress unused warning
                    Err(LauncherError::ExecutionError(
                        "Web search not implemented for this platform".to_string()
                    ))
                }
            }
        }
    }

    /// Returns the number of registered providers
    pub async fn provider_count(&self) -> usize {
        self.providers.read().await.len()
    }

    /// Returns the names of all registered providers
    pub async fn provider_names(&self) -> Vec<String> {
        self.providers
            .read()
            .await
            .iter()
            .map(|p| p.name().to_string())
            .collect()
    }

    /// Invalidates the search result cache
    pub async fn invalidate_cache(&self) {
        self.cache.invalidate_all().await;
        info!("Search cache invalidated");
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
