//! State management for workflow execution.

use std::collections::HashMap;

use remo_runtime::state::{KeyScope, MergeStrategy, StateKey};
use serde::{Deserialize, Serialize};

use crate::dsl::WorkflowSpec;

/// Current time in milliseconds since Unix epoch.
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Status of a workflow or individual node within a workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
    /// The workflow/node has not started yet.
    Pending,
    /// The workflow/node is currently executing.
    Running,
    /// The workflow/node completed successfully.
    Completed,
    /// The workflow/node failed with an error message.
    Failed { error: String },
    /// The workflow/node was cancelled.
    Cancelled,
}

impl Default for WorkflowStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// The result of executing a single workflow node.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeResult {
    /// ID of the node that was executed.
    pub node_id: String,
    /// Final status of the node execution.
    pub status: WorkflowStatus,
    /// Optional output produced by the node (JSON value).
    pub output: Option<serde_json::Value>,
}

/// Complete state of a workflow execution instance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Unique identifier for this workflow execution.
    pub workflow_id: String,
    /// Overall status of the workflow.
    pub status: WorkflowStatus,
    /// Results for each node that has been (or is being) executed.
    pub node_results: HashMap<String, NodeResult>,
    /// Timestamp (epoch millis) when the workflow started, if it has started.
    pub started_at: Option<i64>,
    /// Timestamp (epoch millis) when the workflow completed, if it has completed.
    pub completed_at: Option<i64>,
}

/// Actions that can be applied to a workflow state to drive its lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowAction {
    /// Start a new workflow execution with the given spec.
    Start { spec: WorkflowSpec },
    /// Record the result of a completed node.
    UpdateNode {
        /// ID of the node that completed.
        node_id: String,
        /// Result of the node execution.
        result: NodeResult,
    },
    /// Mark the entire workflow as completed.
    Complete,
    /// Mark the entire workflow as failed with an error message.
    Fail { error: String },
    /// Cancel the workflow.
    Cancel,
}

/// State key for tracking workflow execution state.
///
/// Uses `Exclusive` merge strategy (last-writer-wins semantics) and is
/// scoped to the current run.
pub struct WorkflowStateKey;

impl StateKey for WorkflowStateKey {
    const KEY: &'static str = "workflow_state";
    const MERGE: MergeStrategy = MergeStrategy::Exclusive;
    const SCOPE: KeyScope = KeyScope::Run;

    type Value = WorkflowState;
    type Update = WorkflowAction;

    fn apply(value: &mut Self::Value, update: Self::Update) {
        match update {
            WorkflowAction::Start { spec } => {
                value.workflow_id = spec.id.clone();
                value.status = WorkflowStatus::Running;
                value.started_at = Some(now_ms());
            }
            WorkflowAction::UpdateNode { node_id, result } => {
                value.node_results.insert(node_id, result);
            }
            WorkflowAction::Complete => {
                value.status = WorkflowStatus::Completed;
                value.completed_at = Some(now_ms());
            }
            WorkflowAction::Fail { error } => {
                value.status = WorkflowStatus::Failed { error };
                value.completed_at = Some(now_ms());
            }
            WorkflowAction::Cancel => {
                value.status = WorkflowStatus::Cancelled;
                value.completed_at = Some(now_ms());
            }
        }
    }
}
