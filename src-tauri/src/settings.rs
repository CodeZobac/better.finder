use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::error::{LauncherError, Result};

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Global hotkey combination (e.g., "Ctrl+K")
    pub hotkey: String,

    /// UI theme
    pub theme: Theme,

    /// Maximum number of results to display
    pub max_results: usize,

    /// Enabled search providers
    pub enabled_providers: EnabledProviders,

    /// Search input debounce delay in milliseconds
    pub search_delay: u64,

    /// Whether to start with Windows
    pub start_with_windows: bool,
}

/// UI theme options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Configuration for which providers are enabled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnabledProviders {
    pub files: bool,
    pub applications: bool,
    pub quick_actions: bool,
    pub calculator: bool,
    pub clipboard: bool,
    pub bookmarks: bool,
    pub recent_files: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+K".to_string(),
            theme: Theme::System,
            max_results: 8,
            enabled_providers: EnabledProviders::default(),
            search_delay: 150,
            start_with_windows: false,
        }
    }
}

impl Default for EnabledProviders {
    fn default() -> Self {
        Self {
            files: true,
            applications: true,
            quick_actions: true,
            calculator: true,
            clipboard: true,
            bookmarks: true,
            recent_files: true,
        }
    }
}

impl AppSettings {
    /// Load settings from disk, or create default if not found
    pub fn load() -> Result<Self> {
        let path = Self::settings_path()?;
        
        if path.exists() {
            let contents = fs::read_to_string(&path)
                .map_err(|e| LauncherError::SettingsError(format!("Failed to read settings: {}", e)))?;
            
            let settings: AppSettings = serde_json::from_str(&contents)
                .map_err(|e| LauncherError::SettingsError(format!("Failed to parse settings: {}", e)))?;
            
            settings.validate()?;
            Ok(settings)
        } else {
            let settings = Self::default();
            settings.save()?;
            Ok(settings)
        }
    }

    /// Save settings to disk
    pub fn save(&self) -> Result<()> {
        self.validate()?;
        
        let path = Self::settings_path()?;
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| LauncherError::SettingsError(format!("Failed to create settings directory: {}", e)))?;
        }
        
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| LauncherError::SettingsError(format!("Failed to serialize settings: {}", e)))?;
        
        fs::write(&path, contents)
            .map_err(|e| LauncherError::SettingsError(format!("Failed to write settings: {}", e)))?;
        
        Ok(())
    }

    /// Validate settings
    pub fn validate(&self) -> Result<()> {
        if self.hotkey.is_empty() {
            return Err(LauncherError::ConfigError("Hotkey cannot be empty".to_string()));
        }
        
        if self.max_results == 0 || self.max_results > 50 {
            return Err(LauncherError::ConfigError("Max results must be between 1 and 50".to_string()));
        }
        
        if self.search_delay > 1000 {
            return Err(LauncherError::ConfigError("Search delay must be less than 1000ms".to_string()));
        }
        
        Ok(())
    }

    /// Get the path to the settings file
    fn settings_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let app_data = std::env::var("APPDATA")
                .map_err(|_| LauncherError::SettingsError("APPDATA environment variable not found".to_string()))?;
            
            let mut path = PathBuf::from(app_data);
            path.push("BetterFinder");
            path.push("settings.json");
            
            Ok(path)
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // For Linux/Mac, use XDG_CONFIG_HOME or ~/.config
            let home = std::env::var("HOME")
                .map_err(|_| LauncherError::SettingsError("HOME environment variable not found".to_string()))?;
            
            let config_dir = std::env::var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| format!("{}/.config", home));
            
            let mut path = PathBuf::from(config_dir);
            path.push("better-finder");
            path.push("settings.json");
            
            Ok(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.hotkey, "Ctrl+K");
        assert_eq!(settings.max_results, 8);
        assert_eq!(settings.search_delay, 150);
        assert!(settings.enabled_providers.files);
    }

    #[test]
    fn test_settings_validation() {
        let mut settings = AppSettings::default();
        assert!(settings.validate().is_ok());

        settings.hotkey = String::new();
        assert!(settings.validate().is_err());

        settings.hotkey = "Ctrl+K".to_string();
        settings.max_results = 0;
        assert!(settings.validate().is_err());

        settings.max_results = 100;
        assert!(settings.validate().is_err());

        settings.max_results = 8;
        settings.search_delay = 2000;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        
        assert_eq!(settings.hotkey, deserialized.hotkey);
        assert_eq!(settings.max_results, deserialized.max_results);
    }
}
