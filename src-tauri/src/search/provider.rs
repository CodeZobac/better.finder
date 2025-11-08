use async_trait::async_trait;
use crate::error::Result;
use crate::types::SearchResult;

/// Trait that all search providers must implement
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Returns the name of the provider
    fn name(&self) -> &str;

    /// Returns the priority of the provider (higher = searched first)
    /// Typical values: 0-100
    fn priority(&self) -> u8;

    /// Performs a search with the given query
    /// Returns a vector of search results
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>>;

    /// Executes the action associated with a search result
    async fn execute(&self, result: &SearchResult) -> Result<()>;

    /// Returns whether this provider is currently enabled
    fn is_enabled(&self) -> bool {
        true
    }

    /// Optional: Initialize the provider (e.g., load cache, connect to services)
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Optional: Cleanup resources when provider is no longer needed
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
