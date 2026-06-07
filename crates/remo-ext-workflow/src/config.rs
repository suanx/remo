//! Configuration for the workflow engine extension.

use remo_runtime_contract::PluginConfigKey;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Maximum number of nodes that can execute in parallel within a workflow layer.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct WorkflowConfig {
    /// Maximum number of nodes that can execute concurrently within a single
    /// topological layer. Defaults to 4.
    pub max_parallel: usize,
    /// Global timeout for the entire workflow execution in milliseconds.
    /// Defaults to 30_000 (30 seconds).
    pub timeout_ms: u64,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_parallel: 4,
            timeout_ms: 30_000,
        }
    }
}

/// [`PluginConfigKey`] binding for workflow config in agent specs.
pub struct WorkflowConfigKey;

impl PluginConfigKey for WorkflowConfigKey {
    const KEY: &'static str = "workflow";
    type Config = WorkflowConfig;
}
