//! Configuration for OpenCode Zen provider and CLI integration.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use remo_runtime_contract::PluginConfigKey;

/// Default OpenCode Zen API base URL.
const DEFAULT_ZEN_API_URL: &str = "https://opencode.ai/zen/v1";
const DEFAULT_ZEN_CHAT_URL: &str = "https://opencode.ai/zen/v1/chat/completions";
const DEFAULT_ZEN_MODELS_URL: &str = "https://opencode.ai/zen/v1/models";
const DEFAULT_CLI_TIMEOUT_SECS: u64 = 300;

/// OpenCode extension configuration.
///
/// Stored under `sections["opencode"]` in the agent spec.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct OpenCodeConfig {
    /// API key for OpenCode Zen (required for non-free models).
    /// Free models (DeepSeek V4 Flash Free, Big Pickle, etc.) work without a key.
    pub zen_api_key: Option<String>,

    /// OpenCode Zen API base URL.
    pub zen_api_url: String,

    /// Auto-discover free models on startup.
    pub auto_discover_free_models: bool,

    /// Whether to enable OpenCode CLI tool for code generation.
    pub enable_cli_tool: bool,

    /// Path to the `opencode` CLI binary. If not set, resolves from PATH.
    pub cli_binary_path: Option<String>,

    /// Timeout in seconds for CLI operations.
    pub cli_timeout_secs: u64,

    /// Additional model IDs to fetch from Zen API (semi-colon separated).
    /// Free models are always fetched when auto_discover_free_models is true.
    pub extra_model_ids: Option<String>,

    /// List of discovered free models (populated at runtime, not configured).
    #[serde(skip)]
    pub discovered_free_models: Vec<DiscoveredModel>,
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        Self {
            zen_api_key: None,
            zen_api_url: DEFAULT_ZEN_API_URL.to_string(),
            auto_discover_free_models: true,
            enable_cli_tool: true,
            cli_binary_path: None,
            cli_timeout_secs: DEFAULT_CLI_TIMEOUT_SECS,
            extra_model_ids: None,
            discovered_free_models: Vec::new(),
        }
    }
}

/// A model discovered from OpenCode Zen.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveredModel {
    /// Model ID (e.g. "deepseek-v4-flash-free").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Is this a free model (zero cost).
    pub is_free: bool,
    /// Provider (openai-compatible).
    pub provider: String,
    /// Upstream model ID for the provider.
    pub upstream_model: String,
}

impl DiscoveredModel {
    pub fn free(id: impl Into<String>, name: impl Into<String>, upstream: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            is_free: true,
            provider: "opencode".to_string(),
            upstream_model: upstream.into(),
        }
    }
}

/// Built-in free models known to work with OpenCode Zen.
/// These are always available without API key.
pub fn builtin_free_models() -> Vec<DiscoveredModel> {
    vec![
        DiscoveredModel::free("deepseek-v4-flash-free", "DeepSeek V4 Flash (Free)", "deepseek-v4-flash-free"),
        DiscoveredModel::free("big-pickle", "Big Pickle (Free)", "big-pickle"),
        DiscoveredModel::free("mimo-v2.5-free", "MiMo V2.5 (Free)", "mimo-v2.5-free"),
        DiscoveredModel::free("nemotron-3-ultra-free", "Nemotron 3 Ultra (Free)", "nemotron-3-ultra-free"),
    ]
}

/// PluginConfigKey binding for OpenCode configuration.
pub struct OpenCodeConfigKey;

impl PluginConfigKey for OpenCodeConfigKey {
    const KEY: &'static str = "opencode";
    type Config = OpenCodeConfig;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = OpenCodeConfig::default();
        assert!(cfg.auto_discover_free_models);
        assert!(cfg.enable_cli_tool);
        assert_eq!(cfg.zen_api_url, "https://opencode.ai/zen/v1");
    }

    #[test]
    fn builtin_free_models_count() {
        let models = builtin_free_models();
        assert_eq!(models.len(), 4);
        assert!(models.iter().all(|m| m.is_free));
    }

    #[test]
    fn discovered_model_serde() {
        let m = DiscoveredModel::free("test", "Test Model", "test-upstream");
        let json = serde_json::to_string(&m).unwrap();
        let parsed: DiscoveredModel = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test");
        assert!(parsed.is_free);
    }
}
