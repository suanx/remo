use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::{KeyScope, MergeStrategy, MutationBatch, StateKey, StateKeyOptions};
use remo_runtime_contract::StateError;
use remo_runtime_contract::registry_spec::AgentSpec;

use crate::config::{SandboxConfig, SandboxConfigKey};
use crate::hook::SandboxToolGateHook;
use crate::policy::SandboxPolicy;

/// Stable plugin name for the sandbox extension.
pub const SANDBOX_PLUGIN_NAME: &str = "sandbox";

// ---------------------------------------------------------------------------
// SandboxStateKey — thread-scoped sandbox policy state
// ---------------------------------------------------------------------------

/// State key that holds the resolved sandbox policy in runtime state.
pub struct SandboxStateKey;

impl StateKey for SandboxStateKey {
    const KEY: &'static str = "sandbox_policy";
    const MERGE: MergeStrategy = MergeStrategy::Exclusive;
    const SCOPE: KeyScope = KeyScope::Thread;

    type Value = SandboxPolicy;
    type Update = SandboxPolicy;

    fn apply(value: &mut Self::Value, update: Self::Update) {
        *value = update;
    }
}

// ---------------------------------------------------------------------------
// SandboxPlugin — main plugin entry point
// ---------------------------------------------------------------------------

/// Sandbox extension plugin for the Remo AI Agent framework.
///
/// Registers:
/// - [`SandboxStateKey`]: thread-scoped sandbox policy state
/// - A `ToolGate` hook that validates and intercepts high-risk tool calls
///
/// Reads `SandboxConfig` from `AgentSpec.sections["sandbox"]` during activation
/// and constructs a [`SandboxPolicy`] from it.
pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: SANDBOX_PLUGIN_NAME,
        }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        // Register the sandbox policy state key
        registrar.register_key::<SandboxStateKey>(StateKeyOptions {
            persistent: false,
            retain_on_uninstall: false,
            scope: KeyScope::Thread,
        })?;

        // Register the tool gate hook for sandbox interception
        registrar.register_tool_gate_hook(
            SANDBOX_PLUGIN_NAME,
            SandboxToolGateHook::new(SandboxPolicy::default()),
        )?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<SandboxConfigKey>()
                .with_display_name("Sandbox")
                .with_description("Process-level sandboxing for secure tool execution.")
                .with_category("safety")
                .with_editor("sandbox"),
        ]
    }

    fn on_activate(
        &self,
        agent_spec: &AgentSpec,
        patch: &mut MutationBatch,
    ) -> Result<(), StateError> {
        // Read sandbox config (defaults to SandboxConfig::default() if absent)
        let config: SandboxConfig = agent_spec.config::<SandboxConfigKey>()?;

        // Build runtime policy from config
        let policy = SandboxPolicy::from(&config);

        // Seed the state with the resolved policy
        patch.update::<SandboxStateKey>(policy);

        Ok(())
    }
}
