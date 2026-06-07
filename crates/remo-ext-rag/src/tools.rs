//! Typed tools for interacting with the RAG system.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;

use remo_runtime_contract::contract::tool::{
    ToolCallContext, ToolError, ToolOutput, ToolResult,
};
use remo_runtime_contract::StateCommand;

use crate::config::{RagConfig, RagConfigKey};
use crate::pipeline::RagPipeline;
use crate::retriever::KeywordRetriever;
use crate::state::{RagAction, RagDocumentState, RagDocumentStateKey};

// ---------------------------------------------------------------------------
// IngestDocumentTool
// ---------------------------------------------------------------------------

/// Arguments for ingesting a new document into the RAG store.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct IngestDocumentArgs {
    /// Human-readable name for the document.
    pub name: String,
    /// The full text content of the document.
    pub content: String,
}

/// Tool that ingests a document into the RAG store.
///
/// Chunks the document according to the active RAG config and stores it
/// in the thread-scoped document state.
pub struct IngestDocumentTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for IngestDocumentTool {
    type Args = IngestDocumentArgs;

    fn tool_id(&self) -> &str {
        "rag:ingest"
    }

    fn name(&self) -> &str {
        "Ingest Document"
    }

    fn description(&self) -> &str {
        "Ingest a document into the RAG knowledge base. The document is chunked and indexed for retrieval."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        // Read config from the agent spec
        let config: RagConfig = ctx
            .agent_spec
            .config::<RagConfigKey>()
            .unwrap_or_default();

        // Build a temporary state to run ingestion against
        let mut state = RagDocumentState::default();

        let pipeline = RagPipeline;
        let doc_id = pipeline
            .run_ingest(&args.name, &args.content, &config, &mut state)
            .map_err(|e| ToolError::ExecutionFailed(format!("Ingest failed: {e}")))?;

        // Build a command that applies the ingestion to persistent state
        let mut cmd = StateCommand::new();
        cmd.patch
            .update::<RagDocumentStateKey>(RagAction::Ingest {
                name: args.name.clone(),
                content: args.content.clone(),
            });

        let result = ToolResult::success(
            "rag:ingest",
            serde_json::json!({
                "document_id": doc_id,
                "name": args.name,
                "message": "Document ingested successfully"
            }),
        );

        Ok(ToolOutput::with_command(result, cmd))
    }
}

// ---------------------------------------------------------------------------
// QueryKnowledgeTool
// ---------------------------------------------------------------------------

/// Arguments for querying the knowledge base.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryKnowledgeArgs {
    /// The search query.
    pub query: String,
    /// Maximum number of results to return (default: 5).
    #[serde(default = "default_top_k")]
    pub top_k: Option<usize>,
}

fn default_top_k() -> Option<usize> {
    Some(5)
}

/// Tool that searches the RAG knowledge base for relevant chunks.
pub struct QueryKnowledgeTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for QueryKnowledgeTool {
    type Args = QueryKnowledgeArgs;

    fn tool_id(&self) -> &str {
        "rag:query"
    }

    fn name(&self) -> &str {
        "Query Knowledge"
    }

    fn description(&self) -> &str {
        "Search the RAG knowledge base for document chunks relevant to a query."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let state: RagDocumentState = ctx
            .state::<RagDocumentStateKey>()
            .cloned()
            .unwrap_or_default();

        // Collect all chunks from all documents
        let all_chunks: Vec<_> = state
            .documents
            .iter()
            .flat_map(|doc| doc.chunks.clone())
            .collect();

        let top_k = args.top_k.unwrap_or(5);

        let retriever = KeywordRetriever;
        let results = retriever.search(&args.query, &all_chunks, top_k);

        let data = serde_json::json!({
            "results": results.iter().map(|c| serde_json::json!({
                "chunk_id": c.id,
                "document_id": c.document_id,
                "content": c.content,
                "index": c.index,
            })).collect::<Vec<_>>(),
            "count": results.len(),
            "query": args.query,
        });

        let result = ToolResult::success("rag:query", data);

        Ok(result.into())
    }
}

// ---------------------------------------------------------------------------
// ListDocumentsTool
// ---------------------------------------------------------------------------

/// Arguments for listing documents in the RAG store.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDocumentsArgs;

/// Tool that lists all documents in the RAG knowledge base.
pub struct ListDocumentsTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for ListDocumentsTool {
    type Args = ListDocumentsArgs;

    fn tool_id(&self) -> &str {
        "rag:list_documents"
    }

    fn name(&self) -> &str {
        "List Documents"
    }

    fn description(&self) -> &str {
        "List all documents currently stored in the RAG knowledge base."
    }

    async fn execute(
        &self,
        _args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let state: RagDocumentState = ctx
            .state::<RagDocumentStateKey>()
            .cloned()
            .unwrap_or_default();

        let documents: Vec<_> = state
            .documents
            .iter()
            .map(|doc| {
                serde_json::json!({
                    "id": doc.id,
                    "name": doc.name,
                    "chunk_count": doc.chunks.len(),
                    "created_at": doc.created_at,
                })
            })
            .collect();

        let data = serde_json::json!({
            "documents": documents,
            "total": state.documents.len(),
        });

        let result = ToolResult::success("rag:list_documents", data);

        Ok(result.into())
    }
}

// ---------------------------------------------------------------------------
// DeleteDocumentTool
// ---------------------------------------------------------------------------

/// Arguments for deleting a document from the RAG store.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteDocumentArgs {
    /// The unique ID of the document to delete.
    pub document_id: String,
}

/// Tool that deletes a document from the RAG knowledge base.
pub struct DeleteDocumentTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for DeleteDocumentTool {
    type Args = DeleteDocumentArgs;

    fn tool_id(&self) -> &str {
        "rag:delete_document"
    }

    fn name(&self) -> &str {
        "Delete Document"
    }

    fn description(&self) -> &str {
        "Delete a document and all its chunks from the RAG knowledge base."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        // Verify the document exists
        let state: RagDocumentState = ctx
            .state::<RagDocumentStateKey>()
            .cloned()
            .unwrap_or_default();

        let doc_exists = state
            .documents
            .iter()
            .any(|doc| doc.id == args.document_id);

        if !doc_exists {
            return Err(ToolError::NotFound(format!(
                "Document not found: {}",
                args.document_id
            )));
        }

        let mut cmd = StateCommand::new();
        cmd.patch
            .update::<RagDocumentStateKey>(RagAction::Delete {
                id: args.document_id.clone(),
            });

        let result = ToolResult::success(
            "rag:delete_document",
            serde_json::json!({
                "document_id": args.document_id,
                "message": "Document deleted successfully"
            }),
        );

        Ok(ToolOutput::with_command(result, cmd))
    }
}
