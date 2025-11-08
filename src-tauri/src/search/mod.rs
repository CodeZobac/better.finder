pub mod provider;
pub mod engine;
pub mod providers;
pub mod cache;

#[cfg(test)]
mod engine_test;

#[cfg(test)]
mod performance_bench;

pub use provider::SearchProvider;
pub use engine::SearchEngine;
pub use providers::FileSearchProvider;
pub use cache::ResultCache;
