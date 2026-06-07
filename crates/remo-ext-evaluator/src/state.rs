//! State definitions for the evaluator extension.

use std::collections::HashMap;

use remo_runtime::state::{KeyScope, MergeStrategy, StateKey};
use serde::{Deserialize, Serialize};

/// A single evaluation entry recording the scores for a query-response pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationEntry {
    /// Unique identifier for this evaluation.
    pub id: String,
    /// ID of the agent that produced the response.
    pub agent_id: String,
    /// The original query / user message.
    pub query: String,
    /// The response that was evaluated.
    pub response: String,
    /// Per-criterion scores mapped by criterion name.
    pub scores: HashMap<String, f64>,
    /// Overall aggregated score (0.0 – 1.0).
    pub overall_score: f64,
    /// Timestamp in milliseconds since UNIX epoch.
    pub timestamp: i64,
    /// Names of the criteria used for this evaluation.
    pub criteria_used: Vec<String>,
}

/// Complete evaluation state held in runtime state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvaluationState {
    /// Ordered history of evaluation entries (newest last).
    pub history: Vec<EvaluationEntry>,
}

/// Actions that mutate evaluation state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvaluationAction {
    /// Record a new evaluation entry.
    Record { entry: EvaluationEntry },
    /// Clear all evaluation history.
    Clear,
}

/// State key for the evaluator system.
pub struct EvaluationStateKey;

impl StateKey for EvaluationStateKey {
    const KEY: &'static str = "evaluator_state";
    const MERGE: MergeStrategy = MergeStrategy::Commutative;
    const SCOPE: KeyScope = KeyScope::Thread;

    type Value = EvaluationState;
    type Update = EvaluationAction;

    fn apply(value: &mut Self::Value, update: Self::Update) {
        match update {
            EvaluationAction::Record { entry } => {
                value.history.push(entry);
            }
            EvaluationAction::Clear => {
                value.history.clear();
            }
        }
    }
}
