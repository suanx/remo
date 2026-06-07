//! Memory retrieval: keyword-based matching across short-term and long-term stores.

use crate::long_term::LongTermMemory;
use crate::state::{MemoryEntry, MemoryState};

/// Retrieval engine that searches across both memory stores.
pub struct MemoryRetriever;

impl MemoryRetriever {
    /// Retrieve the most relevant memories from both stores.
    ///
    /// Short-term memories are returned first (by recency), followed by
    /// long-term memories ranked by keyword relevance and importance.
    pub fn retrieve(
        &self,
        query: &str,
        state: &MemoryState,
        top_k: usize,
    ) -> Vec<MemoryEntry> {
        // 1. Get recent short-term memories (sorted by most recent first)
        let mut short_term_entries = state.short_term.clone();
        short_term_entries.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
        let recent_stm: Vec<MemoryEntry> = short_term_entries
            .into_iter()
            .take(top_k)
            .collect();

        // 2. Get long-term memories by keyword search
        let ltm = LongTermMemory {
            entries: state.long_term.clone(),
        };
        let long_term_results = ltm.search(query, top_k);

        // 3. Merge: short-term first, then long-term (deduped by id)
        let mut results = recent_stm;
        let stm_ids: Vec<String> = results.iter().map(|e| e.id.clone()).collect();

        for entry in long_term_results {
            if !stm_ids.contains(&entry.id) {
                results.push(entry);
            }
            if results.len() >= top_k {
                break;
            }
        }

        results
    }
}

impl Default for MemoryRetriever {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::MemoryState;

    fn make_entry(id: &str, content: &str, importance: f64) -> MemoryEntry {
        MemoryEntry {
            id: id.to_string(),
            content: content.to_string(),
            timestamp: 1000,
            importance,
            tags: vec![],
        }
    }

    #[test]
    fn retrieve_merges_stm_and_ltm() {
        let state = MemoryState {
            short_term: vec![
                make_entry("stm-1", "recent conversation about rust", 0.5),
                make_entry("stm-2", "recent conversation about python", 0.5),
            ],
            long_term: vec![
                make_entry("ltm-1", "rust programming language", 0.8),
                make_entry("ltm-2", "cooking recipe", 0.3),
            ],
        };

        let retriever = MemoryRetriever;
        let results = retriever.retrieve("rust", &state, 5);

        // Should include STM entries (recency) and LTM rust entry
        assert!(results.len() >= 2);
        assert!(results.iter().any(|e| e.id == "ltm-1"));
    }

    #[test]
    fn retrieve_respects_top_k() {
        let state = MemoryState {
            short_term: (0..10)
                .map(|i| make_entry(&format!("stm-{i}"), &format!("topic {i}"), 0.5))
                .collect(),
            long_term: vec![],
        };

        let retriever = MemoryRetriever;
        let results = retriever.retrieve("topic", &state, 3);
        assert!(results.len() <= 3);
    }
}
