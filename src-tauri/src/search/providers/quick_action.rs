/// Quick Action provider for system commands
///
/// This provider offers quick access to common system operations like:
/// - Shutdown
/// - Restart
/// - Lock
/// - Sleep
/// - Hibernate
/// - Log Off

use crate::error::{LauncherError, Result};
use crate::search::SearchProvider;
use crate::types::{ResultAction, ResultType, SearchResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

const MAX_RESULTS: usize = 10;

/// System commands that can be executed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemCommand {
    Shutdown,
    Restart,
    Lock,
    Sleep,
    Hibernate,
    LogOff,
}

impl SystemCommand {
    /// Returns the display name for the command
    pub fn display_name(&self) -> &str {
        match self {
            SystemCommand::Shutdown => "Shutdown",
            SystemCommand::Restart => "Restart",
            SystemCommand::Lock => "Lock",
            SystemCommand::Sleep => "Sleep",
            SystemCommand::Hibernate => "Hibernate",
            SystemCommand::LogOff => "Log Off",
        }
    }

    /// Returns the description for the command
    pub fn description(&self) -> &str {
        match self {
            SystemCommand::Shutdown => "Shut down the computer",
            SystemCommand::Restart => "Restart the computer",
            SystemCommand::Lock => "Lock the computer",
            SystemCommand::Sleep => "Put the computer to sleep",
            SystemCommand::Hibernate => "Hibernate the computer",
            SystemCommand::LogOff => "Log off the current user",
        }
    }

    /// Returns the icon identifier for the command
    pub fn icon(&self) -> &str {
        match self {
            SystemCommand::Shutdown => "power-off",
            SystemCommand::Restart => "refresh-cw",
            SystemCommand::Lock => "lock",
            SystemCommand::Sleep => "moon",
            SystemCommand::Hibernate => "archive",
            SystemCommand::LogOff => "log-out",
        }
    }

    /// Returns whether this command requires confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(
            self,
            SystemCommand::Shutdown | SystemCommand::Restart | SystemCommand::LogOff
        )
    }

    /// Returns all available system commands
    pub fn all() -> Vec<SystemCommand> {
        vec![
            SystemCommand::Shutdown,
            SystemCommand::Restart,
            SystemCommand::Lock,
            SystemCommand::Sleep,
            SystemCommand::Hibernate,
            SystemCommand::LogOff,
        ]
    }
}

/// Represents a quick action
#[derive(Debug, Clone)]
pub struct QuickAction {
    /// Display name of the action
    pub name: String,
    /// Description of what the action does
    pub description: String,
    /// Icon identifier (Lucide icon name)
    pub icon: String,
    /// System command to execute
    pub command: SystemCommand,
}

impl QuickAction {
    /// Creates a new QuickAction from a SystemCommand
    pub fn from_command(command: SystemCommand) -> Self {
        Self {
            name: command.display_name().to_string(),
            description: command.description().to_string(),
            icon: command.icon().to_string(),
            command,
        }
    }

    /// Returns all predefined quick actions
    pub fn all_actions() -> Vec<QuickAction> {
        SystemCommand::all()
            .into_iter()
            .map(QuickAction::from_command)
            .collect()
    }
}

/// Quick Action search provider
pub struct QuickActionProvider {
    /// List of available quick actions
    actions: Vec<QuickAction>,
    /// Whether the provider is enabled
    enabled: bool,
}

impl QuickActionProvider {
    /// Creates a new QuickActionProvider
    pub fn new() -> Result<Self> {
        info!("Initializing QuickActionProvider");

        Ok(Self {
            actions: QuickAction::all_actions(),
            enabled: true,
        })
    }

    /// Performs fuzzy search on action names
    fn fuzzy_match(query: &str, action_name: &str) -> Option<f64> {
        let query_lower = query.to_lowercase();
        let name_lower = action_name.to_lowercase();

        // Exact match
        if name_lower == query_lower {
            return Some(100.0);
        }

        // Starts with query
        if name_lower.starts_with(&query_lower) {
            return Some(90.0);
        }

        // Contains query
        if name_lower.contains(&query_lower) {
            return Some(70.0);
        }

        // Check for fuzzy character match
        if Self::fuzzy_char_match(&query_lower, &name_lower) {
            return Some(50.0);
        }

        None
    }

    /// Checks if all characters in query appear in order in name
    fn fuzzy_char_match(query: &str, name: &str) -> bool {
        let mut name_chars = name.chars();

        for query_char in query.chars() {
            if !name_chars.any(|c| c == query_char) {
                return false;
            }
        }

        true
    }

    /// Converts QuickAction to SearchResult
    fn convert_to_search_result(&self, action: &QuickAction, score: f64) -> SearchResult {
        let mut metadata = HashMap::new();
        metadata.insert(
            "command".to_string(),
            serde_json::json!(action.command),
        );
        metadata.insert(
            "requires_confirmation".to_string(),
            serde_json::json!(action.command.requires_confirmation()),
        );

        SearchResult {
            id: format!("quick_action:{}", action.name.to_lowercase().replace(' ', "_")),
            title: action.name.clone(),
            subtitle: action.description.clone(),
            icon: Some(action.icon.clone()),
            result_type: ResultType::QuickAction,
            score,
            metadata,
            action: ResultAction::ExecuteCommand {
                command: format!("system:{:?}", action.command),
                args: vec![],
            },
        }
    }
}

#[async_trait]
impl SearchProvider for QuickActionProvider {
    fn name(&self) -> &str {
        "QuickAction"
    }

    fn priority(&self) -> u8 {
        80 // High priority for quick actions
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        debug!("Searching quick actions for query: '{}'", query);

        // Perform fuzzy search on action names
        let mut results = Vec::new();
        for action in &self.actions {
            if let Some(score) = Self::fuzzy_match(query, &action.name) {
                let result = self.convert_to_search_result(action, score);
                results.push(result);
            }
        }

        // Sort by score (highest first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(MAX_RESULTS);

        debug!("Found {} matching quick actions", results.len());
        Ok(results)
    }

    async fn execute(&self, result: &SearchResult) -> Result<()> {
        if result.result_type != ResultType::QuickAction {
            return Err(LauncherError::ExecutionError(
                "Not a quick action result".to_string(),
            ));
        }

        // Extract command from metadata
        let command = result
            .metadata
            .get("command")
            .and_then(|v| serde_json::from_value::<SystemCommand>(v.clone()).ok())
            .ok_or_else(|| {
                LauncherError::ExecutionError("Invalid quick action command".to_string())
            })?;

        info!("Executing quick action: {:?}", command);

        // Execute the system command
        Self::execute_system_command(command).await
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("QuickActionProvider initialized with {} actions", self.actions.len());
        Ok(())
    }
}

impl Default for QuickActionProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            actions: Vec::new(),
            enabled: false,
        })
    }
}

impl QuickActionProvider {
    /// Executes a system command
    #[cfg(windows)]
    async fn execute_system_command(command: SystemCommand) -> Result<()> {
        info!("Executing system command: {:?}", command);

        // Execute command in a blocking task
        tokio::task::spawn_blocking(move || Self::execute_system_command_sync(command))
            .await
            .map_err(|e| {
                LauncherError::ExecutionError(format!("Failed to spawn command task: {}", e))
            })??;

        info!("Successfully executed system command: {:?}", command);
        Ok(())
    }

    /// Synchronously executes a system command using Windows API
    #[cfg(windows)]
    fn execute_system_command_sync(command: SystemCommand) -> Result<()> {
        use std::process::Command;

        match command {
            SystemCommand::Shutdown => {
                // shutdown /s /t 0 - Shutdown immediately
                Command::new("shutdown")
                    .args(["/s", "/t", "0"])
                    .spawn()
                    .map_err(|e| {
                        LauncherError::ExecutionError(format!("Failed to execute shutdown: {}", e))
                    })?;
            }
            SystemCommand::Restart => {
                // shutdown /r /t 0 - Restart immediately
                Command::new("shutdown")
                    .args(["/r", "/t", "0"])
                    .spawn()
                    .map_err(|e| {
                        LauncherError::ExecutionError(format!("Failed to execute restart: {}", e))
                    })?;
            }
            SystemCommand::Lock => {
                // rundll32.exe user32.dll,LockWorkStation - Lock the workstation
                Command::new("rundll32.exe")
                    .args(["user32.dll,LockWorkStation"])
                    .spawn()
                    .map_err(|e| {
                        LauncherError::ExecutionError(format!("Failed to execute lock: {}", e))
                    })?;
            }
            SystemCommand::Sleep => {
                // rundll32.exe powrprof.dll,SetSuspendState 0,1,0 - Sleep
                Command::new("rundll32.exe")
                    .args(["powrprof.dll,SetSuspendState", "0,1,0"])
                    .spawn()
                    .map_err(|e| {
                        LauncherError::ExecutionError(format!("Failed to execute sleep: {}", e))
                    })?;
            }
            SystemCommand::Hibernate => {
                // shutdown /h - Hibernate
                Command::new("shutdown")
                    .args(["/h"])
                    .spawn()
                    .map_err(|e| {
                        LauncherError::ExecutionError(format!(
                            "Failed to execute hibernate: {}",
                            e
                        ))
                    })?;
            }
            SystemCommand::LogOff => {
                // shutdown /l - Log off
                Command::new("shutdown")
                    .args(["/l"])
                    .spawn()
                    .map_err(|e| {
                        LauncherError::ExecutionError(format!("Failed to execute logoff: {}", e))
                    })?;
            }
        }

        Ok(())
    }

    #[cfg(not(windows))]
    async fn execute_system_command(command: SystemCommand) -> Result<()> {
        Err(LauncherError::ExecutionError(format!(
            "System command execution not supported on this platform: {:?}",
            command
        )))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_action_provider_creation() {
        let provider = QuickActionProvider::new();
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "QuickAction");
        assert_eq!(provider.priority(), 80);
        assert!(provider.is_enabled());
        assert_eq!(provider.actions.len(), 6); // All system commands
    }

    #[tokio::test]
    async fn test_system_command_properties() {
        // Test display names
        assert_eq!(SystemCommand::Shutdown.display_name(), "Shutdown");
        assert_eq!(SystemCommand::Restart.display_name(), "Restart");
        assert_eq!(SystemCommand::Lock.display_name(), "Lock");
        assert_eq!(SystemCommand::Sleep.display_name(), "Sleep");
        assert_eq!(SystemCommand::Hibernate.display_name(), "Hibernate");
        assert_eq!(SystemCommand::LogOff.display_name(), "Log Off");

        // Test confirmation requirements
        assert!(SystemCommand::Shutdown.requires_confirmation());
        assert!(SystemCommand::Restart.requires_confirmation());
        assert!(SystemCommand::LogOff.requires_confirmation());
        assert!(!SystemCommand::Lock.requires_confirmation());
        assert!(!SystemCommand::Sleep.requires_confirmation());
        assert!(!SystemCommand::Hibernate.requires_confirmation());
    }

    #[tokio::test]
    async fn test_fuzzy_search_exact_match() {
        let provider = QuickActionProvider::new().unwrap();

        // Test exact match
        let results = provider.search("shutdown").await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Shutdown");
        assert_eq!(results[0].score, 100.0);
        assert_eq!(results[0].result_type, ResultType::QuickAction);
    }

    #[tokio::test]
    async fn test_fuzzy_search_starts_with() {
        let provider = QuickActionProvider::new().unwrap();

        // Test starts with
        let results = provider.search("rest").await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Restart");
        assert_eq!(results[0].score, 90.0);
    }

    #[tokio::test]
    async fn test_fuzzy_search_contains() {
        let provider = QuickActionProvider::new().unwrap();

        // Test contains
        let results = provider.search("lock").await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Lock");
        assert_eq!(results[0].score, 100.0); // Exact match
    }

    #[tokio::test]
    async fn test_fuzzy_search_partial() {
        let provider = QuickActionProvider::new().unwrap();

        // Test partial match
        let results = provider.search("slp").await.unwrap();
        assert!(!results.is_empty());
        
        // Should find "Sleep" with fuzzy matching
        let sleep_result = results.iter().find(|r| r.title == "Sleep");
        assert!(sleep_result.is_some());
    }

    #[tokio::test]
    async fn test_fuzzy_search_no_match() {
        let provider = QuickActionProvider::new().unwrap();

        // Test no match
        let results = provider.search("xyz123").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_fuzzy_search_empty_query() {
        let provider = QuickActionProvider::new().unwrap();

        // Test empty query
        let results = provider.search("").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_result_sorting() {
        let provider = QuickActionProvider::new().unwrap();

        // Search with a query that matches multiple actions
        let results = provider.search("s").await.unwrap();
        
        // Results should be sorted by score (highest first)
        for i in 1..results.len() {
            assert!(results[i - 1].score >= results[i].score);
        }
    }

    #[tokio::test]
    async fn test_search_result_metadata() {
        let provider = QuickActionProvider::new().unwrap();

        let results = provider.search("shutdown").await.unwrap();
        assert!(!results.is_empty());

        let result = &results[0];
        
        // Check metadata
        assert!(result.metadata.contains_key("command"));
        assert!(result.metadata.contains_key("requires_confirmation"));
        
        let requires_confirmation = result.metadata.get("requires_confirmation")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        assert!(requires_confirmation); // Shutdown requires confirmation
    }

    #[tokio::test]
    async fn test_quick_action_all_actions() {
        let actions = QuickAction::all_actions();
        assert_eq!(actions.len(), 6);

        // Verify all actions have required fields
        for action in actions {
            assert!(!action.name.is_empty());
            assert!(!action.description.is_empty());
            assert!(!action.icon.is_empty());
        }
    }

    #[tokio::test]
    async fn test_fuzzy_char_match() {
        // Test character sequence matching
        assert!(QuickActionProvider::fuzzy_char_match("sdn", "shutdown"));
        assert!(QuickActionProvider::fuzzy_char_match("rst", "restart"));
        assert!(QuickActionProvider::fuzzy_char_match("lck", "lock"));
        
        // Test non-match
        assert!(!QuickActionProvider::fuzzy_char_match("xyz", "shutdown"));
    }

    #[tokio::test]
    async fn test_provider_initialization() {
        let mut provider = QuickActionProvider::new().unwrap();
        
        let result = provider.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_invalid_result_type() {
        let provider = QuickActionProvider::new().unwrap();

        // Create a result with wrong type
        let invalid_result = SearchResult {
            id: "test".to_string(),
            title: "Test".to_string(),
            subtitle: "Test".to_string(),
            icon: None,
            result_type: ResultType::File, // Wrong type
            score: 100.0,
            metadata: HashMap::new(),
            action: ResultAction::ExecuteCommand {
                command: "test".to_string(),
                args: vec![],
            },
        };

        let result = provider.execute(&invalid_result).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_system_command_all() {
        let commands = SystemCommand::all();
        assert_eq!(commands.len(), 6);
        
        // Verify all commands are present
        assert!(commands.contains(&SystemCommand::Shutdown));
        assert!(commands.contains(&SystemCommand::Restart));
        assert!(commands.contains(&SystemCommand::Lock));
        assert!(commands.contains(&SystemCommand::Sleep));
        assert!(commands.contains(&SystemCommand::Hibernate));
        assert!(commands.contains(&SystemCommand::LogOff));
    }
}
