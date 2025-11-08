use thiserror::Error;

/// Main error type for the launcher application
#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("Failed to register global hotkey: {0}")]
    HotkeyRegistrationError(String),

    #[error("Everything SDK not found or not running")]
    EverythingNotAvailable,

    #[error("Failed to execute search: {0}")]
    SearchError(String),

    #[error("Failed to execute result action: {0}")]
    ExecutionError(String),

    #[error("Settings error: {0}")]
    SettingsError(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Security error: {0}")]
    SecurityError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Tray error: {0}")]
    TrayError(String),

    #[error("Window error: {0}")]
    WindowError(String),
}

/// Result type alias for launcher operations
pub type Result<T> = std::result::Result<T, LauncherError>;

/// Convert LauncherError to a string for Tauri commands
impl From<LauncherError> for String {
    fn from(error: LauncherError) -> Self {
        error.to_string()
    }
}
