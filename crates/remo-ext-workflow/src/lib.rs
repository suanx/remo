//! DAG-based workflow execution engine for the Remo AI Agent framework.
//!
//! This crate provides:
//! - A declarative workflow DSL (`WorkflowSpec`, `NodeSpec`, `EdgeSpec`)
//! - A topological-sort executor with parallel node execution
//! - Node executors for LLM, tool, condition, and passthrough nodes
//! - Typed tools (`StartWorkflowTool`, `WorkflowStatusTool`) for agent interaction
//! - Phase hooks for tracking workflow node completion
//! - State management for workflow lifecycle
//! - Plugin integration via `WorkflowPlugin`

pub mod config;
pub mod dsl;
pub mod executor;
pub mod hooks;
pub mod nodes;
pub mod plugin;
pub mod state;
pub mod tools;

pub use config::{WorkflowConfig, WorkflowConfigKey};
pub use dsl::{EdgeSpec, NodeType, NodeSpec, WorkflowSpec};
pub use executor::WorkflowExecutor;
pub use hooks::WorkflowPhaseHook;
pub use nodes::{ConditionNode, LlmNode, NodeExecutor, PassthroughNode, ToolNode};
pub use plugin::WorkflowPlugin;
pub use state::{NodeResult, WorkflowAction, WorkflowState, WorkflowStateKey, WorkflowStatus};
pub use tools::{StartWorkflowTool, WorkflowStatusTool};
