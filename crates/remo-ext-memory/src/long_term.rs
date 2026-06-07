//! Long-term memory: persistent entries with importance-based scoring and time decay.

use crate::state::MemoryEntry;

/// Long-term memory store with keyword-based search and time decay.
#[derive(Debug, Clone)]
pub struct LongTermMemory {
    pub(crate) entries: Vec<MemoryEntry>,
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn search(&self, query: &str, top_k: usize) -> Vec<MemoryEntry> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(f64, &MemoryEntry)> = self.entries.iter().filter_map(|entry| {
            let content_lower = entry.content.to_lowercase();
            let mut score = 0.0_f64;
            for term in &query_terms {
                let matches = content_lower.matches(term).count();
                score += matches as f64;
                if entry.tags.iter().any(|t| t.to_lowercase().contains(term)) {
                    score += 2.0;
                }
            }
            score *= entry.importance;
            if score > 0.0 { Some((score, entry)) } else { None }
        }).collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).map(|(_, e)| e.clone()).collect()
    }

    pub fn entries(&self) -> &[MemoryEntry] { &self.entries }
    pub fn len(&self) -> usize { self.entries.len() }
}

impl Default for LongTermMemory {
    fn default() -> Self { Self::new() }
}
