//! Tool implementations for Text-to-Speech and Speech-to-Text.
//!
//! Provides two [`TypedTool`] implementations:
//! - [`TextToSpeechTool`] for synthesizing speech from text.
//! - [`SpeechToTextTool`] for transcribing audio to text.
//!
//! These are placeholder implementations that log the call and return
//! structured metadata. Production use would require integrating with
//! the configured TTS/ASR provider APIs.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing;

use remo_runtime_contract::contract::tool::{
    ToolCallContext, ToolError, ToolOutput, ToolResult, TypedTool,
};

// ---------------------------------------------------------------------------
// TextToSpeechTool
// ---------------------------------------------------------------------------

/// Arguments for [`TextToSpeechTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TextToSpeechArgs {
    /// The text content to synthesize into speech.
    pub text: String,
    /// Optional voice identifier override. Falls back to the configured
    /// default voice if not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
}

/// Tool that converts text to speech using the configured TTS provider.
///
/// This is a placeholder implementation. In production, it would call
/// the provider specified in [`VoiceConfig`](crate::config::VoiceConfig)
/// — e.g. Edge TTS, OpenAI TTS, or Azure TTS — to generate audio data.
pub struct TextToSpeechTool;

impl TextToSpeechTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TextToSpeechTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TypedTool for TextToSpeechTool {
    type Args = TextToSpeechArgs;

    fn tool_id(&self) -> &str {
        "voice:text_to_speech"
    }

    fn name(&self) -> &str {
        "text_to_speech"
    }

    fn description(&self) -> &str {
        "Convert text to speech audio using the configured TTS provider. Returns audio data or a reference to generated audio."
    }

    fn category(&self) -> Option<&str> {
        Some("voice")
    }

    async fn execute(
        &self,
        args: Self::Args,
        _ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let voice = args.voice.as_deref().unwrap_or("default");

        tracing::info!(
            target: "remo_ext_voice::tools",
            tool = "text_to_speech",
            text_len = args.text.len(),
            voice = voice,
            "Text-to-speech synthesis requested (placeholder)"
        );

        let data = serde_json::json!({
            "status": "placeholder",
            "tool": "text_to_speech",
            "text_length": args.text.len(),
            "voice": voice,
            "message": format!(
                "[placeholder] Text-to-speech synthesis for {text_len} characters using voice '{voice}'. \
                 A real implementation would call the configured TTS provider to generate audio.",
                text_len = args.text.len()
            ),
        });

        Ok(ToolResult::success("text_to_speech", data).into())
    }
}

// ---------------------------------------------------------------------------
// SpeechToTextTool
// ---------------------------------------------------------------------------

/// Arguments for [`SpeechToTextTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SpeechToTextArgs {
    /// Raw audio data bytes (e.g. WAV, MP3, OGG).
    pub audio_data: Vec<u8>,
    /// Audio format hint (e.g. `"wav"`, `"mp3"`, `"ogg"`, `"webm"`).
    /// When `None`, the provider may attempt auto-detection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Optional language code (e.g. `"zh"`, `"en"`, `"ja"`).
    /// Helps the ASR model improve accuracy for the expected language.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Tool that transcribes speech audio to text using the configured ASR provider.
///
/// This is a placeholder implementation. In production, it would call
/// the provider specified in [`VoiceConfig`](crate::config::VoiceConfig)
/// — e.g. OpenAI Whisper or Azure ASR — to transcribe the audio.
pub struct SpeechToTextTool;

impl SpeechToTextTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SpeechToTextTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TypedTool for SpeechToTextTool {
    type Args = SpeechToTextArgs;

    fn tool_id(&self) -> &str {
        "voice:speech_to_text"
    }

    fn name(&self) -> &str {
        "speech_to_text"
    }

    fn description(&self) -> &str {
        "Transcribe speech audio to text using the configured ASR provider. Accepts raw audio data in common formats."
    }

    fn category(&self) -> Option<&str> {
        Some("voice")
    }

    async fn execute(
        &self,
        args: Self::Args,
        _ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let audio_size_kb = args.audio_data.len() as f64 / 1024.0;
        let format = args.format.as_deref().unwrap_or("unknown");
        let language = args.language.as_deref().unwrap_or("auto");

        tracing::info!(
            target: "remo_ext_voice::tools",
            tool = "speech_to_text",
            audio_size_kb = format!("{audio_size_kb:.1}"),
            format = format,
            language = language,
            "Speech-to-text transcription requested (placeholder)"
        );

        let data = serde_json::json!({
            "status": "placeholder",
            "tool": "speech_to_text",
            "audio_size_kb": format!("{audio_size_kb:.1}"),
            "format": format,
            "language": language,
            "message": format!(
                "[placeholder] Speech-to-text transcription for {audio_size_kb:.1} KB of {format} audio \
                 (language: {language}). A real implementation would call the configured ASR provider.",
            ),
        });

        Ok(ToolResult::success("speech_to_text", data).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_to_speech_tool_descriptor() {
        let tool = TextToSpeechTool::new();
        assert_eq!(tool.tool_id(), "voice:text_to_speech");
        assert_eq!(tool.name(), "text_to_speech");
        assert!(tool.description().contains("Convert text to speech"));
        assert_eq!(tool.category(), Some("voice"));
    }

    #[test]
    fn speech_to_text_tool_descriptor() {
        let tool = SpeechToTextTool::new();
        assert_eq!(tool.tool_id(), "voice:speech_to_text");
        assert_eq!(tool.name(), "speech_to_text");
        assert!(tool.description().contains("Transcribe speech audio"));
        assert_eq!(tool.category(), Some("voice"));
    }

    #[test]
    fn text_to_speech_args_defaults() {
        let args = TextToSpeechArgs {
            text: "Hello, world!".to_string(),
            voice: None,
        };
        assert_eq!(args.text, "Hello, world!");
        assert!(args.voice.is_none());
    }

    #[test]
    fn speech_to_text_args_defaults() {
        let args = SpeechToTextArgs {
            audio_data: vec![0u8; 1024],
            format: None,
            language: None,
        };
        assert_eq!(args.audio_data.len(), 1024);
        assert!(args.format.is_none());
        assert!(args.language.is_none());
    }
}
