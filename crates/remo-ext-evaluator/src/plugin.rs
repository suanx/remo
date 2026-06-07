//! Evaluator plugin: registers state keys, config schemas, and tools
//! for LLM-as-judge online evaluation.

use std::sync::Arc;

use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::{KeyScope, MutationBatch, StateKeyOptions};
use remo_runtime_contract::StateError;
use remo_runtime_contract::registry_spec::AgentSpec;

use crate::config::EvaluatorConfigKey;
use crate::state::EvaluationStateKey;
use crate::tools::{EvaluateConversationTool, EvaluateResponseTool};

/// Stable plugin name for the evaluator extension.
pub const EVALUATOR_PLUGIN_NAME: &str = "evaluator";

/// Evaluator extension plugin.
///
/// Registers:
/// - [`EvaluationStateKey`]: thread-scoped evaluation history
/// - Two tools: evaluate_response and evaluate_conversation
pub struct EvaluatorPlugin;

impl Plugin for EvaluatorPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: EVALUATOR_PLUGIN_NAME,
        }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        registrar.register_key::<EvaluationStateKey>(StateKeyOptions {
            persistent: true,
            retain_on_uninstall: false,
            scope: KeyScope::Thread,
        })?;

        registrar.register_tool(
            "evaluate:evaluate_response",
            Arc::new(EvaluateResponseTool),
        )?;
        registrar.register_tool(
            "evaluate:evaluate_conversation",
            Arc::new(EvaluateConversationTool),
        )?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![ConfigSchema::for_key::<EvaluatorConfigKey>()
            .with_display_name("Evaluator")
            .with_description(
                "LLM-as-judge evaluation with configurable criteria and auto-evaluation.",
            )
            .with_category("evaluation")
            .with_editor("evaluator")]
    }

    fn on_activate(
        &self,
        _agent_spec: &AgentSpec,
        _patch: &mut MutationBatch,
    ) -> Result<(), StateError> {
        // Evaluation history starts empty; no initial seeding needed.
        Ok(())
    }
}
