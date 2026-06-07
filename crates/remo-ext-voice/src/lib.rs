//! Voice interaction extension (TTS/ASR) for the Remo AI Agent framework.
//!
//! Provides Text-to-Speech and Speech-to-Text capabilities through
//! configurable provider integrations.

pub mod config;
pub mod plugin;
pub mod tools;

pub use config::{AsrProvider, TtsProvider, VoiceConfig, VoiceConfigKey};
pub use plugin::VoicePlugin;
pub use tools::{SpeechToTextTool, TextToSpeechTool};
