use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

#[cfg(windows)]
use windows::{
    core::PCWSTR,
    Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON},
    Win32::UI::WindowsAndMessaging::DestroyIcon,
};

/// Maximum size for icons to be base64 encoded (in bytes)
const MAX_ICON_SIZE_FOR_BASE64: usize = 10_240; // 10KB

/// Default icon cache capacity
const DEFAULT_CACHE_CAPACITY: usize = 100;

/// Icon cache for storing extracted and encoded icons
pub struct IconCache {
    cache: Arc<RwLock<LruCache<PathBuf, String>>>,
}

impl IconCache {
    /// Creates a new IconCache with default capacity
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CACHE_CAPACITY)
    }

    /// Creates a new IconCache with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap());
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
        }
    }

    /// Gets an icon from cache or extracts it if not cached
    pub async fn get_or_extract(&self, path: &Path) -> Option<String> {
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(icon) = cache.get(path) {
                debug!("Icon cache hit for: {}", path.display());
                return Some(icon.clone());
            }
        }

        debug!("Icon cache miss for: {}", path.display());

        // Extract icon in blocking thread
        let path_buf = path.to_path_buf();
        let icon = tokio::task::spawn_blocking(move || Self::extract_icon_sync(&path_buf))
            .await
            .ok()??;

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.put(path.to_path_buf(), icon.clone());
        }

        Some(icon)
    }

    /// Gets an icon from cache without extracting
    pub async fn get(&self, path: &Path) -> Option<String> {
        let mut cache = self.cache.write().await;
        cache.get(path).cloned()
    }

    /// Puts an icon into the cache
    pub async fn put(&self, path: PathBuf, icon: String) {
        let mut cache = self.cache.write().await;
        cache.put(path, icon);
    }

    /// Clears the cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("Icon cache cleared");
    }

    /// Returns the number of cached icons
    pub async fn len(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Returns whether the cache is empty
    pub async fn is_empty(&self) -> bool {
        let cache = self.cache.read().await;
        cache.is_empty()
    }

    /// Synchronously extracts icon from file (runs in blocking thread)
    #[cfg(windows)]
    fn extract_icon_sync(path: &Path) -> Option<String> {
        use std::os::windows::ffi::OsStrExt;

        unsafe {
            // Convert path to wide string
            let path_wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let mut file_info: SHFILEINFOW = std::mem::zeroed();

            // Get file icon
            use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
            let result = SHGetFileInfoW(
                PCWSTR(path_wide.as_ptr()),
                FILE_FLAGS_AND_ATTRIBUTES(0),
                Some(&mut file_info),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_SMALLICON,
            );

            if result == 0 {
                debug!("Failed to get icon for: {}", path.display());
                return None;
            }

            // For now, return a placeholder based on file extension
            // Full HICON to base64 conversion would require additional image processing
            let icon_identifier = if let Some(ext) = path.extension() {
                format!("file-icon:{}", ext.to_str().unwrap_or("unknown"))
            } else {
                "file-icon:unknown".to_string()
            };

            // Clean up icon handle
            if !file_info.hIcon.is_invalid() {
                let _ = DestroyIcon(file_info.hIcon);
            }

            Some(icon_identifier)
        }
    }

    #[cfg(not(windows))]
    fn extract_icon_sync(path: &Path) -> Option<String> {
        // Return a generic icon identifier based on extension
        if let Some(ext) = path.extension() {
            Some(format!("file-icon:{}", ext.to_str().unwrap_or("unknown")))
        } else {
            Some("file-icon:unknown".to_string())
        }
    }

    /// Gets a generic icon name based on file extension
    pub fn get_generic_icon(path: &Path) -> String {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");

        match extension.to_lowercase().as_str() {
            // Documents
            "txt" | "md" | "log" => "file-text",
            "pdf" => "file-pdf",
            "doc" | "docx" => "file-word",
            "xls" | "xlsx" => "file-excel",
            "ppt" | "pptx" => "file-powerpoint",
            
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" => "file-image",
            
            // Videos
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" => "file-video",
            
            // Audio
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" => "file-audio",
            
            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => "file-archive",
            
            // Code
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "h" | "hpp" => "file-code",
            "html" | "css" | "json" | "xml" | "yaml" | "yml" => "file-code",
            
            // Executables
            "exe" | "msi" | "bat" | "cmd" | "ps1" => "file-executable",
            
            // Default
            _ => "file",
        }
        .to_string()
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Encodes data to base64 if it's small enough
pub fn encode_to_base64_if_small(data: &[u8]) -> Option<String> {
    if data.len() <= MAX_ICON_SIZE_FOR_BASE64 {
        Some(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data))
    } else {
        warn!("Icon data too large for base64 encoding: {} bytes", data.len());
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_icon_cache_basic() {
        let cache = IconCache::new();
        let path = PathBuf::from("test.txt");
        
        // Put an icon in cache
        cache.put(path.clone(), "test-icon".to_string()).await;
        
        // Get it back
        let icon = cache.get(&path).await;
        assert_eq!(icon, Some("test-icon".to_string()));
    }

    #[tokio::test]
    async fn test_icon_cache_miss() {
        let cache = IconCache::new();
        let path = PathBuf::from("nonexistent.txt");
        
        let icon = cache.get(&path).await;
        assert_eq!(icon, None);
    }

    #[tokio::test]
    async fn test_icon_cache_clear() {
        let cache = IconCache::new();
        let path = PathBuf::from("test.txt");
        
        cache.put(path.clone(), "test-icon".to_string()).await;
        assert_eq!(cache.len().await, 1);
        
        cache.clear().await;
        assert_eq!(cache.len().await, 0);
        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_icon_cache_lru_eviction() {
        let cache = IconCache::with_capacity(2);
        
        cache.put(PathBuf::from("file1.txt"), "icon1".to_string()).await;
        cache.put(PathBuf::from("file2.txt"), "icon2".to_string()).await;
        cache.put(PathBuf::from("file3.txt"), "icon3".to_string()).await;
        
        // file1 should be evicted
        assert_eq!(cache.get(&PathBuf::from("file1.txt")).await, None);
        assert_eq!(cache.get(&PathBuf::from("file2.txt")).await, Some("icon2".to_string()));
        assert_eq!(cache.get(&PathBuf::from("file3.txt")).await, Some("icon3".to_string()));
    }

    #[test]
    fn test_generic_icon_names() {
        assert_eq!(IconCache::get_generic_icon(&PathBuf::from("test.txt")), "file-text");
        assert_eq!(IconCache::get_generic_icon(&PathBuf::from("doc.pdf")), "file-pdf");
        assert_eq!(IconCache::get_generic_icon(&PathBuf::from("image.png")), "file-image");
        assert_eq!(IconCache::get_generic_icon(&PathBuf::from("video.mp4")), "file-video");
        assert_eq!(IconCache::get_generic_icon(&PathBuf::from("song.mp3")), "file-audio");
        assert_eq!(IconCache::get_generic_icon(&PathBuf::from("archive.zip")), "file-archive");
        assert_eq!(IconCache::get_generic_icon(&PathBuf::from("code.rs")), "file-code");
        assert_eq!(IconCache::get_generic_icon(&PathBuf::from("app.exe")), "file-executable");
        assert_eq!(IconCache::get_generic_icon(&PathBuf::from("unknown.xyz")), "file");
    }

    #[test]
    fn test_encode_to_base64_if_small() {
        let small_data = vec![1, 2, 3, 4, 5];
        assert!(encode_to_base64_if_small(&small_data).is_some());
        
        let large_data = vec![0u8; MAX_ICON_SIZE_FOR_BASE64 + 1];
        assert!(encode_to_base64_if_small(&large_data).is_none());
    }
}
