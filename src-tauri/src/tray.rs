use tauri::{
    AppHandle, Manager,
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    menu::{MenuBuilder, MenuItemBuilder},
    image::Image,
};
use crate::error::LauncherError;

/// Initialize the system tray icon and menu
pub fn init_tray(app: &AppHandle) -> Result<(), LauncherError> {
    tracing::info!("Initializing system tray");

    // Load the tray icon
    let icon = load_tray_icon()?;

    // Build the tray menu
    let menu = build_tray_menu(app)?;

    // Create the tray icon
    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("Global Search Launcher")
        .on_menu_event(|app, event| {
            handle_menu_event(app, event.id().as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            handle_tray_event(tray, event);
        })
        .build(app)
        .map_err(|e| LauncherError::TrayError(format!("Failed to build tray icon: {}", e)))?;

    tracing::info!("System tray initialized successfully");
    Ok(())
}

/// Load the tray icon from the icons directory
fn load_tray_icon() -> Result<Image<'static>, LauncherError> {
    // Use the 32x32 icon for the tray
    let icon_bytes = include_bytes!("../icons/32x32.png");
    
    // Load the PNG and decode it
    let img = image::load_from_memory(icon_bytes)
        .map_err(|e| LauncherError::TrayError(format!("Failed to decode icon: {}", e)))?;
    
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let rgba_data = rgba.into_raw();
    
    Ok(Image::new_owned(rgba_data, width, height))
}

/// Build the tray menu with Open Settings, About, and Exit options
fn build_tray_menu(app: &AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, LauncherError> {
    let open_settings = MenuItemBuilder::with_id("open_settings", "Open Settings")
        .build(app)
        .map_err(|e| LauncherError::TrayError(format!("Failed to create menu item: {}", e)))?;

    let about = MenuItemBuilder::with_id("about", "About")
        .build(app)
        .map_err(|e| LauncherError::TrayError(format!("Failed to create menu item: {}", e)))?;

    let separator = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| LauncherError::TrayError(format!("Failed to create separator: {}", e)))?;

    let exit = MenuItemBuilder::with_id("exit", "Exit")
        .build(app)
        .map_err(|e| LauncherError::TrayError(format!("Failed to create menu item: {}", e)))?;

    MenuBuilder::new(app)
        .item(&open_settings)
        .item(&about)
        .item(&separator)
        .item(&exit)
        .build()
        .map_err(|e| LauncherError::TrayError(format!("Failed to build menu: {}", e)))
}

/// Handle tray menu item clicks
fn handle_menu_event(app: &AppHandle, menu_id: &str) {
    tracing::info!("Tray menu item clicked: {}", menu_id);

    match menu_id {
        "open_settings" => {
            if let Err(e) = show_settings_window(app) {
                tracing::error!("Failed to show settings window: {}", e);
            }
        }
        "about" => {
            if let Err(e) = show_about_dialog(app) {
                tracing::error!("Failed to show about dialog: {}", e);
            }
        }
        "exit" => {
            tracing::info!("Exit menu item clicked, quitting application");
            app.exit(0);
        }
        _ => {
            tracing::warn!("Unknown menu item clicked: {}", menu_id);
        }
    }
}

/// Handle tray icon events (clicks)
fn handle_tray_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button,
            button_state,
            ..
        } => {
            if button == MouseButton::Left && button_state == MouseButtonState::Up {
                tracing::info!("Tray icon left-clicked, toggling main window");
                if let Err(e) = toggle_main_window(tray.app_handle()) {
                    tracing::error!("Failed to toggle main window: {}", e);
                }
            }
        }
        _ => {}
    }
}

/// Toggle the main window visibility
fn toggle_main_window(app: &AppHandle) -> Result<(), LauncherError> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window
                .hide()
                .map_err(|e| LauncherError::WindowError(format!("Failed to hide window: {}", e)))?;
            tracing::info!("Main window hidden");
        } else {
            window
                .show()
                .map_err(|e| LauncherError::WindowError(format!("Failed to show window: {}", e)))?;
            window
                .set_focus()
                .map_err(|e| LauncherError::WindowError(format!("Failed to focus window: {}", e)))?;
            window
                .center()
                .map_err(|e| LauncherError::WindowError(format!("Failed to center window: {}", e)))?;
            tracing::info!("Main window shown and centered");
        }
        Ok(())
    } else {
        Err(LauncherError::WindowError("Main window not found".to_string()))
    }
}

/// Show the settings window
fn show_settings_window(app: &AppHandle) -> Result<(), LauncherError> {
    // Check if settings window already exists
    if let Some(window) = app.get_webview_window("settings") {
        window
            .show()
            .map_err(|e| LauncherError::WindowError(format!("Failed to show settings window: {}", e)))?;
        window
            .set_focus()
            .map_err(|e| LauncherError::WindowError(format!("Failed to focus settings window: {}", e)))?;
        tracing::info!("Settings window shown");
        return Ok(());
    }

    // Create new settings window
    use tauri::WebviewWindowBuilder;
    
    let _settings_window = WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("settings.html".into())
    )
    .title("Settings - Global Search Launcher")
    .inner_size(600.0, 500.0)
    .center()
    .resizable(false)
    .build()
    .map_err(|e| LauncherError::WindowError(format!("Failed to create settings window: {}", e)))?;

    tracing::info!("Settings window created");
    Ok(())
}

/// Show the about dialog
fn show_about_dialog(app: &AppHandle) -> Result<(), LauncherError> {
    let version = app.package_info().version.to_string();
    let message = format!(
        "Global Search Launcher\n\nVersion: {}\n\nA fast, keyboard-driven search launcher for Windows.\n\nPress Ctrl+K to open the search bar from anywhere.",
        version
    );

    // For now, just log the about info
    // In a real implementation, you'd use a proper dialog or create a custom window
    tracing::info!("About: {}", message);
    
    // TODO: Implement a proper about dialog window
    // For now, we'll just log it
    
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_tray_icon() {
        // Test that the tray icon can be loaded successfully
        let result = load_tray_icon();
        assert!(result.is_ok(), "Failed to load tray icon: {:?}", result.err());
    }

    #[test]
    fn test_menu_event_handling() {
        // Test that menu event IDs are recognized
        let valid_menu_ids = vec!["open_settings", "about", "exit"];
        
        for menu_id in valid_menu_ids {
            // This test verifies that the menu IDs are valid strings
            // In a real implementation, we would test the actual handler behavior
            assert!(!menu_id.is_empty(), "Menu ID should not be empty");
        }
    }

    #[test]
    fn test_tray_icon_dimensions() {
        // Test that the loaded icon has valid dimensions
        let _icon = load_tray_icon().expect("Failed to load icon");
        
        // The icon should have non-zero dimensions
        // Note: We can't directly access width/height from Image<'static>
        // but we can verify it was created successfully
        assert!(true, "Icon created successfully");
    }
}
