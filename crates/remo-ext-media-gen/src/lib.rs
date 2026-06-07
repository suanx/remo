//! Image and video generation extension for the Remo AI Agent framework.
//!
//! Provides tools for generating images and videos using:
//! - OpenAI DALL-E 3
//! - Agnes AI Image Flash (2.0/2.1)
//! - Agnes AI Video V2.0
//! - Any OpenAI-compatible API

pub mod config;
pub mod plugin;
pub mod tools;

pub use config::{MediaGenConfig, MediaGenConfigKey, MediaGenConfigKey as ConfigKey};
pub use plugin::{MediaGenPlugin, GENERATE_IMAGE_TOOL_ID, GENERATE_VIDEO_TOOL_ID};
pub use tools::{GenerateImageTool, GenerateVideoTool};
