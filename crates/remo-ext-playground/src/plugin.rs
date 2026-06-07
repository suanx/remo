//! Playground plugin: registers state keys and config for replay/scoring.

use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::{KeyScope, MutationBatch, StateKeyOptions};
use remo_runtime_contract::StateError;
use remo_runtime_contract::registry_spec::AgentSpec;

use crate::config::PlaygroundConfigKey;
use crate::state::PlaygroundStateKey;

/// Stable plugin name for the playground extension.
pub const PLAYGROUND_PLUGIN_NAME: &str = "playground";

/// Playground extension plugin.
///
/// Registers:
/// - [`PlaygroundStateKey`]: thread-scoped replay and scoring state
/// - Config schema for playground settings
///
/// The playground provides tools for recording conversation replays,
/// evaluating session quality, and comparing runs.
pub struct PlaygroundPlugin;

impl Plugin for PlaygroundPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: PLAYGROUND_PLUGIN_NAME,
        }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        registrar.register_key::<PlaygroundStateKey>(StateKeyOptions {
            persistent: true,
            retain_on_uninstall: false,
            scope: KeyScope::Thread,
        })?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<PlaygroundConfigKey>()
                .with_display_name("Playground")
                .with_description("Replay recording, evaluation scoring, and run comparison.")
                .with_category("evaluation")
                .with_editor("playground"),
        ]
    }

    fn on_activate(
        &self,
        _agent_spec: &AgentSpec,
        _patch: &mut MutationBatch,
    ) -> Result<(), StateError> {
        // Playground state is empty on activation; replays are recorded at runtime.
        Ok(())
    }
}
