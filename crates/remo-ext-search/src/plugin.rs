//! Search plugin: registers config schema and tools for web search and page fetching.

use std::sync::Arc;

use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime_contract::StateError;
use remo_runtime_contract::registry_spec::AgentSpec;
use remo_runtime_contract::state::MutationBatch;

use crate::config::SearchConfigKey;
use crate::tools::{FetchWebpageTool, SearchWebTool};

/// Stable plugin name for the search extension.
pub const SEARCH_PLUGIN_NAME: &str = "search";

/// Search extension plugin.
///
/// Registers:
/// - `SearchConfigKey`: agent-level configuration for search provider, API key, etc.
/// - `SearchWebTool` (`search:web`): perform web searches via the configured provider.
/// - `FetchWebpageTool` (`search:fetch`): fetch and extract webpage content.
pub struct SearchPlugin;

impl Plugin for SearchPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: SEARCH_PLUGIN_NAME,
        }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        registrar.register_tool("search:web", Arc::new(SearchWebTool))?;
        registrar.register_tool("search:fetch", Arc::new(FetchWebpageTool))?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![ConfigSchema::for_key::<SearchConfigKey>()
            .with_display_name("Web Search")
            .with_description(
                "Configure web search providers and API keys for the search and fetch tools.",
            )
            .with_category("tools")
            .with_editor("search")]
    }

    fn on_activate(
        &self,
        _agent_spec: &AgentSpec,
        _patch: &mut MutationBatch,
    ) -> Result<(), StateError> {
        // No initial state seeding needed.
        Ok(())
    }
}
