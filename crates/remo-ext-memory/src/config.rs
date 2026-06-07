//! Configuration for the memory system extension.

use remo_runtime_contract::PluginConfigKey;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Strategy for retrieving memories from long-term storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStrategy {
    /// Return the most recently stored memories.
    Recent,
    /// Return memories ranked by importance score.
    Importance,
    /// Combine recency and importance for hybrid scoring.
    Hybrid,
}

impl Default for MemoryStrategy {
    fn default() -> Self {
        Self::Hybrid
    }
}

/// Configuration for the memory system plugin.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryConfig {
    /// Maximum number of entries in short-term memory before consolidation.
    #[serde(default = "default_retention_size")]
    pub retention_size: usize,

    /// Rate at which long-term memories decay over time (0.0 – 1.0).
    #[serde(default = "default_decay_rate")]
    pub decay_rate: f64,

    /// Default number of top memories to return on retrieval.
    #[serde(default = "default_retrieval_top_k")]
    pub retrieval_top_k: usize,

    /// Retrieval scoring strategy.
    #[serde(default)]
    pub strategy: MemoryStrategy,
}

fn default_retention_size() -> usize {
    100
}

fn default_decay_rate() -> f64 {
    0.1
}

fn default_retrieval_top_k() -> usize {
    5
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            retention_size: default_retention_size(),
            decay_rate: default_decay_rate(),
            retrieval_top_k: default_retrieval_top_k(),
            strategy: MemoryStrategy::default(),
        }
    }
}

/// Plugin config key for the memory system.
pub struct MemoryConfigKey;

impl PluginConfigKey for MemoryConfigKey {
    const KEY: &'static str = "memory";
    type Config = MemoryConfig;
}
