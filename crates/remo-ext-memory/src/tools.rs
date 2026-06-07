//! Tools for memory management: store, recall, and list memories.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;

use remo_runtime_contract::contract::tool::{ToolCallContext, ToolError, ToolOutput, ToolResult};
use remo_runtime_contract::StateCommand;

use crate::state::{MemoryAction, MemoryEntry, MemoryStateKey};

// ---------------------------------------------------------------------------
// StoreMemoryTool
// ---------------------------------------------------------------------------

/// Arguments for storing a new memory.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StoreMemoryArgs {
    /// The textual content to remember.
    pub content: String,
    /// Importance score (0.0 – 1.0). Higher = more likely to be retrieved.
    #[serde(default = "default_importance")]
    pub importance: f64,
    /// Optional tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_importance() -> f64 {
    0.5
}

/// Tool that stores a new memory entry.
pub struct StoreMemoryTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for StoreMemoryTool {
    type Args = StoreMemoryArgs;

    fn tool_id(&self) -> &str {
        "memory:store"
    }

    fn name(&self) -> &str {
        "Store Memory"
    }

    fn description(&self) -> &str {
        "Store a new memory entry in short-term memory."
    }

    async fn execute(
        &self,
        args: Self::Args,
        _ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let entry = MemoryEntry {
            id: format!("mem_{}", now_ms()),
            content: args.content,
            timestamp: now_ms(),
            importance: args.importance.clamp(0.0, 1.0),
            tags: args.tags,
        };

        let mut cmd = StateCommand::new();
        cmd.patch.update::<MemoryStateKey>(MemoryAction::Store {
            entry: entry.clone(),
        });

        let result = ToolResult::success(
            "memory:store",
            serde_json::json!({
                "id": entry.id,
                "message": "Memory stored successfully"
            }),
        );

        Ok(ToolOutput::with_command(result, cmd))
    }
}

// ---------------------------------------------------------------------------
// RecallMemoryTool
// ---------------------------------------------------------------------------

/// Arguments for recalling memories.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecallMemoryArgs {
    /// Search query to find relevant memories.
    pub query: String,
    /// Maximum number of results to return.
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}

/// Tool that recalls memories matching a query.
pub struct RecallMemoryTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for RecallMemoryTool {
    type Args = RecallMemoryArgs;

    fn tool_id(&self) -> &str {
        "memory:recall"
    }

    fn name(&self) -> &str {
        "Recall Memory"
    }

    fn description(&self) -> &str {
        "Search and retrieve relevant memories from both short-term and long-term storage."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        // Read state from the snapshot
        let state = ctx
            .snapshot
            .get::<MemoryStateKey>()
            .cloned()
            .unwrap_or_default();

        let retriever = crate::retrieval::MemoryRetriever;
        let results = retriever.retrieve(&args.query, &state, args.top_k);

        let data = serde_json::json!({
            "memories": results.iter().map(|m| serde_json::json!({
                "id": m.id,
                "content": m.content,
                "importance": m.importance,
                "tags": m.tags,
            })).collect::<Vec<_>>(),
            "count": results.len(),
        });

        let result = ToolResult::success("memory:recall", data);
        Ok(result.into())
    }
}

// ---------------------------------------------------------------------------
// ListMemoryTool
// ---------------------------------------------------------------------------

/// Arguments for listing memories.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListMemoryArgs {
    /// Memory type to list: "short_term", "long_term", or "all" (default: "all").
    #[serde(default = "default_memory_type")]
    pub memory_type: String,
}

fn default_memory_type() -> String {
    "all".into()
}

/// Tool that lists stored memories.
pub struct ListMemoryTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for ListMemoryTool {
    type Args = ListMemoryArgs;

    fn tool_id(&self) -> &str {
        "memory:list"
    }

    fn name(&self) -> &str {
        "List Memories"
    }

    fn description(&self) -> &str {
        "List all stored memories, optionally filtered by type (short_term, long_term, all)."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let state = ctx
            .snapshot
            .get::<MemoryStateKey>()
            .cloned()
            .unwrap_or_default();

        let (short_term, long_term) = match args.memory_type.as_str() {
            "short_term" => (state.short_term.clone(), vec![]),
            "long_term" => (vec![], state.long_term.clone()),
            _ => (state.short_term.clone(), state.long_term.clone()),
        };

        let data = serde_json::json!({
            "short_term": short_term.len(),
            "long_term": long_term.len(),
            "total": short_term.len() + long_term.len(),
        });

        let result = ToolResult::success("memory:list", data);
        Ok(result.into())
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
