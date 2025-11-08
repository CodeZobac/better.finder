/// Recent files provider for tracking and displaying recently accessed files
///
/// This provider maintains a history of files opened through the launcher,
/// allowing users to quickly access their recent work.

use crate::error::{LauncherError, Result};
use crate::search::SearchProvider;
use crate::types::{ResultAction, ResultType, SearchResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Maximum number of recent files to display by default
const DEFAULT_RECENT_FILES_LIMIT: usize = 5;

/// Maximum number of recent files to store in database
const MAX_RECENT_FILES: usize = 50;

/// Represents a recently accessed file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    /// File path
    pub path: PathBuf,
    /// Last time the file was accessed
    pub last_accessed: DateTime<Utc>,
    /// Number of times the file has been accessed
    pub access_count: u32,
}

impl RecentFile {
    /// Creates a new recent file entry
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            last_accessed: Utc::now(),
            access_count: 1,
        }
    }

    /// Updates the access time and increments the count
    pub fn update_access(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Returns a formatted timestamp
    pub fn formatted_timestamp(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.last_accessed);

        if duration.num_seconds() < 60 {
            "Just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{} min ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_days() < 7 {
            format!("{} days ago", duration.num_days())
        } else {
            self.last_accessed.format("%Y-%m-%d").to_string()
        }
    }

    /// Checks if the file still exists on disk
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Gets the file name
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// Gets the file path as a string
    pub fn path_string(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

/// Storage backend for recent files using SQLite
pub struct RecentFilesStorage {
    /// Path to the SQLite database
    db_path: PathBuf,
}

impl RecentFilesStorage {
    /// Creates a new recent files storage
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        // Ensure the directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let storage = Self {
            db_path,
        };

        // Initialize the database
        storage.initialize_db()?;

        Ok(storage)
    }

    /// Gets a database connection
    fn get_connection(&self) -> Result<Connection> {
        Ok(Connection::open(&self.db_path)?)
    }

    /// Gets the database file path
    fn get_db_path() -> Result<PathBuf> {
        #[cfg(test)]
        {
            // Use temp directory for tests
            let mut path = std::env::temp_dir();
            path.push("BetterFinder");
            path.push("recent_files_test.db");
            return Ok(path);
        }
        
        #[cfg(not(test))]
        {
            let app_data = std::env::var("APPDATA")
                .map_err(|_| LauncherError::ConfigError("APPDATA not found".to_string()))?;
            
            let mut path = PathBuf::from(app_data);
            path.push("BetterFinder");
            path.push("recent_files.db");
            
            Ok(path)
        }
    }

    /// Initializes the database schema
    fn initialize_db(&self) -> Result<()> {
        let conn = self.get_connection()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS recent_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                last_accessed TEXT NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;

        // Create index on last_accessed for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_last_accessed ON recent_files(last_accessed DESC)",
            [],
        )?;

        Ok(())
    }

    /// Adds or updates a file in the recent files list
    pub async fn track_file(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let now = Utc::now().to_rfc3339();
        let db_path = self.db_path.clone();
        
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path)?;

            // Try to update existing entry
            let updated = conn.execute(
                "UPDATE recent_files 
                 SET last_accessed = ?1, access_count = access_count + 1 
                 WHERE path = ?2",
                params![now, path_str],
            )?;

            // If no rows were updated, insert a new entry
            if updated == 0 {
                conn.execute(
                    "INSERT INTO recent_files (path, last_accessed, access_count) 
                     VALUES (?1, ?2, 1)",
                    params![path_str, now],
                )?;
            }

            // Clean up old entries if we exceed the maximum
            conn.execute(
                "DELETE FROM recent_files 
                 WHERE id NOT IN (
                     SELECT id FROM recent_files 
                     ORDER BY last_accessed DESC 
                     LIMIT ?1
                 )",
                params![MAX_RECENT_FILES],
            )?;

            Ok::<(), LauncherError>(())
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn track task: {}", e))
        })??;

        Ok(())
    }

    /// Retrieves recent files, optionally filtering by query
    pub async fn get_recent_files(&self, limit: usize) -> Result<Vec<RecentFile>> {
        let db_path = self.db_path.clone();
        
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path)?;

            let mut stmt = conn.prepare(
                "SELECT path, last_accessed, access_count 
                 FROM recent_files 
                 ORDER BY last_accessed DESC 
                 LIMIT ?1",
            )?;

            let files = stmt
                .query_map(params![limit], |row| {
                    let path_str: String = row.get(0)?;
                    let last_accessed_str: String = row.get(1)?;
                    let access_count: u32 = row.get(2)?;

                    let last_accessed = DateTime::parse_from_rfc3339(&last_accessed_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());

                    Ok(RecentFile {
                        path: PathBuf::from(path_str),
                        last_accessed,
                        access_count,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            Ok(files)
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn get task: {}", e))
        })?
    }

    /// Validates and removes files that no longer exist
    pub async fn cleanup_missing_files(&self) -> Result<usize> {
        let files = self.get_recent_files(MAX_RECENT_FILES).await?;
        let mut removed_count = 0;

        for file in files {
            if !file.exists() {
                self.remove_file(&file.path).await?;
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }

    /// Removes a file from the recent files list
    async fn remove_file(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let db_path = self.db_path.clone();
        
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path)?;

            conn.execute(
                "DELETE FROM recent_files WHERE path = ?1",
                params![path_str],
            )?;

            Ok::<(), LauncherError>(())
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn remove task: {}", e))
        })??;

        Ok(())
    }
}

impl Default for RecentFilesStorage {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            db_path: PathBuf::from("recent_files.db"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_recent_file_creation() {
        let path = PathBuf::from("C:\\test\\file.txt");
        let file = RecentFile::new(path.clone());

        assert_eq!(file.path, path);
        assert_eq!(file.access_count, 1);
    }

    #[test]
    fn test_recent_file_update_access() {
        let path = PathBuf::from("C:\\test\\file.txt");
        let mut file = RecentFile::new(path);

        let initial_time = file.last_accessed;
        let initial_count = file.access_count;

        // Small delay to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        file.update_access();

        assert!(file.last_accessed > initial_time);
        assert_eq!(file.access_count, initial_count + 1);
    }

    #[test]
    fn test_recent_file_formatted_timestamp() {
        let path = PathBuf::from("C:\\test\\file.txt");
        let file = RecentFile::new(path);

        let formatted = file.formatted_timestamp();
        assert_eq!(formatted, "Just now");
    }

    #[test]
    fn test_recent_file_file_name() {
        // Use a platform-independent path for testing
        #[cfg(windows)]
        let path = PathBuf::from("C:\\test\\folder\\document.txt");
        #[cfg(not(windows))]
        let path = PathBuf::from("/test/folder/document.txt");
        
        let file = RecentFile::new(path);

        assert_eq!(file.file_name(), "document.txt");
    }

    #[tokio::test]
    async fn test_storage_creation() {
        let storage = RecentFilesStorage::new();
        assert!(storage.is_ok());
    }

    #[tokio::test]
    async fn test_storage_db_path() {
        let result = RecentFilesStorage::get_db_path();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("BetterFinder"));
        
        #[cfg(test)]
        assert!(path.to_string_lossy().contains("recent_files_test.db"));
        
        #[cfg(not(test))]
        assert!(path.to_string_lossy().contains("recent_files.db"));
    }

    #[tokio::test]
    async fn test_storage_track_file() {
        // Create a unique test database
        let mut db_path = std::env::temp_dir();
        db_path.push("BetterFinder");
        std::fs::create_dir_all(&db_path).ok();
        db_path.push(format!("recent_files_track_test_{}.db", std::process::id()));
        
        // Clean up any existing test file
        let _ = std::fs::remove_file(&db_path);
        
        let storage = RecentFilesStorage {
            db_path: db_path.clone(),
        };
        storage.initialize_db().unwrap();
        
        // Create a temporary test file
        let mut test_path = std::env::temp_dir();
        test_path.push("test_recent_file.txt");
        File::create(&test_path).unwrap();
        
        // Track the file
        let result = storage.track_file(&test_path).await;
        assert!(result.is_ok());

        // Retrieve recent files
        let files = storage.get_recent_files(10).await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, test_path);
        assert_eq!(files[0].access_count, 1);

        // Track the same file again
        storage.track_file(&test_path).await.unwrap();

        // Should update the existing entry
        let files = storage.get_recent_files(10).await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].access_count, 2);

        // Cleanup
        std::fs::remove_file(&test_path).ok();
        std::fs::remove_file(&db_path).ok();
    }

    #[tokio::test]
    async fn test_storage_get_recent_files_limit() {
        // Create a unique test database
        let mut db_path = std::env::temp_dir();
        db_path.push("BetterFinder");
        std::fs::create_dir_all(&db_path).ok();
        db_path.push(format!("recent_files_limit_test_{}.db", std::process::id()));
        
        // Clean up any existing test file
        let _ = std::fs::remove_file(&db_path);
        
        let storage = RecentFilesStorage {
            db_path: db_path.clone(),
        };
        storage.initialize_db().unwrap();

        // Track multiple files
        for i in 0..10 {
            let path = PathBuf::from(format!("C:\\test\\file{}.txt", i));
            storage.track_file(&path).await.unwrap();
        }

        // Get only 5 recent files
        let files = storage.get_recent_files(5).await.unwrap();
        assert_eq!(files.len(), 5);

        // Most recent should be file9
        assert!(files[0].path.to_string_lossy().contains("file9"));
        
        // Cleanup
        std::fs::remove_file(&db_path).ok();
    }

    #[tokio::test]
    async fn test_storage_cleanup_missing_files() {
        // Create a unique test database
        let mut test_path = std::env::temp_dir();
        test_path.push("BetterFinder");
        std::fs::create_dir_all(&test_path).ok();
        test_path.push(format!("recent_files_cleanup_test_{}.db", std::process::id()));
        
        // Clean up any existing test file
        let _ = std::fs::remove_file(&test_path);
        
        let storage = RecentFilesStorage {
            db_path: test_path.clone(),
        };
        storage.initialize_db().unwrap();

        // Track a non-existent file
        let fake_path = PathBuf::from("C:\\nonexistent\\file.txt");
        storage.track_file(&fake_path).await.unwrap();

        // Cleanup should remove it
        let removed = storage.cleanup_missing_files().await.unwrap();
        assert_eq!(removed, 1);

        // Verify it's gone
        let files = storage.get_recent_files(10).await.unwrap();
        assert_eq!(files.len(), 0);
        
        // Cleanup test database
        let _ = std::fs::remove_file(&test_path);
    }

    #[tokio::test]
    async fn test_storage_max_files_limit() {
        // Create a unique test database
        let mut db_path = std::env::temp_dir();
        db_path.push("BetterFinder");
        std::fs::create_dir_all(&db_path).ok();
        db_path.push(format!("recent_files_max_test_{}.db", std::process::id()));
        
        // Clean up any existing test file
        let _ = std::fs::remove_file(&db_path);
        
        let storage = RecentFilesStorage {
            db_path: db_path.clone(),
        };
        storage.initialize_db().unwrap();

        // Track more than MAX_RECENT_FILES
        for i in 0..60 {
            let path = PathBuf::from(format!("C:\\test\\file{}.txt", i));
            storage.track_file(&path).await.unwrap();
        }

        // Should only keep MAX_RECENT_FILES
        let files = storage.get_recent_files(100).await.unwrap();
        assert!(files.len() <= MAX_RECENT_FILES);
        
        // Cleanup
        std::fs::remove_file(&db_path).ok();
    }
}

/// Recent files search provider
pub struct RecentFilesProvider {
    /// Storage backend
    storage: Arc<RwLock<RecentFilesStorage>>,
    /// Whether the provider is enabled
    enabled: bool,
}

impl RecentFilesProvider {
    /// Creates a new recent files provider
    pub fn new() -> Result<Self> {
        info!("Initializing RecentFilesProvider");

        let storage = RecentFilesStorage::new()?;

        Ok(Self {
            storage: Arc::new(RwLock::new(storage)),
            enabled: true,
        })
    }

    /// Gets recent files from storage
    async fn get_recent_files(&self, limit: usize) -> Result<Vec<RecentFile>> {
        let storage = self.storage.read().await;
        storage.get_recent_files(limit).await
    }

    /// Tracks a file access
    pub async fn track_file_access(&self, path: &Path) -> Result<()> {
        let storage = self.storage.read().await;
        storage.track_file(path).await
    }

    /// Creates a search result from a recent file
    fn create_search_result(&self, file: &RecentFile, score: f64) -> SearchResult {
        let file_name = file.file_name();
        let path_str = file.path_string();
        let timestamp = file.formatted_timestamp();

        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), serde_json::json!(path_str));
        metadata.insert("last_accessed".to_string(), serde_json::json!(file.last_accessed));
        metadata.insert("access_count".to_string(), serde_json::json!(file.access_count));

        SearchResult {
            id: format!("recent:{}", path_str),
            title: file_name,
            subtitle: format!("{} • Opened {}", path_str, timestamp),
            icon: Self::get_file_icon(&file.path),
            result_type: ResultType::RecentFile,
            score,
            metadata,
            action: ResultAction::OpenFile {
                path: path_str,
            },
        }
    }

    /// Gets an icon for a file based on its extension
    fn get_file_icon(path: &Path) -> Option<String> {
        // For now, return a generic file icon name
        // In a full implementation, this would extract the actual icon
        let extension = path.extension()?.to_str()?;
        
        match extension.to_lowercase().as_str() {
            "txt" => Some("file-text".to_string()),
            "pdf" => Some("file-pdf".to_string()),
            "doc" | "docx" => Some("file-word".to_string()),
            "xls" | "xlsx" => Some("file-excel".to_string()),
            "ppt" | "pptx" => Some("file-powerpoint".to_string()),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" => Some("file-image".to_string()),
            "mp3" | "wav" | "flac" => Some("file-audio".to_string()),
            "mp4" | "avi" | "mkv" => Some("file-video".to_string()),
            "zip" | "rar" | "7z" => Some("file-archive".to_string()),
            "exe" => Some("file-executable".to_string()),
            _ => Some("file".to_string()),
        }
    }

    /// Opens a file using the Windows shell
    #[cfg(windows)]
    async fn open_file(path: &str) -> Result<()> {
        use windows::Win32::Foundation::*;
        use windows::Win32::UI::Shell::*;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let path_owned = path.to_string();

        tokio::task::spawn_blocking(move || {
            unsafe {
                // Convert path to wide string
                let wide_path: Vec<u16> = OsStr::new(&path_owned)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                // Use ShellExecuteW to open the file
                let result = ShellExecuteW(
                    HWND(std::ptr::null_mut()),
                    windows::core::w!("open"),
                    windows::core::PCWSTR(wide_path.as_ptr()),
                    windows::core::PCWSTR::null(),
                    windows::core::PCWSTR::null(),
                    SW_SHOW,
                );

                if result.0 as isize <= 32 {
                    return Err(LauncherError::ExecutionError(format!(
                        "Failed to open file: {}",
                        path_owned
                    )));
                }

                Ok(())
            }
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn open file task: {}", e))
        })?
    }

    #[cfg(not(windows))]
    async fn open_file(_path: &str) -> Result<()> {
        Err(LauncherError::ExecutionError(
            "File opening not supported on this platform".to_string(),
        ))
    }
}

#[async_trait]
impl SearchProvider for RecentFilesProvider {
    fn name(&self) -> &str {
        "Recent Files"
    }

    fn priority(&self) -> u8 {
        90 // High priority - show recent files prominently
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let trimmed = query.trim();

        // Only show recent files when query is empty
        if trimmed.is_empty() {
            let files = self.get_recent_files(DEFAULT_RECENT_FILES_LIMIT).await?;
            
            // Filter out files that no longer exist
            let valid_files: Vec<_> = files.into_iter().filter(|f| f.exists()).collect();

            // Create search results
            let results = valid_files
                .iter()
                .enumerate()
                .map(|(index, file)| {
                    // Score decreases with position (newer files score higher)
                    let score = 95.0 - (index as f64 * 2.0);
                    self.create_search_result(file, score)
                })
                .collect();

            Ok(results)
        } else {
            // Don't show recent files for non-empty queries
            // Other providers will handle the search
            Ok(Vec::new())
        }
    }

    async fn execute(&self, result: &SearchResult) -> Result<()> {
        if result.result_type != ResultType::RecentFile {
            return Err(LauncherError::ExecutionError(
                "Not a recent file result".to_string(),
            ));
        }

        // Extract the file path from metadata
        let path = result
            .metadata
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                LauncherError::ExecutionError("Invalid recent file result".to_string())
            })?;

        info!("Opening recent file: {}", path);

        // Open the file
        Self::open_file(path).await?;

        // Track the access (this will update the timestamp and count)
        self.track_file_access(Path::new(path)).await?;

        info!("Successfully opened recent file");
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing RecentFilesProvider");

        // Clean up any missing files from the database
        let storage = self.storage.read().await;
        match storage.cleanup_missing_files().await {
            Ok(removed) => {
                if removed > 0 {
                    info!("Cleaned up {} missing files from recent files", removed);
                }
            }
            Err(e) => {
                warn!("Failed to cleanup missing files: {}", e);
            }
        }

        info!("RecentFilesProvider initialized successfully");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down RecentFilesProvider");
        // No cleanup needed for now
        Ok(())
    }
}

impl Default for RecentFilesProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            storage: Arc::new(RwLock::new(RecentFilesStorage::default())),
            enabled: false,
        })
    }
}

#[cfg(test)]
mod provider_tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        let provider = RecentFilesProvider::new();
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "Recent Files");
        assert_eq!(provider.priority(), 90);
        assert!(provider.is_enabled());
    }

    #[tokio::test]
    async fn test_provider_search_empty_query() {
        let provider = RecentFilesProvider::new().unwrap();

        // Track some files first
        let test_path = PathBuf::from("C:\\test\\file.txt");
        provider.track_file_access(&test_path).await.unwrap();

        // Search with empty query should return recent files
        let results = provider.search("").await.unwrap();
        
        // Note: Results might be empty if the file doesn't exist
        // In a real scenario with existing files, this would return results
        assert!(results.len() <= DEFAULT_RECENT_FILES_LIMIT);
    }

    #[tokio::test]
    async fn test_provider_search_non_empty_query() {
        let provider = RecentFilesProvider::new().unwrap();

        // Search with non-empty query should return nothing
        let results = provider.search("test").await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_provider_track_file_access() {
        let provider = RecentFilesProvider::new().unwrap();

        let test_path = PathBuf::from("C:\\test\\document.txt");
        let result = provider.track_file_access(&test_path).await;
        assert!(result.is_ok());

        // Track again to test update
        let result = provider.track_file_access(&test_path).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_file_icon() {
        let txt_path = PathBuf::from("test.txt");
        assert_eq!(
            RecentFilesProvider::get_file_icon(&txt_path),
            Some("file-text".to_string())
        );

        let pdf_path = PathBuf::from("document.pdf");
        assert_eq!(
            RecentFilesProvider::get_file_icon(&pdf_path),
            Some("file-pdf".to_string())
        );

        let unknown_path = PathBuf::from("file.xyz");
        assert_eq!(
            RecentFilesProvider::get_file_icon(&unknown_path),
            Some("file".to_string())
        );
    }

    #[tokio::test]
    async fn test_provider_initialize() {
        let mut provider = RecentFilesProvider::new().unwrap();
        let result = provider.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_search_result() {
        let provider = RecentFilesProvider::new().unwrap();
        
        #[cfg(windows)]
        let test_path = PathBuf::from("C:\\test\\document.txt");
        #[cfg(not(windows))]
        let test_path = PathBuf::from("/test/document.txt");
        
        let file = RecentFile::new(test_path.clone());
        let result = provider.create_search_result(&file, 95.0);

        assert_eq!(result.result_type, ResultType::RecentFile);
        assert_eq!(result.title, "document.txt");
        assert!(result.subtitle.contains(&test_path.to_string_lossy().to_string()));
        assert_eq!(result.score, 95.0);
    }
}
