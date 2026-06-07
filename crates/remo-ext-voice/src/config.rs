//! Configuration types for the voice extension.
//!
//! Provides [`VoiceConfigKey`] for integration with the agent spec
//! configuration system, and [`VoiceConfig`] with sensible defaults for
//! Text-to-Speech and Speech-to-Text providers.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use remo_runtime_contract::PluginConfigKey;
use remo_runtime_contract::RedactedString;

// ---------------------------------------------------------------------------
// Provider enums
// ---------------------------------------------------------------------------

/// Supported Text-to-Speech provider backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TtsProvider {
    /// Microsoft Edge's built-in TTS engine (free, no API key required).
    #[serde(rename = "edge_tts")]
    EdgeTts,
    /// OpenAI Text-to-Speech API.
    #[serde(rename = "openai_tts")]
    OpenAiTts,
    /// Azure Cognitive Services Text-to-Speech.
    #[serde(rename = "azure_tts")]
    AzureTts,
}

/// Supported Speech-to-Text provider backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AsrProvider {
    /// OpenAI Whisper (local or API-based).
    #[serde(rename = "whisper")]
    Whisper,
    /// Azure Cognitive Services Speech-to-Text.
    #[serde(rename = "azure_asr")]
    AzureAsr,
}

// ---------------------------------------------------------------------------
// VoiceConfig
// ---------------------------------------------------------------------------

/// Default TTS voice for Edge TTS.
const DEFAULT_TTS_VOICE: &str = "zh-CN-XiaoxiaoNeural";

/// Voice extension configuration.
///
/// Loaded from the `voice` section of `AgentSpec.sections`.
///
/// # Example JSON
///
/// ```json
/// {
///     "tts_provider": "edge_tts",
///     "asr_provider": "whisper",
///     "tts_voice": "zh-CN-XiaoxiaoNeural",
///     "tts_api_key": null,
///     "asr_api_key": null
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct VoiceConfig {
    /// Text-to-Speech provider to use.
    pub tts_provider: TtsProvider,
    /// Speech-to-Text provider to use.
    pub asr_provider: AsrProvider,
    /// Voice identifier for TTS output (provider-specific).
    pub tts_voice: String,
    /// API key for the TTS provider (if required).
    pub tts_api_key: Option<RedactedString>,
    /// API key for the ASR provider (if required).
    pub asr_api_key: Option<RedactedString>,
    /// Base URL override for the TTS provider API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tts_api_base: Option<String>,
    /// Base URL override for the ASR provider API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asr_api_base: Option<String>,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            tts_provider: TtsProvider::EdgeTts,
            asr_provider: AsrProvider::Whisper,
            tts_voice: DEFAULT_TTS_VOICE.to_string(),
            tts_api_key: None,
            asr_api_key: None,
            tts_api_base: None,
            asr_api_base: None,
        }
    }
}

/// [`PluginConfigKey`] binding for voice configuration in agent specs.
///
/// # Example
///
/// ```ignore
/// let config = agent_spec.config::<VoiceConfigKey>()?;
/// ```
pub struct VoiceConfigKey;

impl PluginConfigKey for VoiceConfigKey {
    const KEY: &'static str = "voice";
    type Config = VoiceConfig;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = VoiceConfig::default();
        assert_eq!(config.tts_provider, TtsProvider::EdgeTts);
        assert_eq!(config.asr_provider, AsrProvider::Whisper);
        assert_eq!(config.tts_voice, "zh-CN-XiaoxiaoNeural");
        assert!(config.tts_api_key.is_none());
        assert!(config.asr_api_key.is_none());
        assert!(config.tts_api_base.is_none());
        assert!(config.asr_api_base.is_none());
    }

    #[test]
    fn config_key_binding() {
        assert_eq!(VoiceConfigKey::KEY, "voice");
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = VoiceConfig {
            tts_provider: TtsProvider::OpenAiTts,
            asr_provider: AsrProvider::AzureAsr,
            tts_voice: "en-US-JennyNeural".to_string(),
            tts_api_key: Some(RedactedString::new("sk-tts")),
            asr_api_key: Some(RedactedString::new("sk-asr")),
            tts_api_base: Some("https://custom-tts.example.com".to_string()),
            asr_api_base: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let decoded: VoiceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.tts_provider, TtsProvider::OpenAiTts);
        assert_eq!(decoded.asr_provider, AsrProvider::AzureAsr);
        assert_eq!(decoded.tts_voice, "en-US-JennyNeural");
        assert!(decoded.tts_api_key.is_some());
        assert!(decoded.asr_api_key.is_some());
        assert_eq!(
            decoded.tts_api_base.as_deref(),
            Some("https://custom-tts.example.com")
        );
        assert!(decoded.asr_api_base.is_none());
    }

    #[test]
    fn config_partial_deserialize() {
        let json = r#"{"tts_provider": "azure_tts"}"#;
        let config: VoiceConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.tts_provider, TtsProvider::AzureTts);
        assert_eq!(config.asr_provider, AsrProvider::Whisper);
        assert!(config.tts_api_key.is_none());
    }

    #[test]
    fn config_empty_object_uses_defaults() {
        let json = r#"{}"#;
        let config: VoiceConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config, VoiceConfig::default());
    }

    #[test]
    fn tts_provider_serde() {
        assert_eq!(
            serde_json::to_value(TtsProvider::EdgeTts).unwrap(),
            serde_json::json!("edge_tts")
        );
        assert_eq!(
            serde_json::to_value(TtsProvider::OpenAiTts).unwrap(),
            serde_json::json!("openai_tts")
        );
        assert_eq!(
            serde_json::to_value(TtsProvider::AzureTts).unwrap(),
            serde_json::json!("azure_tts")
        );
    }

    #[test]
    fn asr_provider_serde() {
        assert_eq!(
            serde_json::to_value(AsrProvider::Whisper).unwrap(),
            serde_json::json!("whisper")
        );
        assert_eq!(
            serde_json::to_value(AsrProvider::AzureAsr).unwrap(),
            serde_json::json!("azure_asr")
        );
    }
}
