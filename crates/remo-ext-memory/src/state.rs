//! State definitions for the memory system.

use remo_runtime::state::{KeyScope, MergeStrategy, StateKey};
use serde::{Deserialize, Serialize};

/// A single memory entry stored in either short-term or long-term memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier for this memory entry.
    pub id: String,
    /// The textual content of the memory.
    pub content: String,
    /// Timestamp in milliseconds since UNIX epoch.
    pub timestamp: i64,
    /// Importance score (0.0 – 1.0).
    pub importance: f64,
    /// Tags for categorization and keyword-based retrieval.
    pub tags: Vec<String>,
}

/// Complete memory state held in runtime state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryState {
    /// Recent / working memories (sliding window).
    pub short_term: Vec<MemoryEntry>,
    /// Consolidated long-term memories.
    pub long_term: Vec<MemoryEntry>,
}

/// Actions that mutate memory state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryAction {
    /// Store a new entry into short-term memory.
    Store { entry: MemoryEntry },
    /// Remove a memory entry by its id (searches both stores).
    Remove { id: String },
    /// Trigger consolidation: move oldest short-term entries beyond the
    /// threshold into long-term storage.
    Consolidate { threshold: usize },
    /// Apply time-decay to long-term memory importance scores.
    Decay { rate: f64 },
}

/// State key for the memory system.
pub struct MemoryStateKey;

impl StateKey for MemoryStateKey {
    const KEY: &'static str = "memory_state";
    const MERGE: MergeStrategy = MergeStrategy::Commutative;
    const SCOPE: KeyScope = KeyScope::Thread;

    type Value = MemoryState;
    type Update = MemoryAction;

    fn apply(value: &mut Self::Value, update: Self::Update) {
        match update {
            MemoryAction::Store { entry } => {
                value.short_term.push(entry);
            }
            MemoryAction::Remove { id } => {
                value.short_term.retain(|e| e.id != id);
                value.long_term.retain(|e| e.id != id);
            }
            MemoryAction::Consolidate { threshold } => {
                value.short_term.sort_by_key(|e| e.timestamp);
                let overflow = value.short_term.len().saturating_sub(threshold);
                if overflow > 0 {
                    let drained: Vec<MemoryEntry> =
                        value.short_term.drain(..overflow).collect();
                    value.long_term.extend(drained);
                }
            }
            MemoryAction::Decay { rate } => {
                let now = now_ms_i64();
                for entry in &mut value.long_term {
                    let age_hours =
                        ((now - entry.timestamp) as f64 / 3_600_000.0).max(0.0);
                    let decay_factor = (-rate * age_hours).exp();
                    entry.importance *= decay_factor;
                }
                // Remove entries whose importance has decayed to negligible levels.
                value.long_term.retain(|e| e.importance > 0.01);
            }
        }
    }
}

/// Current time in milliseconds since UNIX epoch (i64 variant for timestamps).
fn now_ms_i64() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
