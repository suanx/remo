//! Replay engine for recording and comparing conversation sessions.

use crate::state::{PlaygroundState, ReplayEntry, ReplayMessage};

/// Engine for recording, listing, comparing, and managing replays.
pub struct ReplayEngine;

impl ReplayEngine {
    /// Record a new replay entry in the playground state.
    ///
    /// Returns the generated replay ID.
    pub fn record_replay(
        state: &mut PlaygroundState,
        session_id: String,
        messages: Vec<ReplayMessage>,
        tags: Vec<String>,
    ) -> String {
        let id = uuid::Uuid::now_v7().to_string();
        let created_at = now_ms();

        let entry = ReplayEntry {
            id: id.clone(),
            session_id,
            messages,
            created_at,
            tags,
        };

        state.replays.push(entry);
        id
    }

    /// List all replays in the state, optionally filtered by tag.
    pub fn list_replays<'a>(
        state: &'a PlaygroundState,
        tag_filter: Option<&str>,
    ) -> Vec<&'a ReplayEntry> {
        match tag_filter {
            Some(tag) => state
                .replays
                .iter()
                .filter(|r| r.tags.iter().any(|t| t == tag))
                .collect(),
            None => state.replays.iter().collect(),
        }
    }

    /// Get a specific replay by ID.
    pub fn get_replay<'a>(
        state: &'a PlaygroundState,
        replay_id: &str,
    ) -> Option<&'a ReplayEntry> {
        state.replays.iter().find(|r| r.id == replay_id)
    }

    /// Compare two replays using simple string similarity.
    ///
    /// Returns a similarity score between 0.0 and 1.0 based on shared
    /// content tokens.
    pub fn compare(replay_a: &ReplayEntry, replay_b: &ReplayEntry) -> f64 {
        let tokens_a = extract_tokens(replay_a);
        let tokens_b = extract_tokens(replay_b);

        if tokens_a.is_empty() && tokens_b.is_empty() {
            return 1.0;
        }
        if tokens_a.is_empty() || tokens_b.is_empty() {
            return 0.0;
        }

        // Jaccard similarity on token sets
        let set_a: std::collections::HashSet<&str> = tokens_a.iter().map(String::as_str).collect();
        let set_b: std::collections::HashSet<&str> = tokens_b.iter().map(String::as_str).collect();

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

/// Extract content tokens from all messages in a replay.
fn extract_tokens(replay: &ReplayEntry) -> Vec<String> {
    replay
        .messages
        .iter()
        .flat_map(|m| {
            m.content
                .split_whitespace()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_replay(messages: Vec<ReplayMessage>) -> ReplayEntry {
        ReplayEntry {
            id: "test-id".into(),
            session_id: "session-1".into(),
            messages,
            created_at: 0,
            tags: vec![],
        }
    }

    #[test]
    fn compare_identical_returns_one() {
        let msgs = vec![
            ReplayMessage {
                role: "user".into(),
                content: "hello world".into(),
                timestamp: 0,
                tool_calls: None,
            },
            ReplayMessage {
                role: "assistant".into(),
                content: "hi there".into(),
                timestamp: 0,
                tool_calls: None,
            },
        ];
        let a = make_replay(msgs.clone());
        let b = make_replay(msgs);
        assert_eq!(ReplayEngine::compare(&a, &b), 1.0);
    }

    #[test]
    fn compare_disjoint_returns_zero() {
        let a = make_replay(vec![ReplayMessage {
            role: "user".into(),
            content: "aaa".into(),
            timestamp: 0,
            tool_calls: None,
        }]);
        let b = make_replay(vec![ReplayMessage {
            role: "user".into(),
            content: "bbb".into(),
            timestamp: 0,
            tool_calls: None,
        }]);
        assert_eq!(ReplayEngine::compare(&a, &b), 0.0);
    }
}
