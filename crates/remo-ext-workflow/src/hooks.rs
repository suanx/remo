//! Phase hooks for workflow progress tracking.

use async_trait::async_trait;

use remo_runtime::PhaseHook;
use remo_runtime_contract::StateError;
use remo_runtime_contract::StateCommand;

use crate::state::{WorkflowState, WorkflowStateKey};

/// Phase hook that tracks workflow progress after tool execution.
///
/// Runs on `AfterToolExecute` to observe tool results that may be part of
/// an active workflow and update progress tracking accordingly.
pub struct WorkflowPhaseHook;

#[async_trait]
impl PhaseHook for WorkflowPhaseHook {
    async fn run(
        &self,
        ctx: &remo_runtime::PhaseContext,
    ) -> Result<StateCommand, StateError> {
        let cmd = StateCommand::new();

        // Read workflow state — if no workflow is running, this is a no-op.
        let state: &WorkflowState = match ctx.state::<WorkflowStateKey>() {
            Some(s) => s,
            None => return Ok(cmd),
        };

        // Only act if a workflow is actively running.
        if !matches!(
            state.status,
            crate::state::WorkflowStatus::Running
        ) {
            return Ok(cmd);
        }

        // Placeholder for future progress tracking:
        // - Observe which tool was executed (via ctx.tool_name)
        // - Correlate with workflow node results
        // - Emit progress events or context messages

        Ok(cmd)
    }
}
