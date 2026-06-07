//! Workflow DSL — declarative specification types for DAG-based workflows.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A complete workflow specification defining a directed acyclic graph of nodes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowSpec {
    /// Unique identifier for this workflow definition.
    pub id: String,
    /// Human-readable workflow name.
    pub name: String,
    /// The set of nodes in this workflow.
    pub nodes: Vec<NodeSpec>,
    /// The directed edges connecting nodes (DAG structure).
    pub edges: Vec<EdgeSpec>,
}

/// Specification for a single node within a workflow.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeSpec {
    /// Unique identifier for this node within the workflow.
    pub id: String,
    /// The type of execution this node performs.
    pub node_type: NodeType,
    /// IDs of nodes that must complete before this node can execute.
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// The type of execution a node performs.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// Execute an LLM call with the given prompt template.
    Llm {
        /// Prompt template string (may contain variable placeholders).
        prompt: String,
    },
    /// Execute a registered tool by ID with the given arguments.
    Tool {
        /// ID of the tool to invoke.
        tool_id: String,
        /// Arguments to pass to the tool.
        args: serde_json::Value,
    },
    /// Evaluate a boolean expression and branch accordingly.
    Condition {
        /// Boolean expression string to evaluate.
        expression: String,
    },
    /// Pass input through without modification.
    Passthrough,
}

/// A directed edge in the workflow graph, connecting `from` to `to`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EdgeSpec {
    /// ID of the source node.
    pub from: String,
    /// ID of the target node.
    pub to: String,
}
