//! Typed tools for interacting with the workflow engine.

use async_trait::async_trait;
use remo_runtime_contract::contract::tool::{ToolCallContext, ToolError, ToolOutput, ToolResult, TypedTool};
use schemars::JsonSchema;
use serde::Deserialize;
use crate::dsl::WorkflowSpec;
use crate::executor::WorkflowExecutor;
use crate::state::{WorkflowState, WorkflowStateKey, WorkflowStatus};

/// Arguments for the `start_workflow` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StartWorkflowArgs {
    /// JSON-serialised workflow specification.
    pub workflow_spec: serde_json::Value,
}

/// Tool that starts a new workflow execution.
///
/// Deserialises the provided JSON into a [`WorkflowSpec`], creates a fresh
/// [`WorkflowState`], runs the DAG executor, and returns the `workflow_id`.
pub struct StartWorkflowTool;

#[async_trait]
impl TypedTool for StartWorkflowTool {
    type Args = StartWorkflowArgs;

    fn tool_id(&self) -> &str {
        "start_workflow"
    }

    fn name(&self) -> &str {
        "start_workflow"
    }

    fn description(&self) -> &str {
        "Start a new DAG-based workflow execution from a declarative specification."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let spec: WorkflowSpec =
            serde_json::from_value(args.workflow_spec).map_err(|e| {
                ToolError::InvalidArguments(format!("Invalid workflow spec: {e}"))
            })?;

        let mut state = WorkflowState {
            workflow_id: spec.id.clone(),
            status: WorkflowStatus::Pending,
            ..Default::default()
        };

        // Read max_parallel from workflow config if available, else default.
        let max_parallel = ctx
            .agent_spec
            .config::<crate::config::WorkflowConfigKey>()
            .unwrap_or_default()
            .max_parallel;

        let executor = WorkflowExecutor::new(max_parallel);

        // For now, use the node executor dispatch as the runner.
        executor
            .execute(&spec, &mut state, |node| {
                Box::pin(async move {
                    let executor_fn = crate::nodes::resolve_executor(&node.node_type);
                    // We have no real ToolCallContext here, so create a minimal one.
                    let minimal_ctx = ToolCallContext::test_default();
                    executor_fn.execute(&node, &minimal_ctx).await
                })
            })
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Workflow execution failed: {e}")))?;

        let workflow_id = state.workflow_id.clone();

        // Build a command that writes the workflow state.
        let mut command = remo_runtime_contract::state::StateCommand::new();
        command
            .patch
            .update::<WorkflowStateKey>(crate::state::WorkflowAction::Start {
                spec: WorkflowSpec {
                    id: state.workflow_id.clone(),
                    ..spec
                },
            });
        // Update all node results
        for (_, node_result) in &state.node_results {
            command.patch.update::<WorkflowStateKey>(
                crate::state::WorkflowAction::UpdateNode {
                    node_id: node_result.node_id.clone(),
                    result: node_result.clone(),
                },
            );
        }
        // Mark completion/failure
        match &state.status {
            WorkflowStatus::Completed => {
                command
                    .patch
                    .update::<WorkflowStateKey>(crate::state::WorkflowAction::Complete);
            }
            WorkflowStatus::Failed { error } => {
                command.patch.update::<WorkflowStateKey>(
                    crate::state::WorkflowAction::Fail {
                        error: error.clone(),
                    },
                );
            }
            WorkflowStatus::Cancelled => {
                command
                    .patch
                    .update::<WorkflowStateKey>(crate::state::WorkflowAction::Cancel);
            }
            _ => {}
        }

        Ok(ToolOutput::with_command(
            ToolResult::success(
                "start_workflow",
                serde_json::json!({
                    "workflow_id": workflow_id,
                    "status": state.status,
                }),
            ),
            command,
        ))
    }
}

/// Arguments for the `workflow_status` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkflowStatusArgs {
    /// The ID of the workflow to query.
    pub workflow_id: String,
}

/// Tool that returns the current status and node results of a workflow.
pub struct WorkflowStatusTool;

#[async_trait]
impl TypedTool for WorkflowStatusTool {
    type Args = WorkflowStatusArgs;

    fn tool_id(&self) -> &str {
        "workflow_status"
    }

    fn name(&self) -> &str {
        "workflow_status"
    }

    fn description(&self) -> &str {
        "Get the current status and node-level results for a workflow execution."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let state = ctx.state::<WorkflowStateKey>().ok_or_else(|| {
            ToolError::NotFound(format!(
                "No workflow state found for workflow_id={}",
                args.workflow_id
            ))
        })?;

        if state.workflow_id != args.workflow_id {
            return Err(ToolError::NotFound(format!(
                "Workflow not found: {}",
                args.workflow_id
            )));
        }

        Ok(ToolOutput::new(ToolResult::success(
            "workflow_status",
            serde_json::json!({
                "workflow_id": state.workflow_id,
                "status": state.status,
                "node_results": state.node_results,
                "started_at": state.started_at,
                "completed_at": state.completed_at,
            }),
        )))
    }
}
