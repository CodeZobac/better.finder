/// Clipboard history provider for tracking and searching clipboard content
///
/// This provider monitors the system clipboard and maintains a history of
/// the last 20 clipboard items, allowing users to search and restore
/// previously copied content.

use crate::error::{LauncherError, Result};
use crate::search::SearchProvider;
use crate::types::{ResultAction, ResultType, SearchResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Maximum number of clipboard items to store
const MAX_CLIPBOARD_ITEMS: usize = 20;

/// Maximum preview length for clipboard content
const MAX_PREVIEW_LENGTH: usize = 100;

/// Represents a single clipboard item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// Unique identifier for the clipboard item
    pub id: String,
    /// The clipboard content (text only for now)
    pub content: String,
    /// When this item was copied
    pub timestamp: DateTime<Utc>,
    /// Type of clipboard content
    pub content_type: ClipboardContentType,
}

/// Types of clipboard content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardContentType {
    Text,
    // Future: Image, File, etc.
}

impl ClipboardItem {
    /// Creates a new clipboard item
    pub fn new(content: String) -> Self {
        let timestamp = Utc::now();
        let id = format!("clipboard:{}", timestamp.timestamp_millis());
        
        Self {
            id,
            content,
            timestamp,
            content_type: ClipboardContentType::Text,
        }
    }

    /// Returns a preview of the clipboard content
    pub fn preview(&self) -> String {
        let content = self.content.trim();
        
        if content.len() <= MAX_PREVIEW_LENGTH {
            content.to_string()
        } else {
            format!("{}...", &content[..MAX_PREVIEW_LENGTH])
        }
    }

    /// Returns a formatted timestamp
    pub fn formatted_timestamp(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.timestamp);

        if duration.num_seconds() < 60 {
            "Just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{} min ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{} hours ago", duration.num_hours())
        } else {
            format!("{} days ago", duration.num_days())
        }
    }
}

/// Clipboard monitor that watches for clipboard changes
pub struct ClipboardMonitor {
    /// Last known clipboard content
    last_content: Arc<RwLock<Option<String>>>,
    /// Whether the monitor is running
    is_running: Arc<RwLock<bool>>,
}

impl ClipboardMonitor {
    /// Creates a new clipboard monitor
    pub fn new() -> Self {
        Self {
            last_content: Arc::new(RwLock::new(None)),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Starts monitoring the clipboard
    pub async fn start<F>(&self, on_change: F) -> Result<()>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            warn!("Clipboard monitor is already running");
            return Ok(());
        }

        *is_running = true;
        drop(is_running);

        info!("Starting clipboard monitor");

        let last_content = Arc::clone(&self.last_content);
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            while *is_running.read().await {
                // Check clipboard content
                match Self::get_clipboard_text().await {
                    Ok(Some(content)) => {
                        let mut last = last_content.write().await;
                        
                        // Only trigger callback if content changed
                        if last.as_ref() != Some(&content) {
                            debug!("Clipboard content changed");
                            *last = Some(content.clone());
                            drop(last);
                            
                            on_change(content);
                        }
                    }
                    Ok(None) => {
                        // Clipboard is empty or contains non-text data
                    }
                    Err(e) => {
                        error!("Failed to read clipboard: {}", e);
                    }
                }

                // Poll every 500ms
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            info!("Clipboard monitor stopped");
        });

        Ok(())
    }

    /// Stops monitoring the clipboard
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Stopping clipboard monitor");
    }

    /// Gets the current clipboard text content
    #[cfg(windows)]
    async fn get_clipboard_text() -> Result<Option<String>> {
        use windows::Win32::Foundation::*;
        use windows::Win32::System::DataExchange::*;
        use windows::Win32::System::Memory::*;

        tokio::task::spawn_blocking(|| {
            unsafe {
                // Open the clipboard
                if OpenClipboard(HWND(std::ptr::null_mut())).is_err() {
                    return Err(LauncherError::ExecutionError(
                        "Failed to open clipboard".to_string(),
                    ));
                }

                // Check if clipboard contains text
                const CF_UNICODETEXT: u32 = 13;
                if IsClipboardFormatAvailable(CF_UNICODETEXT).is_err() {
                    CloseClipboard().ok();
                    return Ok(None);
                }

                // Get clipboard data
                let handle = GetClipboardData(CF_UNICODETEXT);
                if handle.is_err() {
                    CloseClipboard().ok();
                    return Err(LauncherError::ExecutionError(
                        "Failed to get clipboard data".to_string(),
                    ));
                }

                let handle = handle.unwrap();
                if handle.0.is_null() {
                    CloseClipboard().ok();
                    return Ok(None);
                }

                // Lock the memory
                let ptr = GlobalLock(HGLOBAL(handle.0));
                if ptr.is_null() {
                    CloseClipboard().ok();
                    return Err(LauncherError::ExecutionError(
                        "Failed to lock clipboard memory".to_string(),
                    ));
                }

                // Read the text
                let wide_ptr = ptr as *const u16;
                let mut len = 0;
                while *wide_ptr.add(len) != 0 {
                    len += 1;
                }

                let wide_slice = std::slice::from_raw_parts(wide_ptr, len);
                let text = String::from_utf16_lossy(wide_slice);

                GlobalUnlock(HGLOBAL(handle.0)).ok();
                CloseClipboard().ok();

                Ok(Some(text))
            }
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn clipboard task: {}", e))
        })?
    }

    #[cfg(not(windows))]
    async fn get_clipboard_text() -> Result<Option<String>> {
        Err(LauncherError::ExecutionError(
            "Clipboard operations not supported on this platform".to_string(),
        ))
    }
}

impl Default for ClipboardMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage for clipboard history with encryption
pub struct ClipboardStorage {
    /// Path to the storage file
    storage_path: PathBuf,
}

impl ClipboardStorage {
    /// Creates a new clipboard storage
    pub fn new() -> Result<Self> {
        let storage_path = Self::get_storage_path()?;
        
        // Ensure the directory exists
        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self { storage_path })
    }

    /// Gets the storage file path
    fn get_storage_path() -> Result<PathBuf> {
        #[cfg(test)]
        {
            // Use temp directory for tests
            let mut path = std::env::temp_dir();
            path.push("BetterFinder");
            path.push("clipboard_history_test.json");
            return Ok(path);
        }
        
        #[cfg(not(test))]
        {
            let app_data = std::env::var("APPDATA")
                .map_err(|_| LauncherError::ConfigError("APPDATA not found".to_string()))?;
            
            let mut path = PathBuf::from(app_data);
            path.push("BetterFinder");
            path.push("clipboard_history.json");
            
            Ok(path)
        }
    }

    /// Loads clipboard history from disk
    pub async fn load(&self) -> Result<VecDeque<ClipboardItem>> {
        let path = self.storage_path.clone();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Ok(VecDeque::new());
            }

            let content = std::fs::read_to_string(&path)?;
            
            // For now, store as plain JSON
            // TODO: Add encryption in future
            let items: Vec<ClipboardItem> = serde_json::from_str(&content)?;
            
            Ok(items.into_iter().collect())
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn load task: {}", e))
        })?
    }

    /// Saves clipboard history to disk
    pub async fn save(&self, items: &VecDeque<ClipboardItem>) -> Result<()> {
        let path = self.storage_path.clone();
        let items_vec: Vec<ClipboardItem> = items.iter().cloned().collect();
        
        tokio::task::spawn_blocking(move || {
            // For now, store as plain JSON
            // TODO: Add encryption in future
            let content = serde_json::to_string_pretty(&items_vec)?;
            std::fs::write(&path, content)?;
            
            Ok(())
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn save task: {}", e))
        })?
    }
}

impl Default for ClipboardStorage {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            storage_path: PathBuf::from("clipboard_history.json"),
        })
    }
}

/// Clipboard history search provider
pub struct ClipboardHistoryProvider {
    /// Clipboard history storage
    history: Arc<RwLock<VecDeque<ClipboardItem>>>,
    /// Maximum number of items to store
    max_items: usize,
    /// Storage backend
    storage: ClipboardStorage,
    /// Clipboard monitor
    monitor: Arc<ClipboardMonitor>,
    /// Whether the provider is enabled
    enabled: bool,
}

impl ClipboardHistoryProvider {
    /// Creates a new clipboard history provider
    pub fn new() -> Result<Self> {
        info!("Initializing ClipboardHistoryProvider");

        let storage = ClipboardStorage::new()?;
        let monitor = Arc::new(ClipboardMonitor::new());

        Ok(Self {
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_items: MAX_CLIPBOARD_ITEMS,
            storage,
            monitor,
            enabled: true,
        })
    }

    /// Adds a new clipboard item to history
    async fn add_item(&self, content: String) {
        let mut history = self.history.write().await;
        
        // Don't add if it's the same as the most recent item
        if let Some(last) = history.front() {
            if last.content == content {
                return;
            }
        }

        // Don't add empty content
        if content.trim().is_empty() {
            return;
        }

        let item = ClipboardItem::new(content);
        debug!("Adding clipboard item: {}", item.id);
        
        // Add to front of queue
        history.push_front(item);
        
        // Remove oldest items if we exceed max
        while history.len() > self.max_items {
            history.pop_back();
        }

        // Save to disk
        if let Err(e) = self.storage.save(&history).await {
            error!("Failed to save clipboard history: {}", e);
        }
    }

    /// Searches clipboard history
    async fn search_history(&self, query: &str) -> Vec<SearchResult> {
        let history = self.history.read().await;
        let query_lower = query.to_lowercase();
        
        let mut results = Vec::new();
        
        for (index, item) in history.iter().enumerate() {
            // Search in content
            if item.content.to_lowercase().contains(&query_lower) {
                let score = 80.0 - (index as f64 * 2.0); // Newer items score higher
                results.push(self.create_search_result(item, score));
            }
        }

        results
    }

    /// Returns recent clipboard items (when query is empty or starts with "clip:")
    async fn get_recent_items(&self, limit: usize) -> Vec<SearchResult> {
        let history = self.history.read().await;
        
        history
            .iter()
            .take(limit)
            .enumerate()
            .map(|(index, item)| {
                let score = 70.0 - (index as f64 * 2.0);
                self.create_search_result(item, score)
            })
            .collect()
    }

    /// Creates a search result from a clipboard item
    fn create_search_result(&self, item: &ClipboardItem, score: f64) -> SearchResult {
        let preview = item.preview();
        let timestamp = item.formatted_timestamp();
        
        let mut metadata = HashMap::new();
        metadata.insert("content".to_string(), serde_json::json!(item.content));
        metadata.insert("timestamp".to_string(), serde_json::json!(item.timestamp));
        metadata.insert("content_type".to_string(), serde_json::json!(item.content_type));

        SearchResult {
            id: item.id.clone(),
            title: preview.clone(),
            subtitle: format!("Copied {}", timestamp),
            icon: Some("clipboard".to_string()),
            result_type: ResultType::Clipboard,
            score,
            metadata,
            action: ResultAction::CopyToClipboard {
                content: item.content.clone(),
            },
        }
    }

    /// Copies text to the Windows clipboard
    #[cfg(windows)]
    async fn copy_to_clipboard(text: &str) -> Result<()> {
        use windows::Win32::Foundation::*;
        use windows::Win32::System::DataExchange::*;
        use windows::Win32::System::Memory::*;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let text_owned = text.to_string();

        tokio::task::spawn_blocking(move || {
            unsafe {
                // Open the clipboard
                if OpenClipboard(HWND(std::ptr::null_mut())).is_err() {
                    return Err(LauncherError::ExecutionError(
                        "Failed to open clipboard".to_string(),
                    ));
                }

                // Empty the clipboard
                if EmptyClipboard().is_err() {
                    CloseClipboard().ok();
                    return Err(LauncherError::ExecutionError(
                        "Failed to empty clipboard".to_string(),
                    ));
                }

                // Convert text to wide string
                let wide: Vec<u16> = OsStr::new(&text_owned)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                // Allocate global memory
                let len = wide.len() * std::mem::size_of::<u16>();
                let hmem = GlobalAlloc(GMEM_MOVEABLE, len)
                    .map_err(|_| LauncherError::ExecutionError("Failed to allocate memory".to_string()))?;

                // Lock the memory and copy the text
                let ptr = GlobalLock(hmem);
                if ptr.is_null() {
                    GlobalFree(hmem).ok();
                    CloseClipboard().ok();
                    return Err(LauncherError::ExecutionError(
                        "Failed to lock memory".to_string(),
                    ));
                }

                std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
                GlobalUnlock(hmem).ok();

                // Set the clipboard data
                const CF_UNICODETEXT: u32 = 13;
                if SetClipboardData(CF_UNICODETEXT, HANDLE(hmem.0)).is_err() {
                    GlobalFree(hmem).ok();
                    CloseClipboard().ok();
                    return Err(LauncherError::ExecutionError(
                        "Failed to set clipboard data".to_string(),
                    ));
                }

                // Close the clipboard
                CloseClipboard().ok();

                Ok(())
            }
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn clipboard task: {}", e))
        })??;

        Ok(())
    }

    #[cfg(not(windows))]
    async fn copy_to_clipboard(_text: &str) -> Result<()> {
        Err(LauncherError::ExecutionError(
            "Clipboard operations not supported on this platform".to_string(),
        ))
    }
}

#[async_trait]
impl SearchProvider for ClipboardHistoryProvider {
    fn name(&self) -> &str {
        "Clipboard History"
    }

    fn priority(&self) -> u8 {
        60 // Medium priority
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let trimmed = query.trim();
        
        // Check if query starts with "clip:" prefix
        if let Some(search_query) = trimmed.strip_prefix("clip:") {
            let search_query = search_query.trim();
            
            if search_query.is_empty() {
                // Show recent items
                Ok(self.get_recent_items(10).await)
            } else {
                // Search in history
                Ok(self.search_history(search_query).await)
            }
        } else {
            // Don't show clipboard results for general queries
            Ok(Vec::new())
        }
    }

    async fn execute(&self, result: &SearchResult) -> Result<()> {
        if result.result_type != ResultType::Clipboard {
            return Err(LauncherError::ExecutionError(
                "Not a clipboard result".to_string(),
            ));
        }

        // Extract the content from metadata
        let content = result
            .metadata
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                LauncherError::ExecutionError("Invalid clipboard result".to_string())
            })?;

        info!("Restoring clipboard item: {}", result.id);

        // Copy to clipboard
        Self::copy_to_clipboard(content).await?;
        
        info!("Successfully restored clipboard item");
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing ClipboardHistoryProvider");
        
        // Load history from disk
        match self.storage.load().await {
            Ok(items) => {
                let mut history = self.history.write().await;
                *history = items;
                info!("Loaded {} clipboard items from storage", history.len());
            }
            Err(e) => {
                warn!("Failed to load clipboard history: {}", e);
            }
        }

        // Start clipboard monitoring
        let history = Arc::clone(&self.history);
        let storage = ClipboardStorage::new()?;
        
        self.monitor.start(move |content| {
            let history = Arc::clone(&history);
            let storage_clone = storage.clone();
            
            tokio::spawn(async move {
                let mut hist = history.write().await;
                
                // Don't add if it's the same as the most recent item
                if let Some(last) = hist.front() {
                    if last.content == content {
                        return;
                    }
                }

                // Don't add empty content
                if content.trim().is_empty() {
                    return;
                }

                let item = ClipboardItem::new(content);
                debug!("Adding clipboard item from monitor: {}", item.id);
                
                hist.push_front(item);
                
                while hist.len() > MAX_CLIPBOARD_ITEMS {
                    hist.pop_back();
                }

                // Save to disk
                if let Err(e) = storage_clone.save(&hist).await {
                    error!("Failed to save clipboard history: {}", e);
                }
            });
        }).await?;

        info!("ClipboardHistoryProvider initialized successfully");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down ClipboardHistoryProvider");
        
        // Stop clipboard monitoring
        self.monitor.stop().await;
        
        // Save history one last time
        let history = self.history.read().await;
        self.storage.save(&history).await?;
        
        info!("ClipboardHistoryProvider shut down successfully");
        Ok(())
    }
}

impl Default for ClipboardHistoryProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_items: MAX_CLIPBOARD_ITEMS,
            storage: ClipboardStorage::default(),
            monitor: Arc::new(ClipboardMonitor::new()),
            enabled: false,
        })
    }
}

// Clone implementation for ClipboardStorage (needed for the monitor callback)
impl Clone for ClipboardStorage {
    fn clone(&self) -> Self {
        Self {
            storage_path: self.storage_path.clone(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_item_creation() {
        let content = "Test clipboard content".to_string();
        let item = ClipboardItem::new(content.clone());

        assert_eq!(item.content, content);
        assert_eq!(item.content_type, ClipboardContentType::Text);
        assert!(item.id.starts_with("clipboard:"));
    }

    #[test]
    fn test_clipboard_item_preview_short() {
        let content = "Short content".to_string();
        let item = ClipboardItem::new(content.clone());
        
        assert_eq!(item.preview(), content);
    }

    #[test]
    fn test_clipboard_item_preview_long() {
        let content = "a".repeat(150);
        let item = ClipboardItem::new(content);
        
        let preview = item.preview();
        assert!(preview.len() <= MAX_PREVIEW_LENGTH + 3); // +3 for "..."
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn test_clipboard_item_formatted_timestamp() {
        let item = ClipboardItem::new("Test".to_string());
        let formatted = item.formatted_timestamp();
        
        // Should be "Just now" for newly created items
        assert_eq!(formatted, "Just now");
    }

    #[tokio::test]
    async fn test_clipboard_monitor_creation() {
        let monitor = ClipboardMonitor::new();
        
        let is_running = monitor.is_running.read().await;
        assert!(!*is_running);
    }

    #[tokio::test]
    async fn test_clipboard_storage_path() {
        let result = ClipboardStorage::get_storage_path();
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("BetterFinder"));
        // In tests, it uses clipboard_history_test.json
        #[cfg(test)]
        assert!(path.to_string_lossy().contains("clipboard_history_test.json"));
        #[cfg(not(test))]
        assert!(path.to_string_lossy().contains("clipboard_history.json"));
    }

    #[tokio::test]
    async fn test_clipboard_storage_save_and_load() {
        // Use a unique test file to avoid conflicts with other tests
        let mut test_path = std::env::temp_dir();
        test_path.push("BetterFinder");
        std::fs::create_dir_all(&test_path).ok();
        test_path.push("clipboard_test_save_load.json");
        
        let storage = ClipboardStorage {
            storage_path: test_path.clone(),
        };
        
        // Cleanup any existing test file first
        let _ = std::fs::remove_file(&test_path);
        
        // Create test items
        let mut items = VecDeque::new();
        items.push_back(ClipboardItem::new("Item 1".to_string()));
        items.push_back(ClipboardItem::new("Item 2".to_string()));
        items.push_back(ClipboardItem::new("Item 3".to_string()));

        // Save
        let save_result = storage.save(&items).await;
        assert!(save_result.is_ok(), "Failed to save: {:?}", save_result.err());

        // Small delay to ensure file is written
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Verify file exists and has content
        assert!(test_path.exists(), "Storage file does not exist");
        let file_size = std::fs::metadata(&test_path).unwrap().len();
        assert!(file_size > 0, "Storage file is empty");

        // Load
        let load_result = storage.load().await;
        assert!(load_result.is_ok(), "Failed to load: {:?}", load_result.err());
        
        let loaded_items = load_result.unwrap();
        assert_eq!(loaded_items.len(), 3);
        assert_eq!(loaded_items[0].content, "Item 1");
        assert_eq!(loaded_items[1].content, "Item 2");
        assert_eq!(loaded_items[2].content, "Item 3");
        
        // Cleanup: remove test file
        let _ = std::fs::remove_file(&test_path);
    }

    #[tokio::test]
    async fn test_clipboard_provider_creation() {
        let provider = ClipboardHistoryProvider::new();
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "Clipboard History");
        assert_eq!(provider.priority(), 60);
        assert!(provider.is_enabled());
        assert_eq!(provider.max_items, MAX_CLIPBOARD_ITEMS);
    }

    #[tokio::test]
    async fn test_clipboard_provider_add_item() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Add an item
        provider.add_item("Test content 1".to_string()).await;
        
        let history = provider.history.read().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "Test content 1");
    }

    #[tokio::test]
    async fn test_clipboard_provider_add_duplicate() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Add same item twice
        provider.add_item("Test content".to_string()).await;
        provider.add_item("Test content".to_string()).await;
        
        let history = provider.history.read().await;
        // Should only have one item (duplicate not added)
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_clipboard_provider_add_empty() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Try to add empty content
        provider.add_item("".to_string()).await;
        provider.add_item("   ".to_string()).await;
        
        let history = provider.history.read().await;
        // Should not add empty items
        assert_eq!(history.len(), 0);
    }

    #[tokio::test]
    async fn test_clipboard_provider_max_items() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Add more than max items
        for i in 0..25 {
            provider.add_item(format!("Item {}", i)).await;
        }
        
        let history = provider.history.read().await;
        // Should only keep MAX_CLIPBOARD_ITEMS
        assert_eq!(history.len(), MAX_CLIPBOARD_ITEMS);
        
        // Most recent item should be at the front
        assert_eq!(history[0].content, "Item 24");
    }

    #[tokio::test]
    async fn test_clipboard_provider_search_with_clip_prefix() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Add some items
        provider.add_item("Hello world".to_string()).await;
        provider.add_item("Test content".to_string()).await;
        provider.add_item("Another item".to_string()).await;
        
        // Search with "clip:" prefix
        let results = provider.search("clip:hello").await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].result_type, ResultType::Clipboard);
        assert!(results[0].title.contains("Hello world"));
    }

    #[tokio::test]
    async fn test_clipboard_provider_search_empty_clip_prefix() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Add some items
        provider.add_item("Item 1".to_string()).await;
        provider.add_item("Item 2".to_string()).await;
        provider.add_item("Item 3".to_string()).await;
        
        // Search with just "clip:" (no query)
        let results = provider.search("clip:").await.unwrap();
        
        // Should return recent items
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_clipboard_provider_search_without_prefix() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Add some items
        provider.add_item("Hello world".to_string()).await;
        
        // Search without "clip:" prefix
        let results = provider.search("hello").await.unwrap();
        
        // Should return empty (clipboard only responds to "clip:" prefix)
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_clipboard_provider_search_case_insensitive() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Add item with mixed case
        provider.add_item("Hello World".to_string()).await;
        
        // Search with lowercase
        let results = provider.search("clip:hello").await.unwrap();
        assert_eq!(results.len(), 1);
        
        // Search with uppercase
        let results = provider.search("clip:WORLD").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_clipboard_provider_search_result_score() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Add items (newer items should have higher scores)
        provider.add_item("Old item".to_string()).await;
        provider.add_item("Newer item".to_string()).await;
        
        let results = provider.search("clip:item").await.unwrap();
        
        assert_eq!(results.len(), 2);
        // Newer item should have higher score
        assert!(results[0].score > results[1].score);
    }

    #[tokio::test]
    async fn test_clipboard_provider_create_search_result() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        let item = ClipboardItem::new("Test content".to_string());
        
        let result = provider.create_search_result(&item, 80.0);
        
        assert_eq!(result.result_type, ResultType::Clipboard);
        assert_eq!(result.score, 80.0);
        assert!(result.title.contains("Test content"));
        assert!(result.subtitle.contains("Copied"));
        assert_eq!(result.icon, Some("clipboard".to_string()));
        
        // Check metadata
        assert!(result.metadata.contains_key("content"));
        assert!(result.metadata.contains_key("timestamp"));
        assert!(result.metadata.contains_key("content_type"));
        
        // Check action
        match &result.action {
            ResultAction::CopyToClipboard { content } => {
                assert_eq!(content, "Test content");
            }
            _ => panic!("Expected CopyToClipboard action"),
        }
    }

    #[tokio::test]
    async fn test_clipboard_provider_execute_invalid_type() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Create a result with wrong type
        let invalid_result = SearchResult {
            id: "test".to_string(),
            title: "Test".to_string(),
            subtitle: "Test".to_string(),
            icon: None,
            result_type: ResultType::File, // Wrong type
            score: 100.0,
            metadata: HashMap::new(),
            action: ResultAction::CopyToClipboard {
                content: "test".to_string(),
            },
        };

        let result = provider.execute(&invalid_result).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clipboard_provider_get_recent_items() {
        let provider = ClipboardHistoryProvider::new().unwrap();
        
        // Add multiple items
        for i in 0..10 {
            provider.add_item(format!("Item {}", i)).await;
        }
        
        // Get recent items with limit
        let results = provider.get_recent_items(5).await;
        
        assert_eq!(results.len(), 5);
        // Most recent should be first
        assert!(results[0].title.contains("Item 9"));
    }
}
