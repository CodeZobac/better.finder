use crate::error::{LauncherError, Result};
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, RegQueryValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ, REG_VALUE_TYPE,
};


const REGISTRY_RUN_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "BetterFinder";

/// Enable auto-start by adding a registry entry
#[cfg(target_os = "windows")]
pub fn enable_auto_start() -> Result<()> {
    use windows::core::HSTRING;
    
    let exe_path = get_executable_path()?;
    
    tracing::info!("Enabling auto-start with path: {}", exe_path.display());
    
    unsafe {
        let mut hkey: HKEY = HKEY::default();
        
        // Open the registry key
        let key_name = HSTRING::from(REGISTRY_RUN_PATH);
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &key_name,
            0,
            KEY_WRITE,
            &mut hkey,
        );
        
        if result.is_err() {
            return Err(LauncherError::SettingsError(
                format!("Failed to open registry key: {:?}", result.0)
            ));
        }
        
        // Set the registry value
        let value_name = HSTRING::from(APP_NAME);
        let exe_path_str = exe_path.to_string_lossy().to_string();
        let exe_path_wide: Vec<u16> = exe_path_str.encode_utf16().chain(std::iter::once(0)).collect();
        
        let result = RegSetValueExW(
            hkey,
            &value_name,
            0,
            REG_SZ,
            Some(&exe_path_wide.as_slice().align_to::<u8>().1),
        );
        
        RegCloseKey(hkey).ok();
        
        if result.is_err() {
            return Err(LauncherError::SettingsError(
                format!("Failed to set registry value: {:?}", result.0)
            ));
        }
    }
    
    tracing::info!("Auto-start enabled successfully");
    Ok(())
}

/// Disable auto-start by removing the registry entry
#[cfg(target_os = "windows")]
pub fn disable_auto_start() -> Result<()> {
    use windows::core::HSTRING;
    
    tracing::info!("Disabling auto-start");
    
    unsafe {
        let mut hkey: HKEY = HKEY::default();
        
        // Open the registry key
        let key_name = HSTRING::from(REGISTRY_RUN_PATH);
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &key_name,
            0,
            KEY_WRITE,
            &mut hkey,
        );
        
        if result.is_err() {
            return Err(LauncherError::SettingsError(
                format!("Failed to open registry key: {:?}", result.0)
            ));
        }
        
        // Delete the registry value
        let value_name = HSTRING::from(APP_NAME);
        let result = RegDeleteValueW(hkey, &value_name);
        
        RegCloseKey(hkey).ok();
        
        if result.is_err() {
            // If the value doesn't exist, that's fine
            tracing::debug!("Registry value may not exist: {:?}", result.0);
        }
    }
    
    tracing::info!("Auto-start disabled successfully");
    Ok(())
}

/// Check if auto-start is currently enabled
#[cfg(target_os = "windows")]
pub fn is_auto_start_enabled() -> Result<bool> {
    use windows::core::HSTRING;
    
    unsafe {
        let mut hkey: HKEY = HKEY::default();
        
        // Open the registry key
        let key_name = HSTRING::from(REGISTRY_RUN_PATH);
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &key_name,
            0,
            KEY_READ,
            &mut hkey,
        );
        
        if result.is_err() {
            return Ok(false);
        }
        
        // Query the registry value
        let value_name = HSTRING::from(APP_NAME);
        let mut buffer: Vec<u8> = vec![0; 512];
        let mut buffer_size: u32 = buffer.len() as u32;
        let mut value_type = REG_VALUE_TYPE::default();
        
        let result = RegQueryValueExW(
            hkey,
            &value_name,
            None,
            Some(&mut value_type),
            Some(buffer.as_mut_ptr()),
            Some(&mut buffer_size),
        );
        
        RegCloseKey(hkey).ok();
        
        if result.is_ok() {
            // Value exists, check if it matches our executable path
            let exe_path = get_executable_path()?;
            let exe_path_str = exe_path.to_string_lossy().to_string();
            
            // Convert buffer to string
            let value_str = String::from_utf16_lossy(
                &buffer[..buffer_size as usize]
                    .chunks_exact(2)
                    .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
                    .take_while(|&c| c != 0)
                    .collect::<Vec<u16>>()
            );
            
            tracing::debug!("Registry value: {}", value_str);
            tracing::debug!("Current exe path: {}", exe_path_str);
            
            // Check if the paths match (case-insensitive on Windows)
            Ok(value_str.to_lowercase() == exe_path_str.to_lowercase())
        } else {
            Ok(false)
        }
    }
}

/// Get the path to the current executable
fn get_executable_path() -> Result<PathBuf> {
    std::env::current_exe()
        .map_err(|e| LauncherError::SettingsError(
            format!("Failed to get executable path: {}", e)
        ))
}

// Non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn enable_auto_start() -> Result<()> {
    Err(LauncherError::SettingsError(
        "Auto-start is only supported on Windows".to_string()
    ))
}

#[cfg(not(target_os = "windows"))]
pub fn disable_auto_start() -> Result<()> {
    Err(LauncherError::SettingsError(
        "Auto-start is only supported on Windows".to_string()
    ))
}

#[cfg(not(target_os = "windows"))]
pub fn is_auto_start_enabled() -> Result<bool> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_executable_path() {
        let path = get_executable_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.exists());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_auto_start_enable_disable() {
        // Test enabling auto-start
        let result = enable_auto_start();
        assert!(result.is_ok(), "Failed to enable auto-start: {:?}", result);

        // Check if it's enabled
        let is_enabled = is_auto_start_enabled();
        assert!(is_enabled.is_ok());
        assert!(is_enabled.unwrap(), "Auto-start should be enabled");

        // Test disabling auto-start
        let result = disable_auto_start();
        assert!(result.is_ok(), "Failed to disable auto-start: {:?}", result);

        // Check if it's disabled
        let is_enabled = is_auto_start_enabled();
        assert!(is_enabled.is_ok());
        assert!(!is_enabled.unwrap(), "Auto-start should be disabled");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_is_auto_start_enabled_when_not_set() {
        // First ensure it's disabled
        let _ = disable_auto_start();

        // Check status
        let is_enabled = is_auto_start_enabled();
        assert!(is_enabled.is_ok());
        assert!(!is_enabled.unwrap(), "Auto-start should not be enabled initially");
    }
}
