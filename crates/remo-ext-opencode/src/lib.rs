//! OpenCode CLI integration and free model provider for the Remo AI Agent framework.
//!
//! Provides auto-discovery of free models from OpenCode Zen API and
//! tools for interacting with OpenCode CLI for code generation tasks.

pub mod config;
pub mod model_discovery;
pub mod plugin;
pub mod tools;

pub use config::{OpenCodeConfig, OpenCodeConfigKey, OpenCodeConfigKey as ConfigKey, DiscoveredModel, builtin_free_models};
pub use plugin::{OpenCodePlugin, OPENCODE_EXEC_TOOL_ID, OPENCODE_LIST_MODELS_TOOL_ID, OPENCODE_CHECK_CLI_TOOL_ID};
pub use tools::{OpenCodeExecTool, OpenCodeListModelsTool, OpenCodeCheckCliTool};
