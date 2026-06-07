//! OpenCode plugin — registers OpenCode Zen provider, free model discovery, and CLI tools.

use std::sync::Arc;
use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::MutationBatch;
use remo_runtime_contract::StateError;
use remo_runtime_contract::registry_spec::AgentSpec;
use crate::config::OpenCodeConfigKey;
use crate::tools::{OpenCodeExecTool, OpenCodeListModelsTool, OpenCodeCheckCliTool};

pub const OPENCODE_PLUGIN_NAME: &str = "opencode";
pub const OPENCODE_EXEC_TOOL_ID: &str = "opencode:exec";
pub const OPENCODE_LIST_MODELS_TOOL_ID: &str = "opencode:list_models";
pub const OPENCODE_CHECK_CLI_TOOL_ID: &str = "opencode:check_cli";

/// OpenCode integration plugin.
///
/// Provides:
/// - OpenCode Zen provider with auto-discovered free models
/// - OpenCode CLI execution tool for code generation
/// - Model listing tool
/// - CLI installation check tool
pub struct OpenCodePlugin;

impl Plugin for OpenCodePlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor { name: OPENCODE_PLUGIN_NAME }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        registrar.register_tool(OPENCODE_EXEC_TOOL_ID, Arc::new(OpenCodeExecTool))?;
        registrar.register_tool(OPENCODE_LIST_MODELS_TOOL_ID, Arc::new(OpenCodeListModelsTool))?;
        registrar.register_tool(OPENCODE_CHECK_CLI_TOOL_ID, Arc::new(OpenCodeCheckCliTool))?;
        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<OpenCodeConfigKey>()
                .with_display_name("OpenCode")
                .with_description("OpenCode CLI integration and free model discovery via OpenCode Zen")
                .with_category("opencode")
                .with_editor("opencode"),
        ]
    }

    fn on_activate(&self, agent_spec: &AgentSpec, _patch: &mut MutationBatch) -> Result<(), StateError> {
        let config = agent_spec.config::<OpenCodeConfigKey>()?;
        tracing::info!(
            auto_discover = config.auto_discover_free_models,
            cli_enabled = config.enable_cli_tool,
            "OpenCode plugin activated"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_descriptor() {
        let p = OpenCodePlugin;
        assert_eq!(p.descriptor().name, "opencode");
    }

    #[test]
    fn plugin_has_config_schemas() {
        let p = OpenCodePlugin;
        assert!(!p.config_schemas().is_empty());
    }
}
