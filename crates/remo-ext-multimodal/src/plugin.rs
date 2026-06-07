//! Multimodal plugin implementation.
//!
//! Registers the multimodal extension with the Remo AI Agent framework,
//! providing file parsing hooks and multimodal tools.

use std::sync::Arc;

use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::MutationBatch;
use remo_runtime_contract::StateError;
use remo_runtime_contract::model::Phase;
use remo_runtime_contract::registry_spec::AgentSpec;

use crate::config::MultimodalConfigKey;
use crate::hooks::MultimodalBeforeInferenceHook;
use crate::tools::{AnalyzeImageTool, DescribeImageTool, ParseFileTool};

/// Stable plugin name for the multimodal extension.
pub const MULTIMODAL_PLUGIN_NAME: &str = "multimodal";

/// Tool ID for the parse-file tool.
pub const PARSE_FILE_TOOL_ID: &str = "parse_file";

/// Tool ID for the describe-image tool.
pub const DESCRIBE_IMAGE_TOOL_ID: &str = "describe_image";

/// Tool ID for the analyze-images tool.
pub const ANALYZE_IMAGES_TOOL_ID: &str = "analyze_images";

/// Multimodal support plugin.
///
/// Provides file parsing, multimodal content handling, and image description
/// capabilities for the Remo AI Agent framework.
///
/// # Registered Components
///
/// - **Phase hook**: [`MultimodalBeforeInferenceHook`] on `BeforeInference` —
///   preprocesses multimodal content in incoming messages.
/// - **Tool**: [`ParseFileTool`] — parses files and extracts text content.
/// - **Tool**: [`DescribeImageTool`] — describes images from URLs or base64 data.
/// - **Tool**: [`AnalyzeImageTool`] — batch-analyzes multiple images.
pub struct MultimodalPlugin;

impl Plugin for MultimodalPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: MULTIMODAL_PLUGIN_NAME,
        }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        // Register the BeforeInference hook for multimodal preprocessing
        registrar.register_phase_hook(
            MULTIMODAL_PLUGIN_NAME,
            Phase::BeforeInference,
            MultimodalBeforeInferenceHook,
        )?;

        // Register the parse_file tool
        registrar.register_tool(
            PARSE_FILE_TOOL_ID,
            Arc::new(ParseFileTool::new()),
        )?;

        // Register the describe_image tool
        registrar.register_tool(
            DESCRIBE_IMAGE_TOOL_ID,
            Arc::new(DescribeImageTool::new()),
        )?;

        // Register the analyze_images tool
        registrar.register_tool(
            ANALYZE_IMAGES_TOOL_ID,
            Arc::new(AnalyzeImageTool::new()),
        )?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<MultimodalConfigKey>()
                .with_display_name("Multimodal")
                .with_description("File parsing, multimodal content handling, and image description settings.")
                .with_category("multimodal")
                .with_editor("multimodal"),
        ]
    }

    fn on_activate(
        &self,
        _agent_spec: &AgentSpec,
        _patch: &mut MutationBatch,
    ) -> Result<(), StateError> {
        // Read configuration to validate it. State seeding would happen here
        // if the plugin maintained persistent state keys.
        let _config = _agent_spec.config::<MultimodalConfigKey>()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_descriptor_name() {
        let plugin = MultimodalPlugin;
        assert_eq!(plugin.descriptor().name, "multimodal");
    }

    #[test]
    fn plugin_has_config_schemas() {
        let plugin = MultimodalPlugin;
        let schemas = plugin.config_schemas();
        assert_eq!(schemas.len(), 1);
    }

    #[test]
    fn plugin_constants() {
        assert_eq!(MULTIMODAL_PLUGIN_NAME, "multimodal");
        assert_eq!(PARSE_FILE_TOOL_ID, "parse_file");
        assert_eq!(DESCRIBE_IMAGE_TOOL_ID, "describe_image");
        assert_eq!(ANALYZE_IMAGES_TOOL_ID, "analyze_images");
    }
}
