//! State management for playground replay and scoring.

use remo_runtime::state::{KeyScope, MergeStrategy, StateKey};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ReplayMessage
// ---------------------------------------------------------------------------

/// A single message in a replay session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayMessage {
    /// Role of the message sender (user, assistant, system, tool).
    pub role: String,
    /// Text content of the message.
    pub content: String,
    /// Timestamp (epoch millis) when the message was recorded.
    pub timestamp: i64,
    /// Optional tool calls associated with this message.
    pub tool_calls: Option<Vec<serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// ReplayEntry
// ---------------------------------------------------------------------------

/// A complete replay of a conversation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEntry {
    /// Unique identifier for this replay entry.
    pub id: String,
    /// The session ID this replay belongs to.
    pub session_id: String,
    /// All messages in this replay session.
    pub messages: Vec<ReplayMessage>,
    /// Timestamp (epoch millis) when the replay was created.
    pub created_at: i64,
    /// Optional tags for categorizing replays.
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// ScoreCard
// ---------------------------------------------------------------------------

/// Evaluation scores for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreCard {
    /// The session ID being scored.
    pub session_id: String,
    /// Accuracy score (0.0 – 1.0).
    pub accuracy: f64,
    /// Relevance score (0.0 – 1.0).
    pub relevance: f64,
    /// Latency in milliseconds.
    pub latency_ms: u64,
    /// Optional cost in USD.
    pub cost_usd: Option<f64>,
    /// Overall score (computed from the above).
    pub overall: f64,
}

// ---------------------------------------------------------------------------
// MetricComparison
// ---------------------------------------------------------------------------

/// A single metric comparison between baseline and candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricComparison {
    /// Name of the metric.
    pub metric: String,
    /// Baseline value.
    pub baseline_value: f64,
    /// Candidate value.
    pub candidate_value: f64,
    /// Delta (candidate - baseline).
    pub delta: f64,
}

// ---------------------------------------------------------------------------
// ComparisonResult
// ---------------------------------------------------------------------------

/// Result of comparing two score cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// The baseline score card.
    pub baseline: ScoreCard,
    /// The candidate score card.
    pub candidate: ScoreCard,
    /// Per-metric comparisons.
    pub metrics: Vec<MetricComparison>,
}

// ---------------------------------------------------------------------------
// PlaygroundState
// ---------------------------------------------------------------------------

/// Complete playground state holding replays and scorecards.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaygroundState {
    /// Recorded replay sessions.
    pub replays: Vec<ReplayEntry>,
    /// Scorecards for evaluated sessions.
    pub scorecards: Vec<ScoreCard>,
}

// ---------------------------------------------------------------------------
// PlaygroundAction
// ---------------------------------------------------------------------------

/// Actions that mutate the playground state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlaygroundAction {
    /// Record a new replay entry.
    RecordReplay { entry: ReplayEntry },
    /// Remove a replay by ID.
    RemoveReplay { id: String },
    /// Add a scorecard.
    AddScoreCard { card: ScoreCard },
    /// Clear all replays and scorecards.
    Clear,
}

// ---------------------------------------------------------------------------
// PlaygroundStateKey
// ---------------------------------------------------------------------------

/// State key for the thread-scoped playground state.
pub struct PlaygroundStateKey;

impl StateKey for PlaygroundStateKey {
    const KEY: &'static str = "playground_state";
    const MERGE: MergeStrategy = MergeStrategy::Commutative;
    const SCOPE: KeyScope = KeyScope::Thread;

    type Value = PlaygroundState;
    type Update = PlaygroundAction;

    fn apply(value: &mut Self::Value, update: Self::Update) {
        match update {
            PlaygroundAction::RecordReplay { entry } => {
                value.replays.push(entry);
            }
            PlaygroundAction::RemoveReplay { id } => {
                value.replays.retain(|r| r.id != id);
            }
            PlaygroundAction::AddScoreCard { card } => {
                value.scorecards.push(card);
            }
            PlaygroundAction::Clear => {
                value.replays.clear();
                value.scorecards.clear();
            }
        }
    }
}
