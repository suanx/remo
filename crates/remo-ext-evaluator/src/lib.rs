//! LLM-as-judge online evaluation extension for the Remo AI Agent framework.
//!
//! Provides tools for evaluating agent responses and conversations using
//! LLM-as-judge methodology with configurable criteria.

pub mod config;
pub mod plugin;
pub mod state;
pub mod tools;

pub use config::{EvaluationCriterion, EvaluatorConfig, EvaluatorConfigKey};
pub use plugin::EvaluatorPlugin;
pub use state::{EvaluationEntry, EvaluationState, EvaluationStateKey};
pub use tools::{EvaluateConversationTool, EvaluateResponseTool};
