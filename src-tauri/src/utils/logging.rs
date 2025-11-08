use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use std::fs;
use std::path::PathBuf;
use crate::error::Result;

/// Initialize the logging infrastructure with file rotation
pub fn init_logging() -> Result<()> {
    let log_dir = get_log_directory()?;
    
    // Ensure log directory exists
    fs::create_dir_all(&log_dir)?;
    
    // Rotate logs if needed before opening the file
    rotate_logs_if_needed_internal(&log_dir)?;
    
    let log_file = log_dir.join("better-finder.log");
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;

    // Create a file appender
    let file_layer = fmt::layer()
        .with_writer(std::sync::Arc::new(file))
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);

    // Create a stdout layer for development
    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true);

    // Set up the filter with different levels
    // Default to INFO level, but allow override via RUST_LOG env var
    // Supported levels: trace, debug, info, warn, error
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            #[cfg(debug_assertions)]
            {
                EnvFilter::new("debug")
            }
            #[cfg(not(debug_assertions))]
            {
                EnvFilter::new("info")
            }
        });

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    tracing::info!("Logging initialized with file rotation support");
    tracing::debug!("Debug logging enabled");

    Ok(())
}

/// Get the directory where log files should be stored
fn get_log_directory() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let app_data = std::env::var("APPDATA")
            .map_err(|_| crate::error::LauncherError::SettingsError(
                "APPDATA environment variable not found".to_string()
            ))?;
        
        let mut path = PathBuf::from(app_data);
        path.push("BetterFinder");
        path.push("logs");
        
        Ok(path)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For Linux/Mac, use XDG_DATA_HOME or ~/.local/share
        let home = std::env::var("HOME")
            .map_err(|_| crate::error::LauncherError::SettingsError(
                "HOME environment variable not found".to_string()
            ))?;
        
        let data_dir = std::env::var("XDG_DATA_HOME")
            .unwrap_or_else(|_| format!("{}/.local/share", home));
        
        let mut path = PathBuf::from(data_dir);
        path.push("better-finder");
        path.push("logs");
        
        Ok(path)
    }
}

/// Rotate log files if they exceed a certain size (10MB)
/// Keeps up to 5 rotated log files
pub fn rotate_logs_if_needed() -> Result<()> {
    let log_dir = get_log_directory()?;
    rotate_logs_if_needed_internal(&log_dir)
}

/// Internal function to rotate logs
fn rotate_logs_if_needed_internal(log_dir: &PathBuf) -> Result<()> {
    let log_file = log_dir.join("better-finder.log");
    
    if !log_file.exists() {
        return Ok(());
    }
    
    let metadata = fs::metadata(&log_file)?;
    let max_size = 10 * 1024 * 1024; // 10MB
    
    if metadata.len() > max_size {
        // Rotate existing backup files
        // Keep up to 5 rotated files: .log.1, .log.2, .log.3, .log.4, .log.5
        for i in (1..5).rev() {
            let old_log = log_dir.join(format!("better-finder.log.{}", i));
            let new_log = log_dir.join(format!("better-finder.log.{}", i + 1));
            
            if old_log.exists() {
                if new_log.exists() {
                    fs::remove_file(&new_log)?;
                }
                fs::rename(&old_log, &new_log)?;
            }
        }
        
        // Rotate current log to .log.1
        let first_backup = log_dir.join("better-finder.log.1");
        if first_backup.exists() {
            fs::remove_file(&first_backup)?;
        }
        fs::rename(&log_file, &first_backup)?;
        
        // Log rotation will be logged after the new file is created
    }
    
    Ok(())
}

/// Clean up old log files beyond the retention limit
pub fn cleanup_old_logs() -> Result<()> {
    let log_dir = get_log_directory()?;
    cleanup_old_logs_internal(&log_dir)
}

/// Internal function to clean up old logs
pub(crate) fn cleanup_old_logs_internal(log_dir: &PathBuf) -> Result<()> {
    // Remove log files older than .log.5
    for i in 6..=10 {
        let old_log = log_dir.join(format!("better-finder.log.{}", i));
        if old_log.exists() {
            fs::remove_file(&old_log)?;
            tracing::debug!("Removed old log file: better-finder.log.{}", i);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn get_test_log_dir() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("better-finder-test-logs");
        path
    }

    fn cleanup_test_logs() {
        let log_dir = get_test_log_dir();
        if log_dir.exists() {
            let _ = fs::remove_dir_all(&log_dir);
        }
    }

    #[test]
    fn test_log_directory_creation() {
        cleanup_test_logs();
        
        let log_dir = get_test_log_dir();
        assert!(!log_dir.exists());

        // Create the directory
        let result = fs::create_dir_all(&log_dir);
        assert!(result.is_ok());
        assert!(log_dir.exists());

        cleanup_test_logs();
    }

    #[test]
    fn test_log_rotation_when_file_exceeds_size() {
        cleanup_test_logs();
        
        let log_dir = get_test_log_dir();
        fs::create_dir_all(&log_dir).unwrap();

        let log_file = log_dir.join("better-finder.log");
        
        // Create a large log file (> 10MB)
        let large_content = "x".repeat(11 * 1024 * 1024); // 11MB
        fs::write(&log_file, large_content).unwrap();

        // Verify file is large
        let metadata = fs::metadata(&log_file).unwrap();
        assert!(metadata.len() > 10 * 1024 * 1024);

        // Rotate logs
        let result = rotate_logs_if_needed_internal(&log_dir);
        assert!(result.is_ok());

        // Check that the file was rotated
        let backup_file = log_dir.join("better-finder.log.1");
        assert!(backup_file.exists());

        // Original file should not exist or be smaller
        if log_file.exists() {
            let new_metadata = fs::metadata(&log_file).unwrap();
            assert!(new_metadata.len() < metadata.len());
        }

        cleanup_test_logs();
    }

    #[test]
    fn test_log_rotation_keeps_multiple_backups() {
        cleanup_test_logs();
        
        let log_dir = get_test_log_dir();
        fs::create_dir_all(&log_dir).unwrap();

        // Create existing backup files
        for i in 1..=3 {
            let backup = log_dir.join(format!("better-finder.log.{}", i));
            fs::write(&backup, format!("backup {}", i)).unwrap();
        }

        // Create a large current log file
        let log_file = log_dir.join("better-finder.log");
        let large_content = "x".repeat(11 * 1024 * 1024);
        fs::write(&log_file, large_content).unwrap();

        // Rotate logs
        let result = rotate_logs_if_needed_internal(&log_dir);
        assert!(result.is_ok());

        // Check that backups were shifted
        assert!(log_dir.join("better-finder.log.1").exists());
        assert!(log_dir.join("better-finder.log.2").exists());
        assert!(log_dir.join("better-finder.log.3").exists());
        assert!(log_dir.join("better-finder.log.4").exists());

        cleanup_test_logs();
    }

    #[test]
    fn test_log_rotation_does_not_rotate_small_files() {
        cleanup_test_logs();
        
        let log_dir = get_test_log_dir();
        fs::create_dir_all(&log_dir).unwrap();

        let log_file = log_dir.join("better-finder.log");
        
        // Create a small log file (< 10MB)
        let small_content = "small log content";
        fs::write(&log_file, small_content).unwrap();

        // Rotate logs
        let result = rotate_logs_if_needed_internal(&log_dir);
        assert!(result.is_ok());

        // Check that the file was NOT rotated
        let backup_file = log_dir.join("better-finder.log.1");
        assert!(!backup_file.exists());

        // Original file should still exist
        assert!(log_file.exists());

        cleanup_test_logs();
    }

    #[test]
    fn test_cleanup_old_logs() {
        cleanup_test_logs();
        
        let log_dir = get_test_log_dir();
        fs::create_dir_all(&log_dir).unwrap();

        // Create old backup files that should be cleaned up
        for i in 6..=10 {
            let old_backup = log_dir.join(format!("better-finder.log.{}", i));
            fs::write(&old_backup, format!("old backup {}", i)).unwrap();
        }

        // Verify they exist
        assert!(log_dir.join("better-finder.log.6").exists());
        assert!(log_dir.join("better-finder.log.10").exists());

        // Clean up old logs
        let result = cleanup_old_logs_internal(&log_dir);
        assert!(result.is_ok());

        // Verify they were removed
        assert!(!log_dir.join("better-finder.log.6").exists());
        assert!(!log_dir.join("better-finder.log.10").exists());

        cleanup_test_logs();
    }
}
