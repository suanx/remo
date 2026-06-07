//! Remo Memory System Extension
//!
//! Provides short-term and long-term memory for the Remo AI Agent framework.
//! Memories are managed through a state-key based architecture with automatic
//! consolidation from short-term to long-term storage.

pub mod config;
pub mod long_term;
pub mod retrieval;
pub mod state;
pub mod hooks;
pub mod tools;
pub mod plugin;
pub use config::MemoryConfigKey;
pub use state::{MemoryEntry, MemoryState, MemoryStateKey};
pub use plugin::MemoryPlugin;
