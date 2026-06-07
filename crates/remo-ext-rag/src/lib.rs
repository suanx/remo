//! RAG (Retrieval-Augmented Generation) extension for the remo agent framework.
//!
//! Provides document ingestion, chunking, keyword-based retrieval, and
//! context injection for the Remo AI Agent framework.

pub mod chunker;
pub mod config;
pub mod hooks;
pub mod pipeline;
pub mod plugin;
pub mod retriever;
pub mod state;
pub mod tools;

pub use config::{RagConfig, RagConfigKey};
pub use plugin::RagPlugin;
pub use state::{RagAction, RagChunk, RagDocument, RagDocumentState, RagDocumentStateKey};
