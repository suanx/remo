//! Keyword-based retriever using TF-IDF scoring.

use std::collections::HashMap;

use crate::state::RagChunk;

/// Keyword-based retriever that scores chunks against a query using
/// TF-IDF (Term Frequency × Inverse Document Frequency) weighting.
pub struct KeywordRetriever;

impl KeywordRetriever {
    /// Search `chunks` for the most relevant results to `query`.
    ///
    /// Returns up to `top_k` chunks sorted by descending relevance score.
    /// If `chunks` is empty or `query` is blank, returns an empty Vec.
    pub fn search(&self, query: &str, chunks: &[RagChunk], top_k: usize) -> Vec<RagChunk> {
        if query.trim().is_empty() || chunks.is_empty() || top_k == 0 {
            return Vec::new();
        }

        let query_tokens = tokenize(query);
        if query_tokens.is_empty() {
            return Vec::new();
        }

        // Compute IDF for each unique token across all chunks.
        let total_docs = chunks.len() as f64;
        let mut doc_freq: HashMap<String, usize> = HashMap::new();

        for chunk in chunks {
            let unique_terms: std::collections::HashSet<String> =
                tokenize(&chunk.content).into_iter().collect();
            for term in unique_terms {
                *doc_freq.entry(term).or_insert(0) += 1;
            }
        }

        // Score each chunk.
        let mut scored: Vec<(usize, f64)> = chunks
            .iter()
            .enumerate()
            .map(|(idx, chunk)| {
                let tf = compute_tf(&query_tokens, &chunk.content);
                let score = query_tokens
                    .iter()
                    .map(|token| {
                        let tf_val = tf.get(token).copied().unwrap_or(0.0);
                        let df = doc_freq.get(token).copied().unwrap_or(0) as f64;
                        let idf = if df > 0.0 {
                            (total_docs / df).ln()
                        } else {
                            0.0
                        };
                        tf_val * idf
                    })
                    .sum::<f64>();
                (idx, score)
            })
            .collect();

        // Sort by descending score, take top_k.
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        scored
            .into_iter()
            .filter(|(_, score)| *score > 0.0)
            .map(|(idx, _)| chunks[idx].clone())
            .collect()
    }
}

/// Tokenize text into lowercase alphanumeric tokens.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Compute term frequency for each query token in the document content.
///
/// TF is normalised by the total number of tokens in the document.
fn compute_tf(query_tokens: &[String], content: &str) -> HashMap<String, f64> {
    let doc_tokens = tokenize(content);
    let doc_len = doc_tokens.len() as f64;

    if doc_len == 0.0 {
        return HashMap::new();
    }

    let mut term_counts: HashMap<String, usize> = HashMap::new();
    for token in &doc_tokens {
        *term_counts.entry(token.clone()).or_insert(0) += 1;
    }

    let mut tf = HashMap::new();
    for qt in query_tokens {
        let count = term_counts.get(qt).copied().unwrap_or(0) as f64;
        tf.insert(qt.clone(), count / doc_len);
    }
    tf
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chunk(id: &str, content: &str) -> RagChunk {
        RagChunk {
            id: id.to_string(),
            document_id: "doc-1".to_string(),
            content: content.to_string(),
            index: 0,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn empty_query_returns_nothing() {
        let retriever = KeywordRetriever;
        let chunks = vec![make_chunk("c1", "hello world")];
        let results = retriever.search("", &chunks, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn returns_ranked_results() {
        let retriever = KeywordRetriever;
        let chunks = vec![
            make_chunk("c1", "the cat sat on the mat"),
            make_chunk("c2", "the dog ran in the park"),
            make_chunk("c3", "the cat played with the dog"),
        ];
        let results = retriever.search("cat", &chunks, 2);
        assert!(results.len() <= 2);
        // Chunks c1 and c3 mention "cat"; they should appear first.
        let ids: Vec<&str> = results.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"c1") || ids.contains(&"c3"));
    }

    #[test]
    fn top_k_limits_results() {
        let retriever = KeywordRetriever;
        let chunks = vec![
            make_chunk("c1", "rust programming language"),
            make_chunk("c2", "rust ownership model"),
            make_chunk("c3", "rust borrow checker"),
        ];
        let results = retriever.search("rust", &chunks, 2);
        assert!(results.len() <= 2);
    }
}
