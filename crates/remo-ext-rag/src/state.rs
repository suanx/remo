//! State keys and reducers for the RAG extension.

use std::collections::HashMap;

use remo_runtime::state::{KeyScope, MergeStrategy, StateKey};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// RagChunk
// ---------------------------------------------------------------------------

/// A single chunk of a document, ready for indexing and retrieval.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RagChunk {
    /// Unique identifier for this chunk.
    pub id: String,
    /// ID of the document this chunk belongs to.
    pub document_id: String,
    /// The text content of the chunk.
    pub content: String,
    /// Position of this chunk within its parent document (0-based).
    pub index: usize,
    /// Arbitrary metadata (e.g. source file, page number).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// RagDocument
// ---------------------------------------------------------------------------

/// A fully ingested document with its generated chunks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagDocument {
    /// Unique identifier for this document.
    pub id: String,
    /// Human-readable name for the document.
    pub name: String,
    /// The original content of the document.
    pub content: String,
    /// Chunks generated from this document.
    pub chunks: Vec<RagChunk>,
    /// Unix-millisecond timestamp when the document was ingested.
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// RagDocumentState
// ---------------------------------------------------------------------------

/// Persisted RAG document store holding all ingested documents and their chunks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RagDocumentState {
    /// All ingested documents.
    pub documents: Vec<RagDocument>,
}

// ---------------------------------------------------------------------------
// RagAction
// ---------------------------------------------------------------------------

/// Actions that mutate the RAG document state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RagAction {
    /// Ingest a new document by name and content.
    Ingest { name: String, content: String },
    /// Delete a document and all its chunks by document ID.
    Delete { id: String },
    /// Clear all documents and chunks.
    Clear,
}

// ---------------------------------------------------------------------------
// RagDocumentStateKey
// ---------------------------------------------------------------------------

/// State key for the thread-scoped RAG document store.
pub struct RagDocumentStateKey;

impl StateKey for RagDocumentStateKey {
    const KEY: &'static str = "rag_documents";
    const MERGE: MergeStrategy = MergeStrategy::Commutative;
    const SCOPE: KeyScope = KeyScope::Thread;

    type Value = RagDocumentState;
    type Update = RagAction;

    fn apply(value: &mut Self::Value, update: Self::Update) {
        match update {
            RagAction::Ingest { name, content } => {
                use crate::chunker::TextChunker;
                use crate::config::RagConfig;

                let config = RagConfig::default();
                let chunker = TextChunker;
                let chunks = chunker.chunk(
                    &content,
                    config.chunk_size,
                    config.chunk_overlap,
                    &config.chunking_strategy,
                );

                let doc_id = uuid::Uuid::now_v7().to_string();
                let created_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);

                // Assign document_id to each chunk
                let chunks: Vec<RagChunk> = chunks
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut chunk)| {
                        chunk.document_id = doc_id.clone();
                        chunk.index = i;
                        chunk
                    })
                    .collect();

                let doc = RagDocument {
                    id: doc_id,
                    name,
                    content,
                    chunks,
                    created_at,
                };
                value.documents.push(doc);
            }
            RagAction::Delete { id } => {
                value.documents.retain(|d| d.id != id);
            }
            RagAction::Clear => {
                value.documents.clear();
            }
        }
    }
}
