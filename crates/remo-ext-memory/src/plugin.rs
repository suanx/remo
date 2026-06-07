//! Memory plugin: registers state keys, hooks, tools, and config for the memory system.

use std::sync::Arc;

use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::{KeyScope, MutationBatch, StateKeyOptions};
use remo_runtime_contract::StateError;
use remo_runtime_contract::model::Phase;
use remo_runtime_contract::registry_spec::AgentSpec;

use crate::config::MemoryConfigKey;
use crate::hooks::MemoryBeforeInferenceHook;
use crate::state::MemoryStateKey;
use crate::tools::{ListMemoryTool, RecallMemoryTool, StoreMemoryTool};

/// Stable plugin name for the memory extension.
pub const MEMORY_PLUGIN_NAME: &str = "memory";

/// Memory extension plugin.
///
/// Registers:
/// - [`MemoryStateKey`]: thread-scoped memory state
/// - A `BeforeInference` phase hook that retrieves relevant memories
/// - Three tools: store, recall, and list
pub struct MemoryPlugin;

impl Plugin for MemoryPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: MEMORY_PLUGIN_NAME,
        }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        registrar.register_key::<MemoryStateKey>(StateKeyOptions {
            persistent: true,
            retain_on_uninstall: false,
            scope: KeyScope::Thread,
        })?;

        registrar.register_phase_hook(
            MEMORY_PLUGIN_NAME,
            Phase::BeforeInference,
            MemoryBeforeInferenceHook,
        )?;

        registrar.register_tool("memory:store", Arc::new(StoreMemoryTool))?;
        registrar.register_tool("memory:recall", Arc::new(RecallMemoryTool))?;
        registrar.register_tool("memory:list", Arc::new(ListMemoryTool))?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<MemoryConfigKey>()
                .with_display_name("Memory")
                .with_description("Short-term and long-term memory with retrieval and consolidation.")
                .with_category("cognition")
                .with_editor("memory"),
        ]
    }

    fn on_activate(
        &self,
        _agent_spec: &AgentSpec,
        _patch: &mut MutationBatch,
    ) -> Result<(), StateError> {
        // Memory starts empty; no initial seeding needed.
        Ok(())
    }
}
