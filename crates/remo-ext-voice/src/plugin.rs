//! Voice plugin implementation.
//!
//! Registers the voice extension with the Remo AI Agent framework,
//! providing Text-to-Speech and Speech-to-Text tools.

use std::sync::Arc;

use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::MutationBatch;
use remo_runtime_contract::StateError;
use remo_runtime_contract::registry_spec::AgentSpec;

use crate::config::VoiceConfigKey;
use crate::tools::{SpeechToTextTool, TextToSpeechTool};

/// Stable plugin name for the voice extension.
pub const VOICE_PLUGIN_NAME: &str = "voice";

/// Tool ID for the text-to-speech tool.
pub const TEXT_TO_SPEECH_TOOL_ID: &str = "voice:text_to_speech";

/// Tool ID for the speech-to-text tool.
pub const SPEECH_TO_TEXT_TOOL_ID: &str = "voice:speech_to_text";

/// Voice interaction plugin.
///
/// Provides Text-to-Speech (TTS) and Speech-to-Text (ASR) capabilities
/// for the Remo AI Agent framework through configurable provider backends.
///
/// # Registered Components
///
/// - **Tool**: [`TextToSpeechTool`] — synthesizes speech from text.
/// - **Tool**: [`SpeechToTextTool`] — transcribes audio to text.
///
/// # Configuration
///
/// This plugin reads the `voice` section of `AgentSpec.sections` (see
/// [`VoiceConfig`](crate::config::VoiceConfig) for available options).
pub struct VoicePlugin;

impl Plugin for VoicePlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: VOICE_PLUGIN_NAME,
        }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        // Register the text-to-speech tool
        registrar.register_tool(
            TEXT_TO_SPEECH_TOOL_ID,
            Arc::new(TextToSpeechTool::new()),
        )?;

        // Register the speech-to-text tool
        registrar.register_tool(
            SPEECH_TO_TEXT_TOOL_ID,
            Arc::new(SpeechToTextTool::new()),
        )?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<VoiceConfigKey>()
                .with_display_name("Voice")
                .with_description("Text-to-Speech and Speech-to-Text provider settings.")
                .with_category("voice")
                .with_editor("voice"),
        ]
    }

    fn on_activate(
        &self,
        _agent_spec: &AgentSpec,
        _patch: &mut MutationBatch,
    ) -> Result<(), StateError> {
        // Read configuration to validate it.
        let _config = _agent_spec.config::<VoiceConfigKey>()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_descriptor_name() {
        let plugin = VoicePlugin;
        assert_eq!(plugin.descriptor().name, "voice");
    }

    #[test]
    fn plugin_has_config_schemas() {
        let plugin = VoicePlugin;
        let schemas = plugin.config_schemas();
        assert_eq!(schemas.len(), 1);
    }

    #[test]
    fn plugin_constants() {
        assert_eq!(VOICE_PLUGIN_NAME, "voice");
        assert_eq!(TEXT_TO_SPEECH_TOOL_ID, "voice:text_to_speech");
        assert_eq!(SPEECH_TO_TEXT_TOOL_ID, "voice:speech_to_text");
    }
}
