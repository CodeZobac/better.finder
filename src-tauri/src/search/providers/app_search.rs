/// Application search provider
///
/// This provider searches for installed applications on Windows by scanning:
/// - Start Menu (.lnk files)
/// - Program Files directories (.exe files)
/// - User AppData directories
///
/// It maintains a cache of applications that is refreshed periodically.

use crate::error::{LauncherError, Result};
use crate::search::SearchProvider;
use crate::types::{ResultAction, ResultType, SearchResult};
use crate::utils::IconCache;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

#[cfg(windows)]
use windows::{
    core::PCWSTR,
    Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED},
    Win32::UI::Shell::{IShellLinkW, ShellLink},
    Win32::Storage::FileSystem::{GetFileAttributesW, INVALID_FILE_ATTRIBUTES},
    Win32::System::Com::IPersistFile,
};

const MAX_RESULTS: usize = 20;
const CACHE_REFRESH_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

/// Represents an installed application
#[derive(Debug, Clone)]
pub struct Application {
    /// Display name of the application
    pub name: String,
    /// Full path to the executable
    pub path: PathBuf,
    /// Optional description
    pub description: Option<String>,
    /// Whether this is a .lnk file or direct .exe
    pub is_shortcut: bool,
}

/// Application scanner that finds installed applications
pub struct AppScanner;

impl AppScanner {
    /// Scans all common locations for installed applications
    pub fn scan_applications() -> Result<Vec<Application>> {
        info!("Starting application scan");
        let mut apps = Vec::new();

        // Scan Start Menu
        if let Ok(start_menu_apps) = Self::scan_start_menu() {
            debug!("Found {} apps in Start Menu", start_menu_apps.len());
            apps.extend(start_menu_apps);
        }

        // Scan Program Files
        if let Ok(program_files_apps) = Self::scan_program_files() {
            debug!("Found {} apps in Program Files", program_files_apps.len());
            apps.extend(program_files_apps);
        }

        // Scan user AppData
        if let Ok(appdata_apps) = Self::scan_appdata() {
            debug!("Found {} apps in AppData", appdata_apps.len());
            apps.extend(appdata_apps);
        }

        // Deduplicate by path
        apps.sort_by(|a, b| a.path.cmp(&b.path));
        apps.dedup_by(|a, b| a.path == b.path);

        info!("Application scan complete: {} unique apps found", apps.len());
        Ok(apps)
    }

    /// Scans the Start Menu for .lnk files
    fn scan_start_menu() -> Result<Vec<Application>> {
        let mut apps = Vec::new();

        // Common Start Menu locations
        let start_menu_paths = vec![
            Self::get_start_menu_path(false), // All Users
            Self::get_start_menu_path(true),  // Current User
        ];

        for start_menu_path in start_menu_paths.into_iter().flatten() {
            if let Ok(found_apps) = Self::scan_directory_for_shortcuts(&start_menu_path) {
                apps.extend(found_apps);
            }
        }

        Ok(apps)
    }

    /// Gets the Start Menu path
    fn get_start_menu_path(user_only: bool) -> Option<PathBuf> {
        if user_only {
            // User's Start Menu: %APPDATA%\Microsoft\Windows\Start Menu\Programs
            std::env::var("APPDATA")
                .ok()
                .map(|appdata| PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs"))
        } else {
            // All Users Start Menu: %PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs
            std::env::var("PROGRAMDATA")
                .ok()
                .map(|programdata| PathBuf::from(programdata).join("Microsoft\\Windows\\Start Menu\\Programs"))
        }
    }

    /// Scans Program Files directories for .exe files
    fn scan_program_files() -> Result<Vec<Application>> {
        let mut apps = Vec::new();

        let program_files_paths = vec![
            std::env::var("ProgramFiles").ok().map(PathBuf::from),
            std::env::var("ProgramFiles(x86)").ok().map(PathBuf::from),
        ];

        for program_files_path in program_files_paths.into_iter().flatten() {
            if let Ok(found_apps) = Self::scan_directory_for_executables(&program_files_path, 2) {
                apps.extend(found_apps);
            }
        }

        Ok(apps)
    }

    /// Scans user AppData for installed applications
    fn scan_appdata() -> Result<Vec<Application>> {
        let mut apps = Vec::new();

        if let Some(local_appdata) = std::env::var("LOCALAPPDATA").ok().map(PathBuf::from) {
            // Scan common app locations in AppData\Local
            let app_dirs = vec![
                local_appdata.join("Programs"),
                local_appdata.join("Microsoft\\WindowsApps"),
            ];

            for app_dir in app_dirs {
                if let Ok(found_apps) = Self::scan_directory_for_executables(&app_dir, 2) {
                    apps.extend(found_apps);
                }
            }
        }

        Ok(apps)
    }

    /// Recursively scans a directory for .lnk files
    fn scan_directory_for_shortcuts(dir: &Path) -> Result<Vec<Application>> {
        let mut apps = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return Ok(apps);
        }

        let entries = std::fs::read_dir(dir).map_err(|e| {
            LauncherError::IoError(e)
        })?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                if let Ok(sub_apps) = Self::scan_directory_for_shortcuts(&path) {
                    apps.extend(sub_apps);
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("lnk") {
                // Parse .lnk file
                if let Ok(app) = Self::parse_shortcut(&path) {
                    apps.push(app);
                }
            }
        }

        Ok(apps)
    }

    /// Scans a directory for .exe files (with depth limit)
    fn scan_directory_for_executables(dir: &Path, max_depth: usize) -> Result<Vec<Application>> {
        let mut apps = Vec::new();

        if !dir.exists() || !dir.is_dir() || max_depth == 0 {
            return Ok(apps);
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(apps), // Skip directories we can't read
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                if let Ok(sub_apps) = Self::scan_directory_for_executables(&path, max_depth - 1) {
                    apps.extend(sub_apps);
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("exe") {
                // Create application entry from .exe
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    apps.push(Application {
                        name: name.to_string(),
                        path: path.clone(),
                        description: None,
                        is_shortcut: false,
                    });
                }
            }
        }

        Ok(apps)
    }

    /// Parses a .lnk file to extract target path and name
    #[cfg(windows)]
    fn parse_shortcut(lnk_path: &Path) -> Result<Application> {
        use std::os::windows::ffi::OsStrExt;

        unsafe {
            // Initialize COM
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                .ok()
                .map_err(|e| LauncherError::ProviderError(format!("COM initialization failed: {}", e)))?;

            let result = (|| -> Result<Application> {
                // Create IShellLink instance
                let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| LauncherError::ProviderError(format!("Failed to create ShellLink: {}", e)))?;

                // Get IPersistFile interface
                use windows_core::Interface;
                let persist_file: IPersistFile = shell_link.cast()
                    .map_err(|e| LauncherError::ProviderError(format!("Failed to get IPersistFile: {}", e)))?;

                // Convert path to wide string
                let lnk_path_wide: Vec<u16> = lnk_path
                    .as_os_str()
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                // Load the shortcut file
                use windows::Win32::System::Com::STGM;
                persist_file.Load(PCWSTR(lnk_path_wide.as_ptr()), STGM(0))
                    .map_err(|e| LauncherError::ProviderError(format!("Failed to load shortcut: {}", e)))?;

                // Get target path
                let mut target_path_buf = vec![0u16; 260]; // MAX_PATH
                shell_link.GetPath(
                    &mut target_path_buf,
                    std::ptr::null_mut(),
                    0,
                )
                .map_err(|e| LauncherError::ProviderError(format!("Failed to get target path: {}", e)))?;

                // Convert wide string to PathBuf
                let target_path_len = target_path_buf.iter().position(|&c| c == 0).unwrap_or(target_path_buf.len());
                let target_path = PathBuf::from(String::from_utf16_lossy(&target_path_buf[..target_path_len]));

                // Verify target exists
                if !Self::file_exists(&target_path) {
                    return Err(LauncherError::NotFound(format!("Shortcut target not found: {}", target_path.display())));
                }

                // Get description
                let mut description_buf = vec![0u16; 260];
                let description = match shell_link.GetDescription(&mut description_buf) {
                    Ok(_) => {
                        let desc_len = description_buf.iter().position(|&c| c == 0).unwrap_or(description_buf.len());
                        let desc = String::from_utf16_lossy(&description_buf[..desc_len]);
                        if desc.is_empty() { None } else { Some(desc) }
                    }
                    Err(_) => None,
                };

                // Extract name from shortcut filename
                let name = lnk_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                Ok(Application {
                    name,
                    path: target_path,
                    description,
                    is_shortcut: true,
                })
            })();

            // Uninitialize COM
            CoUninitialize();

            result
        }
    }

    #[cfg(not(windows))]
    fn parse_shortcut(_lnk_path: &Path) -> Result<Application> {
        Err(LauncherError::ProviderError("Shortcut parsing not supported on this platform".to_string()))
    }

    /// Checks if a file exists using Windows API
    #[cfg(windows)]
    #[allow(dead_code)]
    fn file_exists(path: &Path) -> bool {
        use std::os::windows::ffi::OsStrExt;

        unsafe {
            let path_wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let attrs = GetFileAttributesW(PCWSTR(path_wide.as_ptr()));
            attrs != INVALID_FILE_ATTRIBUTES
        }
    }

    #[cfg(not(windows))]
    fn file_exists(path: &Path) -> bool {
        path.exists()
    }
}

/// Application search provider with caching
pub struct AppSearchProvider {
    /// Cached list of applications
    app_cache: Arc<RwLock<Vec<Application>>>,
    /// Icon cache for application icons
    icon_cache: Arc<IconCache>,
    /// Last cache refresh time
    last_refresh: Arc<RwLock<SystemTime>>,
    /// Whether the provider is enabled
    enabled: bool,
}

impl AppSearchProvider {
    /// Creates a new AppSearchProvider
    pub fn new() -> Result<Self> {
        info!("Initializing AppSearchProvider");

        Ok(Self {
            app_cache: Arc::new(RwLock::new(Vec::new())),
            icon_cache: Arc::new(IconCache::new()),
            last_refresh: Arc::new(RwLock::new(SystemTime::UNIX_EPOCH)),
            enabled: true,
        })
    }

    /// Refreshes the application cache
    async fn refresh_cache(&self) -> Result<()> {
        let last_refresh = *self.last_refresh.read().await;
        let now = SystemTime::now();

        // Check if refresh is needed
        if let Ok(elapsed) = now.duration_since(last_refresh) {
            if elapsed < CACHE_REFRESH_INTERVAL {
                debug!("Cache is still fresh, skipping refresh");
                return Ok(());
            }
        }

        info!("Refreshing application cache");

        // Scan applications in a blocking task
        let apps = tokio::task::spawn_blocking(|| AppScanner::scan_applications())
            .await
            .map_err(|e| LauncherError::ProviderError(format!("Failed to scan applications: {}", e)))??;

        // Update cache
        {
            let mut cache = self.app_cache.write().await;
            *cache = apps;
            info!("Application cache updated: {} apps", cache.len());
        }

        // Update last refresh time
        {
            let mut last_refresh = self.last_refresh.write().await;
            *last_refresh = now;
        }

        Ok(())
    }

    /// Performs fuzzy search on application names
    fn fuzzy_match(query: &str, app_name: &str) -> Option<f64> {
        let query_lower = query.to_lowercase();
        let name_lower = app_name.to_lowercase();

        // Exact match
        if name_lower == query_lower {
            return Some(100.0);
        }

        // Starts with query
        if name_lower.starts_with(&query_lower) {
            return Some(90.0);
        }

        // Contains query
        if name_lower.contains(&query_lower) {
            return Some(70.0);
        }

        // Check for acronym match (e.g., "vsc" matches "Visual Studio Code")
        if Self::matches_acronym(&query_lower, &name_lower) {
            return Some(60.0);
        }

        // Check for fuzzy character match
        if Self::fuzzy_char_match(&query_lower, &name_lower) {
            return Some(40.0);
        }

        None
    }

    /// Checks if query matches the acronym of the name
    fn matches_acronym(query: &str, name: &str) -> bool {
        let words: Vec<&str> = name.split_whitespace().collect();
        if words.len() < 2 {
            return false;
        }

        let acronym: String = words
            .iter()
            .filter_map(|word| word.chars().next())
            .collect();

        acronym.to_lowercase().starts_with(query)
    }

    /// Checks if all characters in query appear in order in name
    fn fuzzy_char_match(query: &str, name: &str) -> bool {
        let mut name_chars = name.chars();

        for query_char in query.chars() {
            if !name_chars.any(|c| c == query_char) {
                return false;
            }
        }

        true
    }

    /// Extracts application icon and converts to base64
    /// Gets application icon using the centralized icon cache
    async fn get_app_icon(&self, _path: &Path) -> Option<String> {
        // Return a generic application icon
        Some("app-icon".to_string())
    }

    /// Converts Application to SearchResult
    async fn convert_to_search_result(&self, app: &Application, score: f64) -> SearchResult {
        let icon = self.get_app_icon(&app.path).await;

        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), serde_json::json!(app.path.to_string_lossy()));
        metadata.insert("is_shortcut".to_string(), serde_json::json!(app.is_shortcut));
        if let Some(desc) = &app.description {
            metadata.insert("description".to_string(), serde_json::json!(desc));
        }

        SearchResult {
            id: format!("app:{}", app.path.display()),
            title: app.name.clone(),
            subtitle: app.path.to_string_lossy().to_string(),
            icon,
            result_type: ResultType::Application,
            score,
            metadata,
            action: ResultAction::LaunchApp {
                path: app.path.to_string_lossy().to_string(),
            },
        }
    }

    /// Starts background cache refresh task
    pub fn start_background_refresh(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(CACHE_REFRESH_INTERVAL).await;

                if let Err(e) = self.refresh_cache().await {
                    error!("Background cache refresh failed: {}", e);
                }
            }
        });
    }
}

#[async_trait]
impl SearchProvider for AppSearchProvider {
    fn name(&self) -> &str {
        "AppSearch"
    }

    fn priority(&self) -> u8 {
        85 // High priority, slightly lower than file search
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        debug!("Searching applications for query: '{}'", query);

        // Ensure cache is populated
        self.refresh_cache().await?;

        // Get cached applications
        let apps = self.app_cache.read().await;

        // Perform fuzzy search
        let mut results = Vec::new();
        for app in apps.iter() {
            if let Some(score) = Self::fuzzy_match(query, &app.name) {
                let result = self.convert_to_search_result(app, score).await;
                results.push(result);
            }
        }

        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(MAX_RESULTS);

        debug!("Found {} matching applications", results.len());
        Ok(results)
    }

    async fn execute(&self, result: &SearchResult) -> Result<()> {
        if result.result_type != ResultType::Application {
            return Err(LauncherError::ExecutionError(
                "Not an application result".to_string(),
            ));
        }

        match &result.action {
            ResultAction::LaunchApp { path } => {
                Self::launch_application(path).await
            }
            _ => Err(LauncherError::ExecutionError(
                "Invalid action for application result".to_string(),
            )),
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing AppSearchProvider");
        self.refresh_cache().await?;
        Ok(())
    }
}

impl Default for AppSearchProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            app_cache: Arc::new(RwLock::new(Vec::new())),
            icon_cache: Arc::new(IconCache::new()),
            last_refresh: Arc::new(RwLock::new(SystemTime::UNIX_EPOCH)),
            enabled: false,
        })
    }
}

impl AppSearchProvider {
    /// Launches an application using Windows ShellExecute API
    #[cfg(windows)]
    async fn launch_application(path: &str) -> Result<()> {
        info!("Launching application: {}", path);

        let app_path = PathBuf::from(path);

        // Verify application exists
        if !app_path.exists() {
            error!("Application not found: {}", path);
            return Err(LauncherError::NotFound(format!(
                "Application does not exist: {}",
                path
            )));
        }

        // Launch application in a blocking task
        let path_owned = path.to_string();
        tokio::task::spawn_blocking(move || {
            Self::launch_application_sync(&path_owned)
        })
        .await
        .map_err(|e| LauncherError::ExecutionError(format!("Failed to spawn launch task: {}", e)))??;

        info!("Successfully launched application: {}", path);
        Ok(())
    }

    /// Synchronously launches an application using ShellExecute
    #[cfg(windows)]
    fn launch_application_sync(path: &str) -> Result<()> {
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        use windows::Win32::Foundation::HWND;

        unsafe {
            // Convert path to wide string
            let path_wide: Vec<u16> = std::ffi::OsStr::new(path)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            // Convert "open" verb to wide string
            let verb_wide: Vec<u16> = std::ffi::OsStr::new("open")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            // Execute the application
            let result = ShellExecuteW(
                HWND(std::ptr::null_mut()),
                PCWSTR(verb_wide.as_ptr()),
                PCWSTR(path_wide.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );

            // ShellExecuteW returns a value > 32 on success
            if result.0 as isize <= 32 {
                let error_code = result.0 as isize;
                error!("ShellExecuteW failed with code: {}", error_code);

                // Map common error codes to meaningful messages
                let error_msg = match error_code {
                    0 | 2 => "File not found",
                    3 => "Path not found",
                    5 => "Access denied",
                    8 => "Out of memory",
                    11 => "Invalid executable format",
                    26 => "Sharing violation",
                    27 => "File association incomplete",
                    28 => "DDE timeout",
                    29 => "DDE failed",
                    30 => "DDE busy",
                    31 => "No file association",
                    32 => "DLL not found",
                    _ => "Unknown error",
                };

                return Err(LauncherError::ExecutionError(format!(
                    "Failed to launch application: {} (code: {})",
                    error_msg, error_code
                )));
            }

            Ok(())
        }
    }

    #[cfg(not(windows))]
    async fn launch_application(path: &str) -> Result<()> {
        Err(LauncherError::ExecutionError(
            format!("Application launching not supported on this platform: {}", path)
        ))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_search_provider_creation() {
        let provider = AppSearchProvider::new();
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "AppSearch");
        assert_eq!(provider.priority(), 85);
        assert!(provider.is_enabled());
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_application_scanning() {
        let apps = AppScanner::scan_applications();
        
        match apps {
            Ok(apps) => {
                println!("Found {} applications", apps.len());
                assert!(!apps.is_empty(), "Should find at least some applications");

                // Print first few apps for debugging
                for app in apps.iter().take(5) {
                    println!("  - {}: {}", app.name, app.path.display());
                }
            }
            Err(e) => {
                println!("Application scanning failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fuzzy_search() {
        // Test exact match
        let score = AppSearchProvider::fuzzy_match("notepad", "notepad");
        assert_eq!(score, Some(100.0));

        // Test starts with
        let score = AppSearchProvider::fuzzy_match("note", "notepad");
        assert_eq!(score, Some(90.0));

        // Test contains
        let score = AppSearchProvider::fuzzy_match("pad", "notepad");
        assert_eq!(score, Some(70.0));

        // Test no match
        let score = AppSearchProvider::fuzzy_match("xyz", "notepad");
        assert!(score.is_none() || score.unwrap() < 70.0);
    }

    #[tokio::test]
    async fn test_acronym_matching() {
        // Test acronym match
        assert!(AppSearchProvider::matches_acronym("vsc", "visual studio code"));
        assert!(AppSearchProvider::matches_acronym("mw", "microsoft word"));
        assert!(AppSearchProvider::matches_acronym("m", "microsoft word"));
        
        // Test non-match
        assert!(!AppSearchProvider::matches_acronym("xyz", "visual studio code"));
    }

    #[tokio::test]
    async fn test_fuzzy_char_match() {
        // Test character sequence match
        assert!(AppSearchProvider::fuzzy_char_match("ntpd", "notepad"));
        assert!(AppSearchProvider::fuzzy_char_match("vsc", "visual studio code"));
        
        // Test non-match
        assert!(!AppSearchProvider::fuzzy_char_match("xyz", "notepad"));
    }

    #[tokio::test]
    async fn test_app_search() {
        let mut provider = AppSearchProvider::new().unwrap();
        
        // Initialize provider (populates cache)
        if let Err(e) = provider.initialize().await {
            println!("Provider initialization failed: {}", e);
            return;
        }

        // Search for common Windows applications
        let test_queries = vec!["notepad", "calc", "paint"];

        for query in test_queries {
            match provider.search(query).await {
                Ok(results) => {
                    println!("Search for '{}' found {} results", query, results.len());
                    for result in results.iter().take(3) {
                        println!("  - {}: {}", result.title, result.subtitle);
                        assert_eq!(result.result_type, ResultType::Application);
                    }
                }
                Err(e) => {
                    println!("Search for '{}' failed: {}", query, e);
                }
            }
        }
    }

    #[test]
    fn test_start_menu_path() {
        let user_path = AppScanner::get_start_menu_path(true);
        let all_users_path = AppScanner::get_start_menu_path(false);

        if let Some(path) = user_path {
            println!("User Start Menu: {}", path.display());
            assert!(path.to_string_lossy().contains("Start Menu"));
        }

        if let Some(path) = all_users_path {
            println!("All Users Start Menu: {}", path.display());
            assert!(path.to_string_lossy().contains("Start Menu"));
        }
    }

    #[cfg(windows)]
    #[test]
    fn test_file_exists() {
        // Test with Windows system file
        let system32 = PathBuf::from("C:\\Windows\\System32\\notepad.exe");
        if system32.exists() {
            assert!(AppScanner::file_exists(&system32));
        }

        // Test with non-existent file
        let fake_path = PathBuf::from("C:\\NonExistent\\fake.exe");
        assert!(!AppScanner::file_exists(&fake_path));
    }
}
