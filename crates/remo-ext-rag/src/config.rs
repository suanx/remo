//! Configuration types for the RAG extension.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use remo_runtime_contract::PluginConfigKey;

// ---------------------------------------------------------------------------
// Chunking strategy
// ---------------------------------------------------------------------------

/// Strategy used when splitting documents into chunks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChunkingStrategy {
    /// Split on sentence boundaries (. ! ?).
    Sentence,
    /// Split on double-newline paragraph boundaries.
    Paragraph,
    /// Split paragraphs first, then recursively split long sentences.
    #[default]
    Recursive,
}

// ---------------------------------------------------------------------------
// RagConfig
// ---------------------------------------------------------------------------

/// Configuration for the RAG pipeline.
///
/// Stored in `AgentSpec.sections["rag"]` and resolved on each inference step.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct RagConfig {
    /// Maximum number of tokens (characters) per chunk.
    pub chunk_size: usize,
    /// Number of characters to overlap between consecutive chunks.
    pub chunk_overlap: usize,
    /// Number of top-ranked chunks to retrieve for each query.
    pub top_k: usize,
    /// Whether to enable a reranking step after initial retrieval.
    pub rerank_enabled: bool,
    /// The chunking strategy to use when splitting documents.
    pub chunking_strategy: ChunkingStrategy,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            chunk_overlap: 50,
            top_k: 5,
            rerank_enabled: false,
            chunking_strategy: ChunkingStrategy::Recursive,
        }
    }
}

// ---------------------------------------------------------------------------
// RagConfigKey
// ---------------------------------------------------------------------------

/// [`PluginConfigKey`] binding for RAG configuration in agent specs.
///
/// ```ignore
/// let config = spec.config::<RagConfigKey>();
/// ```
pub struct RagConfigKey;

impl PluginConfigKey for RagConfigKey {
    const KEY: &'static str = "rag";
    type Config = RagConfig;
}
