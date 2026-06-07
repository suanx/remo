//! Node executors — trait and implementations for individual workflow node types.

use async_trait::async_trait;
use remo_runtime_contract::contract::tool::ToolCallContext;

use crate::dsl::{NodeSpec, NodeType};

/// Trait for executing a workflow node.
#[async_trait]
pub trait NodeExecutor: Send + Sync {
    /// Execute the given node spec and return its JSON output, or an error string.
    async fn execute(&self, node: &NodeSpec, ctx: &ToolCallContext) -> Result<serde_json::Value, String>;
}

/// Placeholder LLM node — returns a mock response.
pub struct LlmNode;

#[async_trait]
impl NodeExecutor for LlmNode {
    async fn execute(&self, node: &NodeSpec, _ctx: &ToolCallContext) -> Result<serde_json::Value, String> {
        match &node.node_type {
            NodeType::Llm { prompt } => {
                // Placeholder: In a real implementation this would call the LLM.
                Ok(serde_json::json!({
                    "prompt": prompt,
                    "response": format!("[LLM placeholder response for: {prompt})"),
                }))
            }
            _ => Err(format!(
                "LlmNode received non-LLM node type: {:?}",
                node.node_type
            )),
        }
    }
}

/// Tool node — delegates execution to a registered tool by ID.
pub struct ToolNode;

#[async_trait]
impl NodeExecutor for ToolNode {
    async fn execute(&self, node: &NodeSpec, _ctx: &ToolCallContext) -> Result<serde_json::Value, String> {
        match &node.node_type {
            NodeType::Tool { tool_id, args } => {
                // Placeholder: In a real implementation this would look up and
                // invoke the tool via the tool registry / ToolCallContext.
                Ok(serde_json::json!({
                    "tool_id": tool_id,
                    "args": args,
                    "result": format!("[Tool placeholder: {tool_id}]"),
                }))
            }
            _ => Err(format!(
                "ToolNode received non-tool node type: {:?}",
                node.node_type
            )),
        }
    }
}

/// Condition node — evaluates a simple boolean expression.
///
/// Currently uses a naive keyword-based evaluator as a placeholder.
pub struct ConditionNode;

#[async_trait]
impl NodeExecutor for ConditionNode {
    async fn execute(&self, node: &NodeSpec, _ctx: &ToolCallContext) -> Result<serde_json::Value, String> {
        match &node.node_type {
            NodeType::Condition { expression } => {
                // Placeholder evaluation: treat "true"/"1" as true, everything else as false.
                let result = matches!(expression.as_str(), "true" | "1");
                Ok(serde_json::json!({
                    "expression": expression,
                    "result": result,
                }))
            }
            _ => Err(format!(
                "ConditionNode received non-condition node type: {:?}",
                node.node_type
            )),
        }
    }
}

/// Passthrough node — returns a no-op acknowledgement.
pub struct PassthroughNode;

#[async_trait]
impl NodeExecutor for PassthroughNode {
    async fn execute(&self, node: &NodeSpec, _ctx: &ToolCallContext) -> Result<serde_json::Value, String> {
        match &node.node_type {
            NodeType::Passthrough => {
                Ok(serde_json::json!({
                    "node_id": node.id,
                    "passthrough": true,
                }))
            }
            _ => Err(format!(
                "PassthroughNode received non-passthrough node type: {:?}",
                node.node_type
            )),
        }
    }
}

/// Resolve the appropriate [`NodeExecutor`] for a given node type.
pub fn resolve_executor(node_type: &NodeType) -> Box<dyn NodeExecutor> {
    match node_type {
        NodeType::Llm { .. } => Box::new(LlmNode),
        NodeType::Tool { .. } => Box::new(ToolNode),
        NodeType::Condition { .. } => Box::new(ConditionNode),
        NodeType::Passthrough => Box::new(PassthroughNode),
    }
}
