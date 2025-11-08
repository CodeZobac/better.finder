use crate::error::Result;
use crate::settings::Theme;

#[cfg(target_os = "windows")]
use windows::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_VALUE_TYPE};
#[cfg(target_os = "windows")]
use windows::core::PCWSTR;

/// Detect the current Windows system theme
#[cfg(target_os = "windows")]
pub fn detect_system_theme() -> Result<Theme> {
    use std::ptr;
    
    unsafe {
        let key_path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
            .encode_utf16()
            .collect();
        
        let value_name: Vec<u16> = "AppsUseLightTheme\0"
            .encode_utf16()
            .collect();
        
        let mut h_key: HKEY = HKEY::default();
        
        // Open registry key
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path.as_ptr()),
            0,
            KEY_READ,
            &mut h_key,
        );
        
        if result.is_err() {
            tracing::warn!("Failed to open registry key for theme detection, defaulting to dark theme");
            return Ok(Theme::Dark);
        }
        
        // Query the value
        let mut data: u32 = 0;
        let mut data_size: u32 = std::mem::size_of::<u32>() as u32;
        let mut value_type: REG_VALUE_TYPE = REG_VALUE_TYPE::default();
        
        let result = RegQueryValueExW(
            h_key,
            PCWSTR(value_name.as_ptr()),
            Some(ptr::null_mut()),
            Some(&mut value_type),
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut data_size),
        );
        
        if result.is_err() {
            tracing::warn!("Failed to query registry value for theme detection, defaulting to dark theme");
            return Ok(Theme::Dark);
        }
        
        // 0 = Dark theme, 1 = Light theme
        Ok(if data == 0 { Theme::Dark } else { Theme::Light })
    }
}

#[cfg(not(target_os = "windows"))]
pub fn detect_system_theme() -> Result<Theme> {
    // Default to dark theme on non-Windows platforms
    Ok(Theme::Dark)
}

/// Resolve the actual theme to use based on settings
pub fn resolve_theme(theme_setting: Theme) -> Result<Theme> {
    match theme_setting {
        Theme::System => detect_system_theme(),
        other => Ok(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_theme_explicit() {
        assert_eq!(resolve_theme(Theme::Light).unwrap(), Theme::Light);
        assert_eq!(resolve_theme(Theme::Dark).unwrap(), Theme::Dark);
    }

    #[test]
    fn test_resolve_theme_system() {
        // Should not panic and return a valid theme
        let theme = resolve_theme(Theme::System);
        assert!(theme.is_ok());
        let theme = theme.unwrap();
        assert!(theme == Theme::Light || theme == Theme::Dark);
    }

    #[test]
    fn test_detect_system_theme() {
        // Should not panic and return a valid theme
        let theme = detect_system_theme();
        assert!(theme.is_ok());
    }
}
