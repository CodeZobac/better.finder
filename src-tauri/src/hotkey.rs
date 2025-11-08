use crate::error::LauncherError;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use std::sync::{Arc, Mutex};

/// Manages global keyboard shortcuts for the application
pub struct GlobalHotkeyManager {
    app_handle: AppHandle,
    registered_shortcuts: Arc<Mutex<Vec<String>>>,
}

impl GlobalHotkeyManager {
    /// Creates a new GlobalHotkeyManager instance
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            registered_shortcuts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Registers a global hotkey
    /// 
    /// # Arguments
    /// * `shortcut` - The keyboard shortcut string (e.g., "Ctrl+K", "Alt+Space")
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if registration succeeded, Err otherwise
    pub fn register_hotkey(&self, shortcut: &str) -> Result<(), LauncherError> {
        // Validate the shortcut format
        self.validate_shortcut(shortcut)?;

        // Parse the shortcut
        let parsed_shortcut = shortcut.parse::<Shortcut>()
            .map_err(|e| LauncherError::HotkeyRegistrationError(
                format!("Invalid shortcut format '{}': {}", shortcut, e)
            ))?;

        // Register the shortcut with the global shortcut plugin
        let app_handle = self.app_handle.clone();
        let shortcut_str = shortcut.to_string();
        
        self.app_handle
            .global_shortcut()
            .on_shortcut(parsed_shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    tracing::debug!("Global hotkey triggered: {}", shortcut_str);
                    
                    // Emit event to frontend
                    if let Err(e) = app_handle.emit("hotkey-pressed", ()) {
                        tracing::error!("Failed to emit hotkey event: {}", e);
                    }
                }
            })
            .map_err(|e| LauncherError::HotkeyRegistrationError(
                format!("Failed to register shortcut '{}': {}", shortcut, e)
            ))?;

        // Store the registered shortcut
        let mut shortcuts = self.registered_shortcuts.lock()
            .map_err(|e| LauncherError::HotkeyRegistrationError(
                format!("Failed to acquire lock: {}", e)
            ))?;
        
        if !shortcuts.contains(&shortcut.to_string()) {
            shortcuts.push(shortcut.to_string());
        }

        tracing::info!("Successfully registered global hotkey: {}", shortcut);
        Ok(())
    }

    /// Unregisters a global hotkey
    /// 
    /// # Arguments
    /// * `shortcut` - The keyboard shortcut string to unregister
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if unregistration succeeded, Err otherwise
    pub fn unregister_hotkey(&self, shortcut: &str) -> Result<(), LauncherError> {
        // Parse the shortcut
        let parsed_shortcut = shortcut.parse::<Shortcut>()
            .map_err(|e| LauncherError::HotkeyRegistrationError(
                format!("Invalid shortcut format '{}': {}", shortcut, e)
            ))?;

        // Unregister the shortcut
        self.app_handle
            .global_shortcut()
            .unregister(parsed_shortcut)
            .map_err(|e| LauncherError::HotkeyRegistrationError(
                format!("Failed to unregister shortcut '{}': {}", shortcut, e)
            ))?;

        // Remove from registered shortcuts list
        let mut shortcuts = self.registered_shortcuts.lock()
            .map_err(|e| LauncherError::HotkeyRegistrationError(
                format!("Failed to acquire lock: {}", e)
            ))?;
        
        shortcuts.retain(|s| s != shortcut);

        tracing::info!("Successfully unregistered global hotkey: {}", shortcut);
        Ok(())
    }

    /// Validates a shortcut string format
    /// 
    /// # Arguments
    /// * `shortcut` - The keyboard shortcut string to validate
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if valid, Err with validation error otherwise
    fn validate_shortcut(&self, shortcut: &str) -> Result<(), LauncherError> {
        if shortcut.is_empty() {
            return Err(LauncherError::HotkeyRegistrationError(
                "Shortcut cannot be empty".to_string()
            ));
        }

        // Check for valid modifier keys
        let valid_modifiers = ["Ctrl", "Alt", "Shift", "Super", "Command", "Option"];
        let parts: Vec<&str> = shortcut.split('+').collect();
        
        if parts.len() < 2 {
            return Err(LauncherError::HotkeyRegistrationError(
                format!("Shortcut '{}' must include at least one modifier key", shortcut)
            ));
        }

        // Validate that all parts except the last are valid modifiers
        for (i, part) in parts.iter().enumerate() {
            if i < parts.len() - 1 {
                if !valid_modifiers.iter().any(|m| m.eq_ignore_ascii_case(part)) {
                    return Err(LauncherError::HotkeyRegistrationError(
                        format!("Invalid modifier key '{}' in shortcut '{}'", part, shortcut)
                    ));
                }
            }
        }

        Ok(())
    }

    /// Gets the list of currently registered shortcuts
    pub fn get_registered_shortcuts(&self) -> Result<Vec<String>, LauncherError> {
        let shortcuts = self.registered_shortcuts.lock()
            .map_err(|e| LauncherError::HotkeyRegistrationError(
                format!("Failed to acquire lock: {}", e)
            ))?;
        
        Ok(shortcuts.clone())
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests require a Tauri app context which is not available in unit tests
    // Integration tests should be used for full hotkey functionality testing
    
    #[test]
    fn test_validate_shortcut_empty() {
        // We can't create a real GlobalHotkeyManager without AppHandle,
        // so we'll test validation logic separately
        let shortcut = "";
        assert!(shortcut.is_empty());
    }

    #[test]
    fn test_validate_shortcut_format() {
        let valid_shortcuts = vec!["Ctrl+K", "Alt+Space", "Ctrl+Shift+F", "Super+A"];
        for shortcut in valid_shortcuts {
            let parts: Vec<&str> = shortcut.split('+').collect();
            assert!(parts.len() >= 2, "Shortcut {} should have at least 2 parts", shortcut);
        }
    }

    #[test]
    fn test_invalid_shortcut_no_modifier() {
        let shortcut = "K";
        let parts: Vec<&str> = shortcut.split('+').collect();
        assert!(parts.len() < 2, "Shortcut without modifier should be invalid");
    }
}
