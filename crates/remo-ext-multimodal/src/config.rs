//! Configuration types for the multimodal extension.
//!
//! Provides [`MultimodalConfigKey`] for integration with the agent spec
//! configuration system, and [`MultimodalConfig`] with sensible defaults.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use remo_runtime_contract::PluginConfigKey;

/// Default maximum image size in megabytes.
const DEFAULT_MAX_IMAGE_SIZE_MB: f64 = 10.0;

/// Default supported file formats.
fn default_supported_formats() -> Vec<String> {
    vec![
        "txt".to_string(),
        "md".to_string(),
        "csv".to_string(),
        "json".to_string(),
    ]
}

/// Multimodal extension configuration.
///
/// Loaded from the `multimodal` section of `AgentSpec.sections`.
///
/// # Example JSON
///
/// ```json
/// {
///     "max_image_size_mb": 20.0,
///     "supported_formats": ["txt", "md", "csv", "json", "pdf"],
///     "tts_provider": "openai",
///     "vision_provider": "openai",
///     "vision_model": "gpt-4o",
///     "vision_api_key": "sk-...",
///     "vision_max_tokens": 300
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct MultimodalConfig {
    /// Maximum image size in megabytes for processing. Larger images are rejected.
    pub max_image_size_mb: f64,
    /// File formats supported for text extraction and parsing.
    pub supported_formats: Vec<String>,
    /// Optional text-to-speech provider identifier. When set, enables TTS
    /// synthesis of text content through the specified provider.
    pub tts_provider: Option<String>,

    // ── Vision API configuration ─────────────────────────────────────────

    /// Vision provider to use for image description (`"openai"`, `"anthropic"`,
    /// or `"ollama"`). When unset, vision features are disabled.
    pub vision_provider: Option<String>,
    /// Model name for the vision provider (e.g. `"gpt-4o"`,
    /// `"claude-3-5-sonnet-20241022"`, `"llava"`).
    /// Defaults to the provider's recommended model.
    pub vision_model: Option<String>,
    /// API key for the vision provider (required for OpenAI and Anthropic).
    pub vision_api_key: Option<String>,
    /// Base URL for the vision provider. Required for Ollama
    /// (default `"http://localhost:11434"`); optional for cloud providers.
    pub vision_base_url: Option<String>,
    /// Maximum number of tokens in the vision API response (default 300).
    pub vision_max_tokens: Option<u32>,
}

impl Default for MultimodalConfig {
    fn default() -> Self {
        Self {
            max_image_size_mb: DEFAULT_MAX_IMAGE_SIZE_MB,
            supported_formats: default_supported_formats(),
            tts_provider: None,
            vision_provider: None,
            vision_model: None,
            vision_api_key: None,
            vision_base_url: None,
            vision_max_tokens: None,
        }
    }
}

/// [`PluginConfigKey`] binding for multimodal configuration in agent specs.
///
/// # Example
///
/// ```ignore
/// let config = agent_spec.config::<MultimodalConfigKey>()?;
/// ```
pub struct MultimodalConfigKey;

impl PluginConfigKey for MultimodalConfigKey {
    const KEY: &'static str = "multimodal";
    type Config = MultimodalConfig;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = MultimodalConfig::default();
        assert_eq!(config.max_image_size_mb, 10.0);
        assert_eq!(config.supported_formats, vec!["txt", "md", "csv", "json"]);
        assert!(config.tts_provider.is_none());
        assert!(config.vision_provider.is_none());
        assert!(config.vision_model.is_none());
        assert!(config.vision_api_key.is_none());
        assert!(config.vision_base_url.is_none());
        assert!(config.vision_max_tokens.is_none());
    }

    #[test]
    fn config_key_binding() {
        assert_eq!(MultimodalConfigKey::KEY, "multimodal");
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = MultimodalConfig {
            max_image_size_mb: 20.0,
            supported_formats: vec!["txt".to_string(), "pdf".to_string()],
            tts_provider: Some("openai".to_string()),
            vision_provider: Some("anthropic".to_string()),
            vision_model: Some("claude-3-5-sonnet-20241022".to_string()),
            vision_api_key: Some("sk-ant-xxx".to_string()),
            vision_base_url: Some("https://custom.anthropic.com".to_string()),
            vision_max_tokens: Some(500),
        };
        let json = serde_json::to_string(&config).unwrap();
        let decoded: MultimodalConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.max_image_size_mb, 20.0);
        assert_eq!(decoded.supported_formats, vec!["txt", "pdf"]);
        assert_eq!(decoded.tts_provider.as_deref(), Some("openai"));
        assert_eq!(decoded.vision_provider.as_deref(), Some("anthropic"));
        assert_eq!(
            decoded.vision_model.as_deref(),
            Some("claude-3-5-sonnet-20241022")
        );
        assert_eq!(decoded.vision_api_key.as_deref(), Some("sk-ant-xxx"));
        assert_eq!(
            decoded.vision_base_url.as_deref(),
            Some("https://custom.anthropic.com")
        );
        assert_eq!(decoded.vision_max_tokens, Some(500));
    }

    #[test]
    fn config_partial_deserialize() {
        let json = r#"{"max_image_size_mb": 5.0}"#;
        let config: MultimodalConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_image_size_mb, 5.0);
        assert_eq!(config.supported_formats, default_supported_formats());
        assert!(config.tts_provider.is_none());
        assert!(config.vision_provider.is_none());
    }

    #[test]
    fn config_empty_object_uses_defaults() {
        let json = r#"{}"#;
        let config: MultimodalConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config, MultimodalConfig::default());
    }
}
