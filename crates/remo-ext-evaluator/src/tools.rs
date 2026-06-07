//! Tools for LLM-as-judge evaluation of agent responses and conversations.
//!
//! Both tools use heuristic scoring as a stand-in until a real LLM judge
//! is integrated. The scoring considers response length, keyword overlap,
//! and structural indicators.

use std::collections::HashMap;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;

use remo_runtime_contract::contract::tool::{ToolCallContext, ToolError, ToolOutput, ToolResult};
use remo_runtime_contract::StateCommand;

use crate::config::EvaluationCriterion;
use crate::state::{EvaluationAction, EvaluationEntry, EvaluationStateKey};

// ---------------------------------------------------------------------------
// EvaluateResponseTool
// ---------------------------------------------------------------------------

/// Arguments for evaluating a single response.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EvaluateResponseArgs {
    /// The original query or user message.
    pub query: String,
    /// The agent response to evaluate.
    pub response: String,
    /// Optional evaluation criteria. If omitted, default criteria are used.
    #[serde(default)]
    pub criteria: Option<Vec<EvaluationCriterion>>,
}

/// Tool that evaluates a single agent response using LLM-as-judge methodology.
///
/// Currently uses heuristic scoring based on keyword overlap, response length,
/// and structural indicators. This serves as a placeholder until a real LLM
/// judge integration is available.
pub struct EvaluateResponseTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for EvaluateResponseTool {
    type Args = EvaluateResponseArgs;

    fn tool_id(&self) -> &str {
        "evaluate:evaluate_response"
    }

    fn name(&self) -> &str {
        "Evaluate Response"
    }

    fn description(&self) -> &str {
        "Evaluate an agent response using LLM-as-judge scoring with configurable criteria."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let criteria = args.criteria.clone().unwrap_or_else(default_criteria);

        let scores = score_response(&args.query, &args.response, &criteria);
        let overall = compute_overall(&scores, &criteria);

        let entry = EvaluationEntry {
            id: format!("eval_{}", now_ms()),
            agent_id: ctx.run_identity.agent_id.clone(),
            query: args.query,
            response: args.response,
            scores: scores.clone(),
            overall_score: overall,
            timestamp: now_ms(),
            criteria_used: criteria.iter().map(|c| c.name.clone()).collect(),
        };

        let mut cmd = StateCommand::new();
        cmd.patch
            .update::<EvaluationStateKey>(EvaluationAction::Record {
                entry: entry.clone(),
            });

        let data = serde_json::json!({
            "evaluation_id": entry.id,
            "overall_score": entry.overall_score,
            "scores": entry.scores,
            "criteria_used": entry.criteria_used,
            "timestamp": entry.timestamp,
            "message": "Response evaluation completed"
        });

        let result = ToolResult::success(self.tool_id(), data);
        Ok(ToolOutput::with_command(result, cmd))
    }
}

// ---------------------------------------------------------------------------
// EvaluateConversationTool
// ---------------------------------------------------------------------------

/// Arguments for evaluating a conversation.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EvaluateConversationArgs {
    /// Ordered list of messages in the conversation (user and assistant turns).
    pub messages: Vec<String>,
    /// Optional evaluation criteria. If omitted, default criteria are used.
    #[serde(default)]
    pub criteria: Option<Vec<EvaluationCriterion>>,
}

/// Tool that evaluates an entire conversation history for quality, coherence,
/// and engagement using LLM-as-judge methodology.
///
/// Currently uses heuristic scoring as a stand-in until a real LLM judge
/// integration is available.
pub struct EvaluateConversationTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for EvaluateConversationTool {
    type Args = EvaluateConversationArgs;

    fn tool_id(&self) -> &str {
        "evaluate:evaluate_conversation"
    }

    fn name(&self) -> &str {
        "Evaluate Conversation"
    }

    fn description(&self) -> &str {
        "Evaluate the quality of an entire conversation using LLM-as-judge scoring with configurable criteria."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let criteria = args.criteria.clone().unwrap_or_else(default_criteria);

        let scores = score_conversation(&args.messages, &criteria);
        let overall = compute_overall(&scores, &criteria);

        let conversation_text = args.messages.join("\n");
        let entry = EvaluationEntry {
            id: format!("eval_conv_{}", now_ms()),
            agent_id: ctx.run_identity.agent_id.clone(),
            query: args.messages.first().cloned().unwrap_or_default(),
            response: conversation_text,
            scores: scores.clone(),
            overall_score: overall,
            timestamp: now_ms(),
            criteria_used: criteria.iter().map(|c| c.name.clone()).collect(),
        };

        let mut cmd = StateCommand::new();
        cmd.patch
            .update::<EvaluationStateKey>(EvaluationAction::Record {
                entry: entry.clone(),
            });

        let data = serde_json::json!({
            "evaluation_id": entry.id,
            "overall_score": entry.overall_score,
            "scores": entry.scores,
            "criteria_used": entry.criteria_used,
            "message_count": args.messages.len(),
            "timestamp": entry.timestamp,
            "message": "Conversation evaluation completed"
        });

        let result = ToolResult::success(self.tool_id(), data);
        Ok(ToolOutput::with_command(result, cmd))
    }
}

// ---------------------------------------------------------------------------
// Heuristic scoring helpers
// ---------------------------------------------------------------------------

/// Default criteria used when none are provided.
fn default_criteria() -> Vec<EvaluationCriterion> {
    vec![
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
    ]
}

/// Score a single response against the given criteria using heuristics.
fn score_response(query: &str, response: &str, criteria: &[EvaluationCriterion]) -> HashMap<String, f64> {
    let mut scores = HashMap::new();

    for criterion in criteria {
        let score = match criterion.name.as_str() {
            "relevance" => heuristic_relevance(query, response),
            "coherence" => heuristic_coherence(response),
            "completeness" => heuristic_completeness(query, response),
            _ => heuristic_generic(response),
        };
        scores.insert(criterion.name.clone(), score.clamp(0.0, 1.0));
    }

    scores
}

/// Score an entire conversation against the given criteria using heuristics.
fn score_conversation(messages: &[String], criteria: &[EvaluationCriterion]) -> HashMap<String, f64> {
    let mut scores = HashMap::new();
    let full_text = messages.join(" ");

    for criterion in criteria {
        let score = match criterion.name.as_str() {
            "relevance" | "coherence" => {
                // For conversation, check turn-taking structure and message variety
                heuristic_conversation_coherence(messages)
            }
            "completeness" => {
                // Longer conversations with more messages tend to be more complete
                heuristic_conversation_completeness(messages)
            }
            _ => heuristic_generic(&full_text),
        };
        scores.insert(criterion.name.clone(), score.clamp(0.0, 1.0));
    }

    scores
}

/// Compute the weighted overall score from per-criterion scores.
fn compute_overall(scores: &HashMap<String, f64>, criteria: &[EvaluationCriterion]) -> f64 {
    let total_weight: f64 = criteria.iter().map(|c| c.weight).sum();
    if total_weight == 0.0 {
        return 0.0;
    }
    let weighted: f64 = criteria
        .iter()
        .map(|c| scores.get(&c.name).copied().unwrap_or(0.0) * c.weight)
        .sum();
    (weighted / total_weight).clamp(0.0, 1.0)
}

/// Heuristic: relevance based on keyword overlap between query and response.
fn heuristic_relevance(query: &str, response: &str) -> f64 {
    let query_words: Vec<&str> = query
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| w.len() > 2)
        .collect();
    if query_words.is_empty() {
        return 0.5;
    }

    let response_lower = response.to_lowercase();
    let matches = query_words
        .iter()
        .filter(|w| response_lower.contains(&w.to_lowercase()))
        .count();

    let ratio = matches as f64 / query_words.len() as f64;
    // Bonus for longer responses that show effort
    let length_bonus = (response.len() as f64 / 500.0).min(0.2);
    (ratio * 0.8 + length_bonus).min(1.0)
}

/// Heuristic: coherence based on sentence structure and length.
fn heuristic_coherence(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }

    let sentence_count = text.split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .count();

    if sentence_count == 0 {
        return 0.3;
    }

    // Prefer texts with 2-8 sentences (well-structured)
    let sentence_score = if sentence_count >= 2 && sentence_count <= 8 {
        1.0
    } else if sentence_count == 1 {
        0.5
    } else {
        0.7
    };

    // Check for proper capitalization and punctuation
    let has_capitals = text.chars().any(|c| c.is_uppercase());
    let has_punctuation = text.contains('.') || text.contains('!') || text.contains('?');
    let structure_score = match (has_capitals, has_punctuation) {
        (true, true) => 1.0,
        (true, false) => 0.6,
        (false, true) => 0.5,
        (false, false) => 0.3,
    };

    sentence_score * 0.6 + structure_score * 0.4
}

/// Heuristic: completeness based on whether the response addresses query length.
fn heuristic_completeness(query: &str, response: &str) -> f64 {
    if response.is_empty() {
        return 0.0;
    }

    // A good response should be at least 30% of the query length
    let query_len = query.len().max(1);
    let response_len = response.len();
    let length_ratio = response_len as f64 / query_len as f64;

    if length_ratio >= 3.0 {
        1.0 // Response is at least 3x query length — likely thorough
    } else if length_ratio >= 1.0 {
        0.8 // Response is at least as long as query
    } else if length_ratio >= 0.3 {
        0.5 // Response is somewhat short
    } else {
        0.3 // Response is very short relative to query
    }
}

/// Generic heuristic for unknown criteria: based on response length and variety.
fn heuristic_generic(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }

    let len = text.len();
    let unique_words: std::collections::HashSet<&str> =
        text.split_whitespace().collect();
    let diversity = if unique_words.len() < 3 {
        0.3
    } else {
        (unique_words.len() as f64 / 50.0).min(1.0)
    };

    let length_score = if len > 200 {
        1.0
    } else if len > 50 {
        0.7
    } else {
        0.4
    };

    diversity * 0.5 + length_score * 0.5
}

/// Heuristic: conversation coherence based on turn-taking and message balance.
fn heuristic_conversation_coherence(messages: &[String]) -> f64 {
    if messages.is_empty() {
        return 0.0;
    }

    let msg_count = messages.len();

    // More than 1 message suggests turn-taking
    let turn_score = if msg_count >= 4 {
        1.0
    } else if msg_count >= 2 {
        0.7
    } else {
        0.3
    };

    // Check for message length variety (not all one-word)
    let avg_len = messages.iter().map(|m| m.len()).sum::<usize>() as f64 / msg_count as f64;
    let variety_score = if avg_len > 50.0 { 1.0 } else if avg_len > 10.0 { 0.6 } else { 0.3 };

    turn_score * 0.6 + variety_score * 0.4
}

/// Heuristic: conversation completeness based on message count and total length.
fn heuristic_conversation_completeness(messages: &[String]) -> f64 {
    if messages.is_empty() {
        return 0.0;
    }

    let msg_count = messages.len();
    let total_len: usize = messages.iter().map(|m| m.len()).sum();

    let count_score = if msg_count >= 6 {
        1.0
    } else if msg_count >= 3 {
        0.7
    } else {
        0.4
    };

    let length_score = if total_len > 1000 {
        1.0
    } else if total_len > 300 {
        0.7
    } else {
        0.4
    };

    count_score * 0.5 + length_score * 0.5
}

/// Current time in milliseconds since UNIX epoch.
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
