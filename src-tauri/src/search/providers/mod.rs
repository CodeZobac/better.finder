pub mod everything;
pub mod file_search;
pub mod windows_search;
pub mod app_search;
pub mod quick_action;
pub mod calculator;
pub mod clipboard;
pub mod bookmark;
pub mod recent_files;
pub mod web_search;

#[cfg(test)]
mod fallback_test;

pub use file_search::FileSearchProvider;
pub use windows_search::WindowsSearchProvider;
pub use app_search::AppSearchProvider;
pub use quick_action::QuickActionProvider;
pub use calculator::CalculatorProvider;
pub use clipboard::ClipboardHistoryProvider;
pub use bookmark::BookmarkProvider;
pub use recent_files::RecentFilesProvider;
pub use web_search::WebSearchProvider;
