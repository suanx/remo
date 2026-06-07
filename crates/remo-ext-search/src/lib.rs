//! Web search extension with multi-provider support for the Remo AI Agent framework.
//!
//! Provides tools for searching the web and fetching webpage content through
//! configurable search providers (Tavily, SerpAPI, Bing, Google Custom Search).

pub mod config;
pub mod plugin;
pub mod tools;

pub use config::{SearchConfig, SearchConfigKey, SearchProvider};
pub use plugin::SearchPlugin;
pub use tools::{FetchWebpageTool, SearchWebTool};
