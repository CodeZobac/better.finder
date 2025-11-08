// Core modules
pub mod error;
pub mod types;
pub mod settings;
pub mod search;
pub mod utils;
pub mod hotkey;
pub mod tray;
pub mod autostart;
pub mod updater;

use settings::AppSettings;
use hotkey::GlobalHotkeyManager;
use search::{SearchEngine, SearchProvider};
use types::SearchResult;
use std::sync::Arc;
use tauri::{Manager, Emitter};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Tauri command to register a new global hotkey
#[tauri::command]
fn register_hotkey(
    hotkey_manager: tauri::State<Arc<GlobalHotkeyManager>>,
    shortcut: String,
) -> Result<(), String> {
    hotkey_manager
        .register_hotkey(&shortcut)
        .map_err(|e| e.to_string())
}

/// Tauri command to unregister a global hotkey
#[tauri::command]
fn unregister_hotkey(
    hotkey_manager: tauri::State<Arc<GlobalHotkeyManager>>,
    shortcut: String,
) -> Result<(), String> {
    hotkey_manager
        .unregister_hotkey(&shortcut)
        .map_err(|e| e.to_string())
}

/// Tauri command to get all registered hotkeys
#[tauri::command]
fn get_registered_hotkeys(
    hotkey_manager: tauri::State<Arc<GlobalHotkeyManager>>,
) -> Result<Vec<String>, String> {
    hotkey_manager
        .get_registered_shortcuts()
        .map_err(|e| e.to_string())
}

/// Tauri command to show the main window
#[tauri::command]
fn show_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        window.center().map_err(|e| e.to_string())?;
        tracing::info!("Window shown and centered");
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

/// Tauri command to hide the main window
#[tauri::command]
fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
        tracing::info!("Window hidden");
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

/// Tauri command to perform a search query
#[tauri::command]
async fn search_query(
    search_engine: tauri::State<'_, Arc<SearchEngine>>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    tracing::debug!("Search command received: '{}'", query);
    
    let results = search_engine.search(&query).await;
    Ok(results)
}

/// Tauri command to execute a search result action
#[tauri::command]
async fn execute_result(
    search_engine: tauri::State<'_, Arc<SearchEngine>>,
    result: SearchResult,
) -> Result<(), String> {
    tracing::info!("Execute result command received: {}", result.title);
    
    search_engine
        .execute_result(&result)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri command to get current settings
#[tauri::command]
fn get_settings() -> Result<AppSettings, String> {
    tracing::debug!("Get settings command received");
    
    AppSettings::load()
        .map_err(|e| e.to_string())
}

/// Tauri command to get the resolved theme (resolves 'system' to actual theme)
#[tauri::command]
fn get_resolved_theme() -> Result<settings::Theme, String> {
    tracing::debug!("Get resolved theme command received");
    
    let settings = AppSettings::load().map_err(|e| e.to_string())?;
    utils::theme::resolve_theme(settings.theme)
        .map_err(|e| e.to_string())
}

/// Tauri command to update settings
#[tauri::command]
async fn update_settings(
    app: tauri::AppHandle,
    hotkey_manager: tauri::State<'_, Arc<GlobalHotkeyManager>>,
    settings: AppSettings,
) -> Result<(), String> {
    tracing::info!("Update settings command received");
    
    // Validate settings before applying
    settings.validate().map_err(|e| e.to_string())?;
    
    // Load current settings to compare
    let current_settings = AppSettings::load().map_err(|e| e.to_string())?;
    
    // If hotkey changed, re-register it
    if settings.hotkey != current_settings.hotkey {
        tracing::info!("Hotkey changed from '{}' to '{}'", current_settings.hotkey, settings.hotkey);
        
        // Unregister old hotkey
        if let Err(e) = hotkey_manager.unregister_hotkey(&current_settings.hotkey) {
            tracing::warn!("Failed to unregister old hotkey: {}", e);
        }
        
        // Register new hotkey
        hotkey_manager
            .register_hotkey(&settings.hotkey)
            .map_err(|e| format!("Failed to register new hotkey: {}", e))?;
        
        tracing::info!("Hotkey successfully changed to '{}'", settings.hotkey);
    }
    
    // If theme changed, emit event to frontend
    if settings.theme != current_settings.theme {
        tracing::info!("Theme changed from {:?} to {:?}", current_settings.theme, settings.theme);
        
        if let Err(e) = app.emit("theme-changed", &settings.theme) {
            tracing::warn!("Failed to emit theme-changed event: {}", e);
        }
    }
    
    // If start_with_windows changed, update registry
    if settings.start_with_windows != current_settings.start_with_windows {
        tracing::info!("Auto-start changed from {} to {}", 
            current_settings.start_with_windows, settings.start_with_windows);
        
        if settings.start_with_windows {
            autostart::enable_auto_start()
                .map_err(|e| format!("Failed to enable auto-start: {}", e))?;
        } else {
            autostart::disable_auto_start()
                .map_err(|e| format!("Failed to disable auto-start: {}", e))?;
        }
    }
    
    // Save settings to disk
    settings.save().map_err(|e| e.to_string())?;
    
    tracing::info!("Settings updated successfully");
    Ok(())
}

/// Tauri command to check if auto-start is enabled
#[tauri::command]
fn is_auto_start_enabled() -> Result<bool, String> {
    tracing::debug!("Check auto-start status command received");
    
    autostart::is_auto_start_enabled()
        .map_err(|e| e.to_string())
}

/// Tauri command to enable auto-start
#[tauri::command]
fn enable_auto_start() -> Result<(), String> {
    tracing::info!("Enable auto-start command received");
    
    autostart::enable_auto_start()
        .map_err(|e| e.to_string())
}

/// Tauri command to disable auto-start
#[tauri::command]
fn disable_auto_start() -> Result<(), String> {
    tracing::info!("Disable auto-start command received");
    
    autostart::disable_auto_start()
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    if let Err(e) = utils::init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
    }

    tracing::info!("Starting Better Finder application");

    // Load settings
    let settings = match AppSettings::load() {
        Ok(s) => {
            tracing::info!("Settings loaded successfully");
            s
        }
        Err(e) => {
            tracing::error!("Failed to load settings: {}, using defaults", e);
            AppSettings::default()
        }
    };

    tracing::info!("Settings: hotkey={}, theme={:?}, max_results={}", 
        settings.hotkey, settings.theme, settings.max_results);

    let hotkey = settings.hotkey.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            // Initialize global hotkey manager
            let hotkey_manager = GlobalHotkeyManager::new(app.handle().clone());
            
            // Register the default hotkey
            if let Err(e) = hotkey_manager.register_hotkey(&hotkey) {
                tracing::error!("Failed to register global hotkey '{}': {}", hotkey, e);
                // Continue running even if hotkey registration fails
            } else {
                tracing::info!("Global hotkey '{}' registered successfully", hotkey);
            }

            // Store the hotkey manager in app state for later access
            app.manage(Arc::new(hotkey_manager));

            // Initialize search engine
            let search_engine = Arc::new(SearchEngine::new());
            tracing::info!("Search engine initialized");
            
            // Register providers in background for fast startup
            let search_engine_clone = Arc::clone(&search_engine);
            let app_handle_clone = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let start_time = std::time::Instant::now();
                tracing::info!("Starting provider registration...");
                
                // Phase 1: Register critical providers immediately (Calculator, QuickAction, WebSearch)
                // These are lightweight and don't require initialization
                
                // Register CalculatorProvider (instant, no initialization needed)
                if let Ok(calculator_provider) = search::providers::CalculatorProvider::new() {
                    search_engine_clone.register_provider(Box::new(calculator_provider)).await;
                    tracing::info!("CalculatorProvider registered");
                } else {
                    tracing::error!("Failed to initialize CalculatorProvider");
                }
                
                // Register QuickActionProvider (instant, no initialization needed)
                if let Ok(quick_action_provider) = search::providers::QuickActionProvider::new() {
                    search_engine_clone.register_provider(Box::new(quick_action_provider)).await;
                    tracing::info!("QuickActionProvider registered");
                } else {
                    tracing::error!("Failed to initialize QuickActionProvider");
                }
                
                // Register WebSearchProvider (instant, no initialization needed)
                if let Ok(web_search_provider) = search::providers::WebSearchProvider::new() {
                    search_engine_clone.register_provider(Box::new(web_search_provider)).await;
                    tracing::info!("WebSearchProvider registered");
                } else {
                    tracing::error!("Failed to initialize WebSearchProvider");
                }
                
                tracing::info!("Phase 1 complete: Critical providers registered in {:.2}ms", start_time.elapsed().as_millis());
                
                // Phase 2: Register providers that require initialization
                // Register RecentFilesProvider (high priority)
                let recent_files_provider = match search::providers::RecentFilesProvider::new() {
                    Ok(mut provider) => {
                        // Initialize the provider
                        if let Err(e) = provider.initialize().await {
                            tracing::error!("Failed to initialize RecentFilesProvider: {}", e);
                        }
                        Some(Arc::new(tokio::sync::RwLock::new(provider)))
                    }
                    Err(e) => {
                        tracing::error!("Failed to create RecentFilesProvider: {}", e);
                        None
                    }
                };

                // Set up file access tracker if RecentFilesProvider was created
                if let Some(ref recent_provider) = recent_files_provider {
                    let provider_clone = Arc::clone(recent_provider);
                    search_engine_clone.set_file_access_tracker(move |path: &str| {
                        let provider = Arc::clone(&provider_clone);
                        let path_owned = path.to_string();
                        tokio::spawn(async move {
                            let provider_lock = provider.read().await;
                            if let Err(e) = provider_lock.track_file_access(std::path::Path::new(&path_owned)).await {
                                tracing::error!("Failed to track file access: {}", e);
                            }
                        });
                    }).await;
                    tracing::info!("File access tracker registered");
                }

                // Register the RecentFilesProvider
                if let Some(_recent_provider) = recent_files_provider {
                    // We need to create a new instance to register
                    // The original is kept for file access tracking
                    if let Ok(provider_instance) = search::providers::RecentFilesProvider::new() {
                        search_engine_clone.register_provider(Box::new(provider_instance)).await;
                        tracing::info!("RecentFilesProvider registered");
                    }
                }
                
                // Register FileSearchProvider (Everything SDK) with fallback to Windows Search
                match search::providers::FileSearchProvider::new() {
                    Ok(file_provider) => {
                        if file_provider.is_enabled() {
                            search_engine_clone.register_provider(Box::new(file_provider)).await;
                            tracing::info!("FileSearchProvider (Everything SDK) registered");
                        } else {
                            tracing::warn!("Everything SDK not available, registering Windows Search fallback");
                            utils::notify_warning(
                                &app_handle_clone,
                                "File Search Limited",
                                Some("Everything SDK not found. Using Windows Search as fallback. Install Everything for faster file search.")
                            );
                            
                            // Register Windows Search as fallback
                            if let Ok(windows_search_provider) = search::providers::WindowsSearchProvider::new() {
                                search_engine_clone.register_provider(Box::new(windows_search_provider)).await;
                                tracing::info!("WindowsSearchProvider registered as fallback");
                            } else {
                                tracing::error!("Failed to initialize WindowsSearchProvider fallback");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create FileSearchProvider: {}", e);
                        tracing::warn!("Registering Windows Search fallback");
                        utils::notify_warning(
                            &app_handle_clone,
                            "File Search Limited",
                            Some("File search provider initialization failed. Using Windows Search as fallback.")
                        );
                        
                        // Register Windows Search as fallback
                        if let Ok(windows_search_provider) = search::providers::WindowsSearchProvider::new() {
                            search_engine_clone.register_provider(Box::new(windows_search_provider)).await;
                            tracing::info!("WindowsSearchProvider registered as fallback");
                        } else {
                            tracing::error!("Failed to initialize WindowsSearchProvider fallback");
                        }
                    }
                }
                
                // Register AppSearchProvider
                match search::providers::AppSearchProvider::new() {
                    Ok(mut app_provider) => {
                        // Initialize the provider (scans for applications)
                        if let Err(e) = app_provider.initialize().await {
                            tracing::error!("Failed to initialize AppSearchProvider: {}", e);
                        } else {
                            search_engine_clone.register_provider(Box::new(app_provider)).await;
                            tracing::info!("AppSearchProvider registered and initialized");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create AppSearchProvider: {}", e);
                    }
                }
                
                // Register BookmarkProvider
                match search::providers::BookmarkProvider::new() {
                    Ok(mut bookmark_provider) => {
                        // Initialize the provider (loads bookmarks from browsers)
                        if let Err(e) = bookmark_provider.initialize().await {
                            tracing::error!("Failed to initialize BookmarkProvider: {}", e);
                        } else {
                            search_engine_clone.register_provider(Box::new(bookmark_provider)).await;
                            tracing::info!("BookmarkProvider registered and initialized");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create BookmarkProvider: {}", e);
                    }
                }
                
                // Register ClipboardHistoryProvider
                match search::providers::ClipboardHistoryProvider::new() {
                    Ok(mut clipboard_provider) => {
                        // Initialize the provider (starts clipboard monitoring)
                        if let Err(e) = clipboard_provider.initialize().await {
                            tracing::error!("Failed to initialize ClipboardHistoryProvider: {}", e);
                        } else {
                            search_engine_clone.register_provider(Box::new(clipboard_provider)).await;
                            tracing::info!("ClipboardHistoryProvider registered and initialized");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create ClipboardHistoryProvider: {}", e);
                    }
                }
                
                // Log final provider count and startup time
                let provider_count = search_engine_clone.provider_count().await;
                let provider_names = search_engine_clone.provider_names().await;
                let elapsed = start_time.elapsed();
                tracing::info!(
                    "Search engine initialized with {} providers in {:.2}s: {:?}", 
                    provider_count, 
                    elapsed.as_secs_f64(),
                    provider_names
                );
                
                // Defer non-critical background tasks
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                tracing::info!("Starting deferred background tasks...");
                
                // Background tasks can be added here (e.g., cache warming, index updates)
            });
            
            // Store the search engine in app state
            app.manage(search_engine);

            // Initialize system tray
            if let Err(e) = tray::init_tray(app.handle()) {
                tracing::error!("Failed to initialize system tray: {}", e);
                // Continue running even if tray initialization fails
            } else {
                tracing::info!("System tray initialized successfully");
            }

            // Initialize updater (checks for updates after 5 seconds)
            // Spawn in a separate task to avoid blocking setup
            let app_handle_for_updater = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                updater::init_updater(app_handle_for_updater);
            });
            tracing::info!("Updater initialization scheduled");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            register_hotkey,
            unregister_hotkey,
            get_registered_hotkeys,
            show_window,
            hide_window,
            search_query,
            execute_result,
            get_settings,
            update_settings,
            get_resolved_theme,
            is_auto_start_enabled,
            enable_auto_start,
            disable_auto_start,
            updater::check_for_updates_manual
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
