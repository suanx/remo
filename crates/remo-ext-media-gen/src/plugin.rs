//! Media generation plugin — registers image and video generation tools.

use std::sync::Arc;
use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::MutationBatch;
use remo_runtime_contract::StateError;
use remo_runtime_contract::registry_spec::AgentSpec;
use crate::config::MediaGenConfigKey;
use crate::tools::{GenerateImageTool, GenerateVideoTool};

pub const MEDIA_GEN_PLUGIN_NAME: &str = "media_gen";
pub const GENERATE_IMAGE_TOOL_ID: &str = "media:generate_image";
pub const GENERATE_VIDEO_TOOL_ID: &str = "media:generate_video";

/// Media generation plugin.
///
/// Provides tools for generating images and videos using various AI providers
/// including OpenAI DALL-E 3, Agnes AI Image/Video, and OpenAI-compatible APIs.
pub struct MediaGenPlugin;

impl Plugin for MediaGenPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor { name: MEDIA_GEN_PLUGIN_NAME }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        registrar.register_tool(GENERATE_IMAGE_TOOL_ID, Arc::new(GenerateImageTool))?;
        registrar.register_tool(GENERATE_VIDEO_TOOL_ID, Arc::new(GenerateVideoTool))?;
        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<MediaGenConfigKey>()
                .with_display_name("Media Generation")
                .with_description("Image and video generation via OpenAI DALL-E 3, Agnes AI, and OpenAI-compatible APIs")
                .with_category("media")
                .with_editor("media_gen"),
        ]
    }

    fn on_activate(&self, agent_spec: &AgentSpec, _patch: &mut MutationBatch) -> Result<(), StateError> {
        let config = agent_spec.config::<MediaGenConfigKey>()?;
        tracing::info!(
            image_provider = %config.default_image_provider,
            image_model = %config.default_image_model,
            video_provider = %config.default_video_provider,
            video_model = %config.default_video_model,
            "Media generation plugin activated"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_descriptor() {
        let p = MediaGenPlugin;
        assert_eq!(p.descriptor().name, "media_gen");
    }

    #[test]
    fn plugin_has_config_schemas() {
        let p = MediaGenPlugin;
        assert!(!p.config_schemas().is_empty());
    }
}
