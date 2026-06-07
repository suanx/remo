//! Remo Playground Extension
//!
//! Provides replay recording, evaluation scoring, and comparison capabilities
//! for the Remo AI Agent framework.

pub mod config;
pub mod state;
pub mod replay;
pub mod scorecard;
pub mod plugin;

pub use plugin::PlaygroundPlugin;
pub use config::{PlaygroundConfig, PlaygroundConfigKey};
pub use state::{
    PlaygroundState, PlaygroundStateKey, PlaygroundAction,
    ReplayEntry, ReplayMessage, ScoreCard, ComparisonResult, MetricComparison,
};
pub use replay::ReplayEngine;
pub use scorecard::ScorecardEvaluator;
