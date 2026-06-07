//! RAG plugin: registers state keys, hooks, tools, and config for the RAG system.

use std::sync::Arc;

use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::{KeyScope, MutationBatch, StateKeyOptions};
use remo_runtime_contract::StateError;
use remo_runtime_contract::model::Phase;
use remo_runtime_contract::registry_spec::AgentSpec;

use crate::config::RagConfigKey;
use crate::hooks::RagBeforeInferenceHook;
use crate::state::RagDocumentStateKey;
use crate::tools::{DeleteDocumentTool, IngestDocumentTool, ListDocumentsTool, QueryKnowledgeTool};

/// Stable plugin name for the RAG extension.
pub const RAG_PLUGIN_NAME: &str = "rag";

/// RAG extension plugin.
///
/// Registers:
/// - [`RagDocumentStateKey`]: thread-scoped document store
/// - A `BeforeInference` phase hook that retrieves relevant document chunks
/// - Four tools: ingest, query, list documents, and delete document
pub struct RagPlugin;

impl Plugin for RagPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: RAG_PLUGIN_NAME,
        }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        registrar.register_key::<RagDocumentStateKey>(StateKeyOptions {
            persistent: true,
            retain_on_uninstall: false,
            scope: KeyScope::Thread,
        })?;

        registrar.register_phase_hook(
            RAG_PLUGIN_NAME,
            Phase::BeforeInference,
            RagBeforeInferenceHook,
        )?;

        registrar.register_tool("rag:ingest", Arc::new(IngestDocumentTool))?;
        registrar.register_tool("rag:query", Arc::new(QueryKnowledgeTool))?;
        registrar.register_tool("rag:list_documents", Arc::new(ListDocumentsTool))?;
        registrar.register_tool("rag:delete_document", Arc::new(DeleteDocumentTool))?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<RagConfigKey>()
                .with_display_name("RAG")
                .with_description("Document ingestion, chunking, and retrieval-augmented generation.")
                .with_category("cognition")
                .with_editor("rag"),
        ]
    }

    fn on_activate(
        &self,
        _agent_spec: &AgentSpec,
        _patch: &mut MutationBatch,
    ) -> Result<(), StateError> {
        // RAG starts with empty documents; no initial seeding needed.
        Ok(())
    }
}
