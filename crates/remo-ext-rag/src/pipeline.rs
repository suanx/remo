//! RAG pipeline orchestrating ingestion and retrieval.

use crate::chunker::TextChunker;
use crate::config::RagConfig;
use crate::retriever::KeywordRetriever;
use crate::state::{RagChunk, RagDocumentState};

/// The core RAG pipeline that coordinates chunking, storage, and retrieval.
pub struct RagPipeline;

impl RagPipeline {
    /// Ingest a document: chunk it according to `config` and store it
    /// in `state`.
    ///
    /// Returns the generated document ID on success.
    pub fn run_ingest(
        &self,
        name: &str,
        content: &str,
        config: &RagConfig,
        state: &mut RagDocumentState,
    ) -> Result<String, String> {
        if name.trim().is_empty() {
            return Err("Document name must not be empty".to_string());
        }
        if content.trim().is_empty() {
            return Err("Document content must not be empty".to_string());
        }

        let doc_id = uuid::Uuid::now_v7().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let chunker = TextChunker;
        let chunks: Vec<RagChunk> = chunker
            .chunk(
                content,
                config.chunk_size,
                config.chunk_overlap,
                &config.chunking_strategy,
            )
            .into_iter()
            .enumerate()
            .map(|(i, mut chunk)| {
                chunk.document_id = doc_id.clone();
                chunk.index = i;
                chunk
            })
            .collect();

        let doc = crate::state::RagDocument {
            id: doc_id.clone(),
            name: name.to_string(),
            content: content.to_string(),
            chunks,
            created_at,
        };

        state.documents.push(doc);
        Ok(doc_id)
    }

    /// Retrieve the most relevant chunks for `query` from the current state.
    ///
    /// Uses keyword-based TF-IDF scoring limited to `config.top_k` results.
    pub fn run_retrieve(
        &self,
        query: &str,
        config: &RagConfig,
        state: &RagDocumentState,
    ) -> Vec<RagChunk> {
        // Flatten all chunks across every ingested document.
        let all_chunks: Vec<RagChunk> = state
            .documents
            .iter()
            .flat_map(|doc| doc.chunks.clone())
            .collect();

        let retriever = KeywordRetriever;
        retriever.search(query, &all_chunks, config.top_k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RagConfig;

    #[test]
    fn ingest_and_retrieve() {
        let pipeline = RagPipeline;
        let config = RagConfig::default();
        let mut state = RagDocumentState::default();

        let doc_id = pipeline
            .run_ingest(
                "Test Doc",
                "Rust is a systems programming language. It provides memory safety without garbage collection.",
                &config,
                &mut state,
            )
            .unwrap();

        assert!(!doc_id.is_empty());
        assert_eq!(state.documents.len(), 1);
        assert_eq!(state.documents[0].name, "Test Doc");
        assert!(!state.documents[0].chunks.is_empty());

        let results = pipeline.run_retrieve("rust programming", &config, &state);
        assert!(!results.is_empty());
    }

    #[test]
    fn ingest_empty_name_fails() {
        let pipeline = RagPipeline;
        let config = RagConfig::default();
        let mut state = RagDocumentState::default();

        let result = pipeline.run_ingest("", "Some content", &config, &mut state);
        assert!(result.is_err());
    }

    #[test]
    fn ingest_empty_content_fails() {
        let pipeline = RagPipeline;
        let config = RagConfig::default();
        let mut state = RagDocumentState::default();

        let result = pipeline.run_ingest("Test", "", &config, &mut state);
        assert!(result.is_err());
    }
}
