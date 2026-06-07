//! Phase hooks for RAG-enhanced inference.
//!
//! Automatically retrieves relevant document chunks and injects them as context.

use async_trait::async_trait;

use remo_runtime::agent::state::AddContextMessage;
use remo_runtime::PhaseHook;
use remo_runtime_contract::contract::context_message::ContextMessage;
use remo_runtime_contract::StateError;
use remo_runtime_contract::StateCommand;

use crate::config::{RagConfig, RagConfigKey};
use crate::retriever::KeywordRetriever;
use crate::state::{RagDocumentState, RagDocumentStateKey};

/// Phase hook that retrieves relevant document chunks and injects them as context.
pub struct RagBeforeInferenceHook;

#[async_trait]
impl PhaseHook for RagBeforeInferenceHook {
    async fn run(
        &self,
        ctx: &remo_runtime::PhaseContext,
    ) -> Result<StateCommand, StateError> {
        let mut cmd = StateCommand::new();

        // Read RAG state
        let state: RagDocumentState = match ctx.state::<RagDocumentStateKey>() {
            Some(s) => s.clone(),
            None => return Ok(cmd),
        };

        if state.documents.is_empty() {
            return Ok(cmd);
        }

        // Read config
        let config: RagConfig = ctx.config::<RagConfigKey>().unwrap_or_default();

        // Build query from recent user messages
        let query = extract_query_from_messages(&ctx.messages);
        if query.is_empty() {
            return Ok(cmd);
        }

        // Collect all chunks from all documents
        let all_chunks: Vec<_> = state
            .documents
            .iter()
            .flat_map(|doc| doc.chunks.clone())
            .collect();

        if all_chunks.is_empty() {
            return Ok(cmd);
        }

        // Retrieve relevant chunks
        let retriever = KeywordRetriever;
        let results = retriever.search(&query, &all_chunks, config.top_k);

        if results.is_empty() {
            return Ok(cmd);
        }

        // Build context from retrieved chunks
        let context_text = format!(
            "[Knowledge Base]\n{}",
            results
                .iter()
                .map(|c| format!("[{}]: {}", c.document_id, c.content))
                .collect::<Vec<_>>()
                .join("\n\n")
        );

        let context_msg = ContextMessage::system("rag:knowledge", context_text);
        cmd.schedule_action::<AddContextMessage>(context_msg)?;

        Ok(cmd)
    }
}

/// Extract a search query from the most recent user messages.
fn extract_query_from_messages(
    messages: &[std::sync::Arc<remo_runtime_contract::contract::message::Message>],
) -> String {
    messages
        .iter()
        .rev()
        .take(3)
        .filter_map(|m| {
            if m.role == remo_runtime_contract::contract::message::Role::User {
                Some(m.text())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
