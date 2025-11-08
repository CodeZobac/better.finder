use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a search result from any provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Unique identifier for the result
    pub id: String,
    /// Primary display text
    pub title: String,
    /// Secondary display text (e.g., file path, URL)
    pub subtitle: String,
    /// Base64 encoded icon or icon name
    pub icon: Option<String>,
    /// Type of result
    #[serde(rename = "type")]
    pub result_type: ResultType,
    /// Relevance score (higher is better)
    pub score: f64,
    /// Additional metadata specific to result type
    pub metadata: HashMap<String, serde_json::Value>,
    /// Action to execute when result is selected
    pub action: ResultAction,
}

/// Types of search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultType {
    File,
    Application,
    QuickAction,
    Calculator,
    Clipboard,
    Bookmark,
    RecentFile,
    WebSearch,
}

/// Action to perform when a result is executed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResultAction {
    OpenFile { path: String },
    LaunchApp { path: String },
    ExecuteCommand { command: String, args: Vec<String> },
    CopyToClipboard { content: String },
    OpenUrl { url: String },
    WebSearch { query: String },
}
