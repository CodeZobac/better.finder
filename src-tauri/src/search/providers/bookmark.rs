/// Bookmark provider for searching browser bookmarks
///
/// This provider searches bookmarks from Chrome, Edge, and Firefox browsers,
/// allowing users to quickly access their saved websites.

use crate::error::{LauncherError, Result};
use crate::search::SearchProvider;
use crate::types::{ResultAction, ResultType, SearchResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Maximum number of bookmarks to cache
const MAX_BOOKMARKS: usize = 1000;

/// Cache refresh interval in seconds
const CACHE_REFRESH_INTERVAL: u64 = 300; // 5 minutes

/// Supported browser types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Edge,
    Firefox,
}

impl BrowserType {
    /// Returns the display name of the browser
    pub fn display_name(&self) -> &str {
        match self {
            BrowserType::Chrome => "Chrome",
            BrowserType::Edge => "Edge",
            BrowserType::Firefox => "Firefox",
        }
    }
}

/// Represents a browser bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Bookmark title
    pub title: String,
    /// Bookmark URL
    pub url: String,
    /// Folder/path in bookmark hierarchy
    pub folder: Option<String>,
    /// Browser this bookmark is from
    pub browser: BrowserType,
    /// Base64 encoded favicon (if available)
    pub favicon: Option<String>,
}

impl Bookmark {
    /// Creates a new bookmark
    pub fn new(title: String, url: String, browser: BrowserType) -> Self {
        Self {
            title,
            url,
            folder: None,
            browser,
            favicon: None,
        }
    }

    /// Creates a unique ID for the bookmark
    pub fn id(&self) -> String {
        format!("bookmark:{}:{}", self.browser.display_name(), self.url)
    }

    /// Returns a display subtitle showing the URL and browser
    pub fn subtitle(&self) -> String {
        if let Some(folder) = &self.folder {
            format!("{} • {}", self.url, folder)
        } else {
            self.url.clone()
        }
    }
}

/// Chrome/Edge bookmark structure (JSON format)
#[derive(Debug, Deserialize)]
struct ChromeBookmarkRoot {
    roots: ChromeBookmarkRoots,
}

#[derive(Debug, Deserialize)]
struct ChromeBookmarkRoots {
    bookmark_bar: ChromeBookmarkNode,
    other: ChromeBookmarkNode,
    #[serde(default)]
    synced: Option<ChromeBookmarkNode>,
}

#[derive(Debug, Deserialize)]
struct ChromeBookmarkNode {
    #[serde(default)]
    name: String,
    #[serde(rename = "type")]
    node_type: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    children: Vec<ChromeBookmarkNode>,
}

/// Parser for Chrome/Edge bookmarks
pub struct ChromeBookmarkParser;

impl ChromeBookmarkParser {
    /// Parses Chrome or Edge bookmarks from the Bookmarks file
    pub fn parse(path: &PathBuf, browser: BrowserType) -> Result<Vec<Bookmark>> {
        debug!("Parsing {} bookmarks from: {:?}", browser.display_name(), path);

        if !path.exists() {
            warn!("Bookmark file not found: {:?}", path);
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| LauncherError::SearchError(format!("Failed to read bookmarks: {}", e)))?;

        let root: ChromeBookmarkRoot = serde_json::from_str(&content)
            .map_err(|e| LauncherError::SearchError(format!("Failed to parse bookmarks: {}", e)))?;

        let mut bookmarks = Vec::new();

        // Parse bookmark bar
        Self::parse_node(&root.roots.bookmark_bar, None, browser, &mut bookmarks);

        // Parse other bookmarks
        Self::parse_node(&root.roots.other, None, browser, &mut bookmarks);

        // Parse synced bookmarks if available
        if let Some(synced) = root.roots.synced {
            Self::parse_node(&synced, None, browser, &mut bookmarks);
        }

        info!("Parsed {} bookmarks from {}", bookmarks.len(), browser.display_name());
        Ok(bookmarks)
    }

    /// Recursively parses bookmark nodes
    fn parse_node(
        node: &ChromeBookmarkNode,
        parent_folder: Option<String>,
        browser: BrowserType,
        bookmarks: &mut Vec<Bookmark>,
    ) {
        if node.node_type == "url" {
            if let Some(url) = &node.url {
                let mut bookmark = Bookmark::new(node.name.clone(), url.clone(), browser);
                bookmark.folder = parent_folder;
                bookmarks.push(bookmark);
            }
        } else if node.node_type == "folder" {
            let folder_path = if let Some(parent) = parent_folder {
                format!("{}/{}", parent, node.name)
            } else {
                node.name.clone()
            };

            for child in &node.children {
                Self::parse_node(child, Some(folder_path.clone()), browser, bookmarks);
            }
        }
    }

    /// Locates the Chrome bookmarks file
    pub fn locate_chrome_bookmarks() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let path = PathBuf::from(local_app_data)
                    .join("Google")
                    .join("Chrome")
                    .join("User Data")
                    .join("Default")
                    .join("Bookmarks");

                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    /// Locates the Edge bookmarks file
    pub fn locate_edge_bookmarks() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let path = PathBuf::from(local_app_data)
                    .join("Microsoft")
                    .join("Edge")
                    .join("User Data")
                    .join("Default")
                    .join("Bookmarks");

                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }
}

/// Parser for Firefox bookmarks
pub struct FirefoxBookmarkParser;

impl FirefoxBookmarkParser {
    /// Parses Firefox bookmarks from the places.sqlite database
    pub fn parse(path: &PathBuf) -> Result<Vec<Bookmark>> {
        debug!("Parsing Firefox bookmarks from: {:?}", path);

        if !path.exists() {
            warn!("Firefox places database not found: {:?}", path);
            return Ok(Vec::new());
        }

        let conn = rusqlite::Connection::open(path)
            .map_err(|e| LauncherError::SearchError(format!("Failed to open Firefox database: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT moz_bookmarks.title, moz_places.url, moz_bookmarks.parent
             FROM moz_bookmarks
             INNER JOIN moz_places ON moz_bookmarks.fk = moz_places.id
             WHERE moz_bookmarks.type = 1 AND moz_places.url IS NOT NULL"
        ).map_err(|e| LauncherError::SearchError(format!("Failed to prepare query: {}", e)))?;

        let bookmarks_iter = stmt.query_map([], |row| {
            let title: Option<String> = row.get(0).ok();
            let url: String = row.get(1)?;
            let _parent: Option<i64> = row.get(2).ok();

            Ok((title, url))
        }).map_err(|e| LauncherError::SearchError(format!("Failed to query bookmarks: {}", e)))?;

        let mut bookmarks = Vec::new();

        for bookmark_result in bookmarks_iter {
            if let Ok((title, url)) = bookmark_result {
                // Skip invalid URLs
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    continue;
                }

                let title = title.unwrap_or_else(|| url.clone());
                bookmarks.push(Bookmark::new(title, url, BrowserType::Firefox));
            }
        }

        info!("Parsed {} bookmarks from Firefox", bookmarks.len());
        Ok(bookmarks)
    }

    /// Locates the Firefox places.sqlite file
    pub fn locate_firefox_places() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            if let Ok(app_data) = std::env::var("APPDATA") {
                let firefox_dir = PathBuf::from(app_data)
                    .join("Mozilla")
                    .join("Firefox")
                    .join("Profiles");

                if firefox_dir.exists() {
                    // Find the default profile directory
                    if let Ok(entries) = std::fs::read_dir(&firefox_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                let places_path = path.join("places.sqlite");
                                if places_path.exists() {
                                    return Some(places_path);
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

/// Bookmark search provider
pub struct BookmarkProvider {
    /// Cached bookmarks
    bookmarks: Arc<RwLock<Vec<Bookmark>>>,
    /// Favicon cache (URL -> base64 encoded image)
    favicon_cache: Arc<RwLock<HashMap<String, String>>>,
    /// Whether the provider is enabled
    enabled: bool,
    /// Last cache refresh time
    last_refresh: Arc<RwLock<std::time::Instant>>,
}

impl BookmarkProvider {
    /// Creates a new bookmark provider
    pub fn new() -> Result<Self> {
        info!("Initializing BookmarkProvider");

        Ok(Self {
            bookmarks: Arc::new(RwLock::new(Vec::new())),
            favicon_cache: Arc::new(RwLock::new(HashMap::new())),
            enabled: true,
            last_refresh: Arc::new(RwLock::new(std::time::Instant::now())),
        })
    }

    /// Loads bookmarks from all supported browsers
    async fn load_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let mut all_bookmarks = Vec::new();

        // Load Chrome bookmarks
        if let Some(chrome_path) = ChromeBookmarkParser::locate_chrome_bookmarks() {
            match ChromeBookmarkParser::parse(&chrome_path, BrowserType::Chrome) {
                Ok(bookmarks) => {
                    debug!("Loaded {} Chrome bookmarks", bookmarks.len());
                    all_bookmarks.extend(bookmarks);
                }
                Err(e) => {
                    warn!("Failed to parse Chrome bookmarks: {}", e);
                }
            }
        }

        // Load Edge bookmarks
        if let Some(edge_path) = ChromeBookmarkParser::locate_edge_bookmarks() {
            match ChromeBookmarkParser::parse(&edge_path, BrowserType::Edge) {
                Ok(bookmarks) => {
                    debug!("Loaded {} Edge bookmarks", bookmarks.len());
                    all_bookmarks.extend(bookmarks);
                }
                Err(e) => {
                    warn!("Failed to parse Edge bookmarks: {}", e);
                }
            }
        }

        // Load Firefox bookmarks
        if let Some(firefox_path) = FirefoxBookmarkParser::locate_firefox_places() {
            match FirefoxBookmarkParser::parse(&firefox_path) {
                Ok(bookmarks) => {
                    debug!("Loaded {} Firefox bookmarks", bookmarks.len());
                    all_bookmarks.extend(bookmarks);
                }
                Err(e) => {
                    warn!("Failed to parse Firefox bookmarks: {}", e);
                }
            }
        }

        // Limit to MAX_BOOKMARKS
        if all_bookmarks.len() > MAX_BOOKMARKS {
            all_bookmarks.truncate(MAX_BOOKMARKS);
        }

        info!("Loaded total of {} bookmarks", all_bookmarks.len());
        Ok(all_bookmarks)
    }

    /// Refreshes the bookmark cache
    async fn refresh_cache(&self) -> Result<()> {
        debug!("Refreshing bookmark cache");

        let bookmarks = self.load_bookmarks().await?;
        
        let mut cache = self.bookmarks.write().await;
        *cache = bookmarks;

        let mut last_refresh = self.last_refresh.write().await;
        *last_refresh = std::time::Instant::now();

        info!("Bookmark cache refreshed with {} items", cache.len());
        Ok(())
    }

    /// Checks if cache needs refresh and refreshes if necessary
    async fn check_and_refresh_cache(&self) {
        let last_refresh = self.last_refresh.read().await;
        let elapsed = last_refresh.elapsed().as_secs();

        if elapsed >= CACHE_REFRESH_INTERVAL {
            drop(last_refresh);
            if let Err(e) = self.refresh_cache().await {
                error!("Failed to refresh bookmark cache: {}", e);
            }
        }
    }

    /// Searches bookmarks using fuzzy matching
    async fn search_bookmarks(&self, query: &str) -> Vec<SearchResult> {
        let bookmarks = self.bookmarks.read().await;
        let query_lower = query.to_lowercase();

        let mut results: Vec<(Bookmark, f64)> = bookmarks
            .iter()
            .filter_map(|bookmark| {
                let title_lower = bookmark.title.to_lowercase();
                let url_lower = bookmark.url.to_lowercase();

                // Calculate score based on matches
                let mut score = 0.0;

                // Exact title match
                if title_lower == query_lower {
                    score = 100.0;
                }
                // Title starts with query
                else if title_lower.starts_with(&query_lower) {
                    score = 90.0;
                }
                // Title contains query
                else if title_lower.contains(&query_lower) {
                    score = 70.0;
                }
                // URL contains query
                else if url_lower.contains(&query_lower) {
                    score = 50.0;
                }

                if score > 0.0 {
                    Some((bookmark.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(10);

        // Convert to SearchResults
        let mut search_results = Vec::new();
        for (bookmark, score) in results {
            search_results.push(self.create_search_result(&bookmark, score).await);
        }

        search_results
    }

    /// Creates a search result from a bookmark
    async fn create_search_result(&self, bookmark: &Bookmark, score: f64) -> SearchResult {
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), serde_json::json!(bookmark.url));
        metadata.insert("browser".to_string(), serde_json::json!(bookmark.browser));
        
        if let Some(folder) = &bookmark.folder {
            metadata.insert("folder".to_string(), serde_json::json!(folder));
        }

        // Try to get favicon from cache
        let favicon = {
            let cache = self.favicon_cache.read().await;
            cache.get(&bookmark.url).cloned()
        };

        // If not in cache, download asynchronously (don't block)
        if favicon.is_none() {
            let url = bookmark.url.clone();
            let favicon_cache = Arc::clone(&self.favicon_cache);
            
            tokio::spawn(async move {
                if let Ok(favicon_data) = Self::download_favicon(&url).await {
                    let mut cache = favicon_cache.write().await;
                    cache.insert(url, favicon_data);
                }
            });
        }

        SearchResult {
            id: bookmark.id(),
            title: bookmark.title.clone(),
            subtitle: bookmark.subtitle(),
            icon: favicon.or_else(|| Some("bookmark".to_string())),
            result_type: ResultType::Bookmark,
            score,
            metadata,
            action: ResultAction::OpenUrl {
                url: bookmark.url.clone(),
            },
        }
    }

    /// Downloads a favicon for a URL
    async fn download_favicon(url: &str) -> Result<String> {
        // Extract domain from URL
        let domain = url
            .split("://")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .ok_or_else(|| LauncherError::SearchError("Invalid URL".to_string()))?;

        // Try to download favicon
        let favicon_url = format!("https://{}/favicon.ico", domain);
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| LauncherError::SearchError(format!("Failed to create HTTP client: {}", e)))?;

        let response = client
            .get(&favicon_url)
            .send()
            .await
            .map_err(|e| LauncherError::SearchError(format!("Failed to download favicon: {}", e)))?;

        if !response.status().is_success() {
            return Err(LauncherError::SearchError("Favicon not found".to_string()));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| LauncherError::SearchError(format!("Failed to read favicon: {}", e)))?;

        // Encode as base64
        let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        Ok(format!("data:image/x-icon;base64,{}", base64_data))
    }

    /// Starts the background cache refresh task
    fn start_cache_refresh_task(provider: Arc<RwLock<Self>>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(CACHE_REFRESH_INTERVAL)).await;
                
                let provider_lock = provider.read().await;
                if let Err(e) = provider_lock.refresh_cache().await {
                    error!("Background cache refresh failed: {}", e);
                }
            }
        });
    }
}

#[async_trait]
impl SearchProvider for BookmarkProvider {
    fn name(&self) -> &str {
        "Bookmarks"
    }

    fn priority(&self) -> u8 {
        50 // Medium priority
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let trimmed = query.trim();

        // Don't search if query is too short
        if trimmed.len() < 2 {
            return Ok(Vec::new());
        }

        // Check if cache needs refresh
        self.check_and_refresh_cache().await;

        // Search bookmarks
        Ok(self.search_bookmarks(trimmed).await)
    }

    async fn execute(&self, result: &SearchResult) -> Result<()> {
        if result.result_type != ResultType::Bookmark {
            return Err(LauncherError::ExecutionError(
                "Not a bookmark result".to_string(),
            ));
        }

        // Extract URL from action
        if let ResultAction::OpenUrl { url } = &result.action {
            info!("Opening bookmark: {}", url);
            Self::open_url(url).await?;
            info!("Successfully opened bookmark");
            Ok(())
        } else {
            Err(LauncherError::ExecutionError(
                "Invalid bookmark action".to_string(),
            ))
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing BookmarkProvider");

        // Load bookmarks initially
        if let Err(e) = self.refresh_cache().await {
            warn!("Failed to load initial bookmarks: {}", e);
        }

        info!("BookmarkProvider initialized successfully");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down BookmarkProvider");
        Ok(())
    }
}

impl BookmarkProvider {
    /// Opens a URL in the default browser using Windows API
    #[cfg(windows)]
    async fn open_url(url: &str) -> Result<()> {
        use windows::Win32::Foundation::*;
        use windows::Win32::UI::Shell::*;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        use windows::core::PCWSTR;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let url_owned = url.to_string();

        tokio::task::spawn_blocking(move || {
            unsafe {
                let operation: Vec<u16> = OsStr::new("open")
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                let file: Vec<u16> = OsStr::new(&url_owned)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                let result = ShellExecuteW(
                    HWND(std::ptr::null_mut()),
                    PCWSTR(operation.as_ptr()),
                    PCWSTR(file.as_ptr()),
                    PCWSTR::null(),
                    PCWSTR::null(),
                    SW_SHOWNORMAL,
                );

                if result.0 as isize <= 32 {
                    return Err(LauncherError::ExecutionError(format!(
                        "Failed to open URL: error code {}",
                        result.0 as isize
                    )));
                }

                Ok(())
            }
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn URL open task: {}", e))
        })??;

        Ok(())
    }

    #[cfg(not(windows))]
    async fn open_url(_url: &str) -> Result<()> {
        Err(LauncherError::ExecutionError(
            "URL opening not supported on this platform".to_string(),
        ))
    }
}

impl Default for BookmarkProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            bookmarks: Arc::new(RwLock::new(Vec::new())),
            favicon_cache: Arc::new(RwLock::new(HashMap::new())),
            enabled: false,
            last_refresh: Arc::new(RwLock::new(std::time::Instant::now())),
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_creation() {
        let bookmark = Bookmark::new(
            "Test Bookmark".to_string(),
            "https://example.com".to_string(),
            BrowserType::Chrome,
        );

        assert_eq!(bookmark.title, "Test Bookmark");
        assert_eq!(bookmark.url, "https://example.com");
        assert_eq!(bookmark.browser, BrowserType::Chrome);
        assert!(bookmark.folder.is_none());
        assert!(bookmark.favicon.is_none());
    }

    #[test]
    fn test_bookmark_id() {
        let bookmark = Bookmark::new(
            "Test".to_string(),
            "https://example.com".to_string(),
            BrowserType::Chrome,
        );

        let id = bookmark.id();
        assert!(id.starts_with("bookmark:Chrome:"));
        assert!(id.contains("https://example.com"));
    }

    #[test]
    fn test_bookmark_subtitle_without_folder() {
        let bookmark = Bookmark::new(
            "Test".to_string(),
            "https://example.com".to_string(),
            BrowserType::Chrome,
        );

        assert_eq!(bookmark.subtitle(), "https://example.com");
    }

    #[test]
    fn test_bookmark_subtitle_with_folder() {
        let mut bookmark = Bookmark::new(
            "Test".to_string(),
            "https://example.com".to_string(),
            BrowserType::Chrome,
        );
        bookmark.folder = Some("Work/Projects".to_string());

        assert_eq!(bookmark.subtitle(), "https://example.com • Work/Projects");
    }

    #[test]
    fn test_browser_type_display_name() {
        assert_eq!(BrowserType::Chrome.display_name(), "Chrome");
        assert_eq!(BrowserType::Edge.display_name(), "Edge");
        assert_eq!(BrowserType::Firefox.display_name(), "Firefox");
    }

    #[test]
    fn test_chrome_bookmark_parser_with_valid_json() {
        // Create a temporary Chrome bookmarks file
        let temp_dir = std::env::temp_dir();
        let bookmarks_path = temp_dir.join("test_chrome_bookmarks.json");

        let bookmarks_json = r#"{
            "roots": {
                "bookmark_bar": {
                    "name": "Bookmarks Bar",
                    "type": "folder",
                    "children": [
                        {
                            "name": "Google",
                            "type": "url",
                            "url": "https://www.google.com"
                        },
                        {
                            "name": "Work",
                            "type": "folder",
                            "children": [
                                {
                                    "name": "GitHub",
                                    "type": "url",
                                    "url": "https://github.com"
                                }
                            ]
                        }
                    ]
                },
                "other": {
                    "name": "Other Bookmarks",
                    "type": "folder",
                    "children": [
                        {
                            "name": "Reddit",
                            "type": "url",
                            "url": "https://www.reddit.com"
                        }
                    ]
                }
            }
        }"#;

        std::fs::write(&bookmarks_path, bookmarks_json).unwrap();

        // Parse the bookmarks
        let result = ChromeBookmarkParser::parse(&bookmarks_path, BrowserType::Chrome);
        assert!(result.is_ok());

        let bookmarks = result.unwrap();
        assert_eq!(bookmarks.len(), 3);

        // Check first bookmark
        assert_eq!(bookmarks[0].title, "Google");
        assert_eq!(bookmarks[0].url, "https://www.google.com");
        assert_eq!(bookmarks[0].browser, BrowserType::Chrome);

        // Check nested bookmark
        assert_eq!(bookmarks[1].title, "GitHub");
        assert_eq!(bookmarks[1].url, "https://github.com");
        assert_eq!(bookmarks[1].folder, Some("Bookmarks Bar/Work".to_string()));

        // Check other bookmarks
        assert_eq!(bookmarks[2].title, "Reddit");
        assert_eq!(bookmarks[2].url, "https://www.reddit.com");

        // Cleanup
        std::fs::remove_file(&bookmarks_path).ok();
    }

    #[test]
    fn test_chrome_bookmark_parser_with_nonexistent_file() {
        let path = PathBuf::from("nonexistent_bookmarks.json");
        let result = ChromeBookmarkParser::parse(&path, BrowserType::Chrome);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_firefox_bookmark_parser_with_valid_database() {
        // Create a temporary Firefox places database
        let temp_dir = std::env::temp_dir();
        let places_path = temp_dir.join("test_firefox_places.sqlite");

        // Create a minimal places.sqlite database
        let conn = rusqlite::Connection::open(&places_path).unwrap();
        
        conn.execute(
            "CREATE TABLE moz_places (
                id INTEGER PRIMARY KEY,
                url TEXT
            )",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE moz_bookmarks (
                id INTEGER PRIMARY KEY,
                type INTEGER,
                fk INTEGER,
                parent INTEGER,
                title TEXT
            )",
            [],
        ).unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO moz_places (id, url) VALUES (1, 'https://www.google.com')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO moz_places (id, url) VALUES (2, 'https://github.com')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO moz_bookmarks (id, type, fk, parent, title) VALUES (1, 1, 1, 0, 'Google')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO moz_bookmarks (id, type, fk, parent, title) VALUES (2, 1, 2, 0, 'GitHub')",
            [],
        ).unwrap();

        drop(conn);

        // Parse the bookmarks
        let result = FirefoxBookmarkParser::parse(&places_path);
        assert!(result.is_ok());

        let bookmarks = result.unwrap();
        assert_eq!(bookmarks.len(), 2);

        assert_eq!(bookmarks[0].title, "Google");
        assert_eq!(bookmarks[0].url, "https://www.google.com");
        assert_eq!(bookmarks[0].browser, BrowserType::Firefox);

        assert_eq!(bookmarks[1].title, "GitHub");
        assert_eq!(bookmarks[1].url, "https://github.com");

        // Cleanup
        std::fs::remove_file(&places_path).ok();
    }

    #[test]
    fn test_firefox_bookmark_parser_with_nonexistent_file() {
        let path = PathBuf::from("nonexistent_places.sqlite");
        let result = FirefoxBookmarkParser::parse(&path);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_bookmark_provider_creation() {
        let provider = BookmarkProvider::new();
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "Bookmarks");
        assert_eq!(provider.priority(), 50);
        assert!(provider.is_enabled());
    }

    #[tokio::test]
    async fn test_bookmark_provider_search_empty_query() {
        let provider = BookmarkProvider::new().unwrap();
        
        // Empty query should return no results
        let results = provider.search("").await.unwrap();
        assert_eq!(results.len(), 0);

        // Single character query should return no results
        let results = provider.search("a").await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_bookmark_provider_search_with_bookmarks() {
        let provider = BookmarkProvider::new().unwrap();
        
        // Add some test bookmarks
        let mut bookmarks = Vec::new();
        bookmarks.push(Bookmark::new(
            "Google Search".to_string(),
            "https://www.google.com".to_string(),
            BrowserType::Chrome,
        ));
        bookmarks.push(Bookmark::new(
            "GitHub".to_string(),
            "https://github.com".to_string(),
            BrowserType::Chrome,
        ));
        bookmarks.push(Bookmark::new(
            "Stack Overflow".to_string(),
            "https://stackoverflow.com".to_string(),
            BrowserType::Firefox,
        ));

        {
            let mut cache = provider.bookmarks.write().await;
            *cache = bookmarks;
        }

        // Search for "google"
        let results = provider.search("google").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Google Search");
        assert_eq!(results[0].result_type, ResultType::Bookmark);

        // Search for "git"
        let results = provider.search("git").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "GitHub");

        // Search for "stack"
        let results = provider.search("stack").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Stack Overflow");
    }

    #[tokio::test]
    async fn test_bookmark_provider_search_case_insensitive() {
        let provider = BookmarkProvider::new().unwrap();
        
        let mut bookmarks = Vec::new();
        bookmarks.push(Bookmark::new(
            "GitHub".to_string(),
            "https://github.com".to_string(),
            BrowserType::Chrome,
        ));

        {
            let mut cache = provider.bookmarks.write().await;
            *cache = bookmarks;
        }

        // Search with different cases
        let results = provider.search("github").await.unwrap();
        assert_eq!(results.len(), 1);

        let results = provider.search("GITHUB").await.unwrap();
        assert_eq!(results.len(), 1);

        let results = provider.search("GiTHuB").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_bookmark_provider_search_url_matching() {
        let provider = BookmarkProvider::new().unwrap();
        
        let mut bookmarks = Vec::new();
        bookmarks.push(Bookmark::new(
            "My Site".to_string(),
            "https://example.com/path/to/page".to_string(),
            BrowserType::Chrome,
        ));

        {
            let mut cache = provider.bookmarks.write().await;
            *cache = bookmarks;
        }

        // Search by URL
        let results = provider.search("example.com").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "My Site");
    }

    #[tokio::test]
    async fn test_bookmark_provider_search_scoring() {
        let provider = BookmarkProvider::new().unwrap();
        
        let mut bookmarks = Vec::new();
        bookmarks.push(Bookmark::new(
            "test".to_string(), // Exact match
            "https://test.com".to_string(),
            BrowserType::Chrome,
        ));
        bookmarks.push(Bookmark::new(
            "testing page".to_string(), // Starts with
            "https://example.com".to_string(),
            BrowserType::Chrome,
        ));
        bookmarks.push(Bookmark::new(
            "my test page".to_string(), // Contains
            "https://example.com".to_string(),
            BrowserType::Chrome,
        ));

        {
            let mut cache = provider.bookmarks.write().await;
            *cache = bookmarks;
        }

        let results = provider.search("test").await.unwrap();
        assert_eq!(results.len(), 3);
        
        // Exact match should score highest
        assert_eq!(results[0].title, "test");
        assert_eq!(results[0].score, 100.0);
        
        // Starts with should score second
        assert_eq!(results[1].title, "testing page");
        assert_eq!(results[1].score, 90.0);
        
        // Contains should score third
        assert_eq!(results[2].title, "my test page");
        assert_eq!(results[2].score, 70.0);
    }

    #[tokio::test]
    async fn test_bookmark_provider_execute_invalid_result() {
        let provider = BookmarkProvider::new().unwrap();
        
        let result = SearchResult {
            id: "test".to_string(),
            title: "Test".to_string(),
            subtitle: "Test".to_string(),
            icon: None,
            result_type: ResultType::File, // Wrong type
            score: 100.0,
            metadata: HashMap::new(),
            action: ResultAction::OpenUrl {
                url: "https://example.com".to_string(),
            },
        };

        let execute_result = provider.execute(&result).await;
        assert!(execute_result.is_err());
    }
}
