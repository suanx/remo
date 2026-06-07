//! Configuration types for the playground extension.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use remo_runtime_contract::PluginConfigKey;

// ---------------------------------------------------------------------------
// PlaygroundConfig
// ---------------------------------------------------------------------------

/// Configuration for the playground extension.
///
/// Stored in `AgentSpec.sections["playground"]` and resolved on each use.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PlaygroundConfig {
    /// Whether replay recording is enabled.
    pub replay_enabled: bool,
    /// Maximum number of replay entries to retain (FIFO eviction).
    pub trace_retention: usize,
    /// Whether scoring/evaluation is enabled.
    pub scoring_enabled: bool,
}

impl Default for PlaygroundConfig {
    fn default() -> Self {
        Self {
            replay_enabled: true,
            trace_retention: 50,
            scoring_enabled: false,
        }
    }
}

// ---------------------------------------------------------------------------
// PlaygroundConfigKey
// ---------------------------------------------------------------------------

/// [`PluginConfigKey`] binding for playground config in agent specs.
pub struct PlaygroundConfigKey;

impl PluginConfigKey for PlaygroundConfigKey {
    const KEY: &'static str = "playground";
    type Config = PlaygroundConfig;
}
