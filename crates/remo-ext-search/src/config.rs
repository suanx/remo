//! Configuration types for the web search extension.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use remo_runtime_contract::PluginConfigKey;
use remo_runtime_contract::RedactedString;

// ---------------------------------------------------------------------------
// SearchProvider
// ---------------------------------------------------------------------------

/// Supported web search API providers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchProvider {
    /// Tavily Search API (https://tavily.com)
    #[default]
    Tavily,
    /// SerpAPI (https://serpapi.com)
    SerpApi,
    /// Bing Web Search API (Azure)
    Bing,
    /// Google Custom Search API
    Google,
}

// ---------------------------------------------------------------------------
// SearchConfig
// ---------------------------------------------------------------------------

/// Configuration for the web search extension.
///
/// Stored in `AgentSpec.sections["search"]` and resolved at runtime by
/// search tools to determine which provider to use and how many results
/// to return.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SearchConfig {
    /// API key for the configured search provider.
    pub api_key: RedactedString,

    /// Maximum number of search results to return per query.
    pub max_results: usize,

    /// Optional list of domains to restrict search results to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_domains: Option<Vec<String>>,

    /// Optional list of domains to exclude from search results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_domains: Option<Vec<String>>,

    /// The search provider to use.
    pub provider: SearchProvider,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            api_key: RedactedString::new(""),
            max_results: 5,
            include_domains: None,
            exclude_domains: None,
            provider: SearchProvider::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// SearchConfigKey
// ---------------------------------------------------------------------------

/// [`PluginConfigKey`] binding for web search configuration in agent specs.
///
/// ```ignore
/// let config = spec.config::<SearchConfigKey>();
/// ```
pub struct SearchConfigKey;

impl PluginConfigKey for SearchConfigKey {
    const KEY: &'static str = "search";
    type Config = SearchConfig;
}
