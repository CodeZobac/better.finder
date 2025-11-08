use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;
use tracing::{error, info, warn};

/// Check for updates and prompt user if available
pub async fn check_for_updates(app: AppHandle) {
    info!("Checking for application updates...");
    
    match app.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    info!(
                        "Update available: {} -> {}",
                        update.current_version,
                        update.version
                    );
                    
                    // Show update notification to user
                    if let Err(e) = app.emit("update-available", &update.version) {
                        error!("Failed to emit update-available event: {}", e);
                    }
                    
                    // Download and install the update
                    match update.download_and_install(|chunk_length, content_length| {
                        if let Some(total) = content_length {
                            let progress = (chunk_length as f64 / total as f64) * 100.0;
                            info!("Download progress: {:.2}%", progress);
                        }
                    }, || {
                        info!("Download complete, installing update...");
                    }).await {
                        Ok(_) => {
                            info!("Update installed successfully");
                            // Notify user that update is ready
                            if let Err(e) = app.emit("update-installed", ()) {
                                error!("Failed to emit update-installed event: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to download and install update: {}", e);
                            if let Err(e) = app.emit("update-error", e.to_string()) {
                                error!("Failed to emit update-error event: {}", e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    info!("No updates available");
                }
                Err(e) => {
                    warn!("Failed to check for updates: {}", e);
                    // Don't emit error for update check failures - fail silently
                }
            }
        }
        Err(e) => {
            warn!("Updater not available: {}", e);
        }
    }
}

/// Initialize updater and check for updates on startup
pub fn init_updater(app: AppHandle) {
    // Check for updates 5 seconds after startup to avoid blocking
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        check_for_updates(app).await;
    });
}

#[tauri::command]
pub async fn check_for_updates_manual(app: AppHandle) -> Result<String, String> {
    info!("Manual update check requested");
    
    match app.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    Ok(format!("Update available: {}", update.version))
                }
                Ok(None) => {
                    Ok("No updates available".to_string())
                }
                Err(e) => {
                    Err(format!("Failed to check for updates: {}", e))
                }
            }
        }
        Err(e) => {
            Err(format!("Updater not available: {}", e))
        }
    }
}
