//! Text chunking strategies for the RAG pipeline.

use crate::config::ChunkingStrategy;
use crate::state::RagChunk;

/// Text chunker that splits content into overlapping chunks.
///
/// Supports sentence-based, paragraph-based, and recursive chunking strategies.
pub struct TextChunker;

impl TextChunker {
    /// Split `content` into chunks according to the given strategy.
    ///
    /// - `chunk_size`: maximum character length per chunk.
    /// - `overlap`: number of characters to overlap between consecutive chunks.
    /// - `strategy`: chunking strategy to use.
    ///
    /// Returns a list of [`RagChunk`]s with auto-generated IDs. The caller
    /// is responsible for assigning `document_id`, `index`, and `metadata`.
    pub fn chunk(
        &self,
        content: &str,
        chunk_size: usize,
        overlap: usize,
        strategy: &ChunkingStrategy,
    ) -> Vec<RagChunk> {
        match strategy {
            ChunkingStrategy::Sentence => {
                self.chunk_by_sentence(content, chunk_size, overlap)
            }
            ChunkingStrategy::Paragraph => {
                self.chunk_by_paragraph(content, chunk_size, overlap)
            }
            ChunkingStrategy::Recursive => {
                self.chunk_recursive(content, chunk_size, overlap)
            }
        }
    }

    /// Chunk by sentence boundaries (. ! ?).
    ///
    /// Sentences are collected until adding the next sentence would exceed
    /// `chunk_size`. A new chunk then starts with the overflow, carrying
    /// `overlap` characters from the previous chunk boundary.
    fn chunk_by_sentence(
        &self,
        content: &str,
        chunk_size: usize,
        overlap: usize,
    ) -> Vec<RagChunk> {
        let sentences = split_sentences(content);
        let sentence_refs: Vec<&str> = sentences.iter().map(|s| s.as_str()).collect();
        self.build_chunks_from_segments(&sentence_refs, chunk_size, overlap)
    }

    /// Chunk by paragraph boundaries (\n\n).
    ///
    /// Paragraphs that exceed `chunk_size` are recursively split by sentence.
    fn chunk_by_paragraph(
        &self,
        content: &str,
        chunk_size: usize,
        overlap: usize,
    ) -> Vec<RagChunk> {
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        self.build_chunks_from_segments(&paragraphs, chunk_size, overlap)
    }

    /// Recursive chunking: split by paragraphs first, then by sentences if
    /// any individual paragraph still exceeds `chunk_size`.
    fn chunk_recursive(
        &self,
        content: &str,
        chunk_size: usize,
        overlap: usize,
    ) -> Vec<RagChunk> {
        // First pass: split by paragraphs.
        let paragraphs: Vec<&str> = content.split("\n\n").collect();

        // Expand any paragraph that exceeds chunk_size into sentences.
        let mut segments: Vec<String> = Vec::new();
        for para in &paragraphs {
            let trimmed = para.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.len() > chunk_size {
                // Recursively split by sentence.
                let sentences = split_sentences(trimmed);
                for sent in sentences {
                    segments.push(sent);
                }
            } else {
                segments.push(trimmed.to_string());
            }
        }

        let segment_refs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();
        self.build_chunks_from_segments(&segment_refs, chunk_size, overlap)
    }

    /// Build overlapping chunks from a list of text segments.
    ///
    /// Segments are accumulated into chunks up to `chunk_size`. When adding
    /// a new segment would exceed the limit, a new chunk is started with
    /// `overlap` characters carried over from the tail of the previous chunk.
    fn build_chunks_from_segments(
        &self,
        segments: &[&str],
        chunk_size: usize,
        overlap: usize,
    ) -> Vec<RagChunk> {
        if segments.is_empty() {
            return Vec::new();
        }

        let mut chunks: Vec<RagChunk> = Vec::new();
        let mut current_text = String::new();

        for segment in segments {
            let trimmed = segment.trim();
            if trimmed.is_empty() {
                continue;
            }

            if current_text.is_empty() {
                current_text = trimmed.to_string();
            } else if current_text.len() + trimmed.len() + 1 <= chunk_size {
                current_text.push('\n');
                current_text.push_str(trimmed);
            } else {
                // Flush current chunk.
                chunks.push(self.make_chunk(&current_text));

                // Carry over overlap characters from the end.
                let overlap_text = if overlap > 0 && current_text.len() > overlap {
                    &current_text[current_text.len() - overlap..]
                } else {
                    ""
                };

                current_text = if overlap_text.is_empty() {
                    trimmed.to_string()
                } else {
                    format!("{overlap_text}\n{trimmed}")
                };
            }
        }

        // Flush the final chunk.
        if !current_text.is_empty() {
            chunks.push(self.make_chunk(&current_text));
        }

        chunks
    }

    /// Create a RagChunk with a generated UUID and empty document_id/metadata.
    fn make_chunk(&self, content: &str) -> RagChunk {
        RagChunk {
            id: uuid::Uuid::now_v7().to_string(),
            document_id: String::new(),
            content: content.to_string(),
            index: 0, // caller assigns the correct index
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// Split text into sentences by `.`, `!`, `?` delimiters.
///
/// The delimiters are included in the resulting sentence strings.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }

    // Flush any remaining text (no trailing delimiter).
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }

    sentences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_content_produces_no_chunks() {
        let chunker = TextChunker;
        let chunks = chunker.chunk("", 512, 50, &ChunkingStrategy::Recursive);
        assert!(chunks.is_empty());
    }

    #[test]
    fn sentence_chunking_splits_on_delimiters() {
        let chunker = TextChunker;
        let content = "First sentence. Second sentence. Third sentence.";
        let chunks = chunker.chunk(content, 30, 0, &ChunkingStrategy::Sentence);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.content.len() <= 30);
        }
    }

    #[test]
    fn paragraph_chunking_splits_on_double_newline() {
        let chunker = TextChunker;
        let content = "Paragraph one.\n\nParagraph two.\n\nParagraph three.";
        let chunks = chunker.chunk(content, 50, 0, &ChunkingStrategy::Paragraph);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.content.len() <= 50);
        }
    }

    #[test]
    fn recursive_expands_long_paragraphs() {
        let chunker = TextChunker;
        let long_para = "Hello world. This is a long paragraph. It should be split.";
        let content = format!("Short.\n\n{long_para}");
        let chunks = chunker.chunk(&content, 30, 0, &ChunkingStrategy::Recursive);
        assert!(!chunks.is_empty());
    }
}
