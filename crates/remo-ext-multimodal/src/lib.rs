//! Multimodal support extension for the Remo AI Agent framework.
//!
//! Provides file parsing, multimodal content handling, and image description
//! capabilities. Supports text files, CSV, JSON, and provides hooks for
//! preprocessing multimodal content before inference.

pub mod config;
pub mod file_parser;
pub mod hooks;
pub mod modality;
pub mod plugin;
pub mod tools;
pub mod vision;

pub use config::{MultimodalConfig, MultimodalConfigKey};
pub use modality::{MediaDescriptor, MediaSource, ModalityType, MultimodalContent};
pub use plugin::MultimodalPlugin;
