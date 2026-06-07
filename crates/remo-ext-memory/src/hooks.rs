//! Phase hook for memory-enhanced inference.
//!
//! Injects relevant memories into the system context before each LLM inference call.

use async_trait::async_trait;

use remo_runtime::agent::state::AddContextMessage;
use remo_runtime::PhaseHook;
use remo_runtime_contract::contract::context_message::ContextMessage;
use remo_runtime_contract::StateError;
use remo_runtime_contract::StateCommand;

use crate::retrieval::MemoryRetriever;
use crate::state::MemoryStateKey;

/// Phase hook that retrieves relevant memories and injects them as context.
pub struct MemoryBeforeInferenceHook;

#[async_trait]
impl PhaseHook for MemoryBeforeInferenceHook {
    async fn run(
        &self,
        ctx: &remo_runtime::PhaseContext,
    ) -> Result<StateCommand, StateError> {
        let mut cmd = StateCommand::new();

        // Read current memory state from the snapshot
        let state = match ctx.state::<MemoryStateKey>() {
            Some(s) => s.clone(),
            None => return Ok(cmd),
        };

        // Read config for top_k
        let config = ctx
            .config::<crate::config::MemoryConfigKey>()
            .unwrap_or_default();

        // Build a query from recent user messages
        let query = extract_query_from_messages(&ctx.messages);
        if query.is_empty() {
            return Ok(cmd);
        }

        // Retrieve relevant memories
        let retriever = MemoryRetriever;
        let memories = retriever.retrieve(&query, &state, config.retrieval_top_k);

        if memories.is_empty() {
            return Ok(cmd);
        }

        // Build context message from memories
        let memory_text = format!(
            "[Relevant Memories]\n{}",
            memories
                .iter()
                .map(|m| format!("- {}", m.content))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Schedule context message injection via the runtime's AddContextMessage action
        let context_msg = ContextMessage::system("memory:context", memory_text);
        cmd.schedule_action::<AddContextMessage>(context_msg)?;

        Ok(cmd)
    }
}

/// Extract a search query from the most recent user messages.
fn extract_query_from_messages(
    messages: &[std::sync::Arc<remo_runtime_contract::contract::message::Message>],
) -> String {
    let user_messages: Vec<String> = messages
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
        .collect();

    user_messages.join(" ")
}
