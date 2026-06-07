//! Configuration types for the evaluator extension.

use remo_runtime_contract::PluginConfigKey;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single evaluation criterion used in LLM-as-judge scoring.
///
/// Each criterion has a name, a description, and a relative weight that
/// influences the overall score calculation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvaluationCriterion {
    /// Short name for this criterion (e.g. "relevance", "coherence").
    pub name: String,
    /// Human-readable description of what this criterion measures.
    pub description: String,
    /// Weight factor (0.0 – 1.0) applied when computing the overall score.
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 {
    1.0
}

/// Configuration for the evaluator plugin.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvaluatorConfig {
    /// List of default evaluation criteria used when none are provided.
    #[serde(default)]
    pub criteria: Vec<EvaluationCriterion>,

    /// Maximum number of evaluation history entries to retain.
    #[serde(default = "default_max_history")]
    pub max_history: usize,

    /// If true, automatically evaluate responses on every inference.
    #[serde(default)]
    pub auto_evaluate_on_inference: bool,
}

fn default_max_history() -> usize {
    100
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            criteria: vec![
                EvaluationCriterion {
                    name: "relevance".into(),
                    description: "How relevant the response is to the query.".into(),
                    weight: 1.0,
                },
                EvaluationCriterion {
                    name: "coherence".into(),
                    description: "How coherent and well-structured the response is.".into(),
                    weight: 1.0,
                },
                EvaluationCriterion {
                    name: "completeness".into(),
                    description: "Whether the response fully addresses the query.".into(),
                    weight: 1.0,
                },
            ],
            max_history: default_max_history(),
            auto_evaluate_on_inference: false,
        }
    }
}

/// Plugin config key for the evaluator.
pub struct EvaluatorConfigKey;

impl PluginConfigKey for EvaluatorConfigKey {
    const KEY: &'static str = "evaluator";
    type Config = EvaluatorConfig;
}
