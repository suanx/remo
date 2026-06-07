//! Workflow plugin: registers state keys, hooks, tools, and config for the workflow engine.

use std::sync::Arc;

use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::{KeyScope, MutationBatch, StateKeyOptions};
use remo_runtime_contract::StateError;
use remo_runtime_contract::model::Phase;
use remo_runtime_contract::registry_spec::AgentSpec;

use crate::config::WorkflowConfigKey;
use crate::hooks::WorkflowPhaseHook;
use crate::state::WorkflowStateKey;
use crate::tools::{StartWorkflowTool, WorkflowStatusTool};

/// Stable plugin name for the workflow extension.
pub const WORKFLOW_PLUGIN_NAME: &str = "workflow";

/// Workflow extension plugin.
///
/// Registers:
/// - [`WorkflowStateKey`]: run-scoped workflow state
/// - An `AfterToolExecute` phase hook for workflow progress tracking
/// - Two tools: start_workflow and workflow_status
pub struct WorkflowPlugin;

impl Plugin for WorkflowPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: WORKFLOW_PLUGIN_NAME,
        }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        registrar.register_key::<WorkflowStateKey>(StateKeyOptions {
            persistent: false,
            retain_on_uninstall: false,
            scope: KeyScope::Run,
        })?;

        registrar.register_phase_hook(
            WORKFLOW_PLUGIN_NAME,
            Phase::AfterToolExecute,
            WorkflowPhaseHook,
        )?;

        registrar.register_tool("start_workflow", Arc::new(StartWorkflowTool))?;
        registrar.register_tool("workflow_status", Arc::new(WorkflowStatusTool))?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<WorkflowConfigKey>()
                .with_display_name("Workflow")
                .with_description("DAG-based workflow execution engine.")
                .with_category("automation")
                .with_editor("workflow"),
        ]
    }

    fn on_activate(
        &self,
        _agent_spec: &AgentSpec,
        _patch: &mut MutationBatch,
    ) -> Result<(), StateError> {
        // Workflow state is initialized when a workflow is started; no seeding needed.
        Ok(())
    }
}
