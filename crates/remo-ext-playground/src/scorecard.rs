//! Scorecard evaluation and comparison for playground sessions.

use crate::state::{ComparisonResult, MetricComparison, PlaygroundState, ScoreCard};

/// Evaluator for creating and comparing score cards.
pub struct ScorecardEvaluator;

impl ScorecardEvaluator {
    /// Evaluate a session and produce a score card.
    ///
    /// Computes an overall score as the weighted average of accuracy (40%),
    /// relevance (40%), and an inverse-latency component (20%).
    pub fn evaluate(
        session_id: String,
        accuracy: f64,
        relevance: f64,
        latency_ms: u64,
        cost_usd: Option<f64>,
    ) -> ScoreCard {
        // Inverse latency factor: 1.0 at 0ms, approaching 0.0 for large latencies.
        let latency_factor = 1.0 / (1.0 + (latency_ms as f64) / 1000.0);
        let overall = accuracy * 0.4 + relevance * 0.4 + latency_factor * 0.2;

        ScoreCard {
            session_id,
            accuracy: accuracy.clamp(0.0, 1.0),
            relevance: relevance.clamp(0.0, 1.0),
            latency_ms,
            cost_usd,
            overall,
        }
    }

    /// Compare two score cards and produce a detailed comparison result.
    pub fn compare(baseline: &ScoreCard, candidate: &ScoreCard) -> ComparisonResult {
        let metrics = vec![
            MetricComparison {
                metric: "accuracy".into(),
                baseline_value: baseline.accuracy,
                candidate_value: candidate.accuracy,
                delta: candidate.accuracy - baseline.accuracy,
            },
            MetricComparison {
                metric: "relevance".into(),
                baseline_value: baseline.relevance,
                candidate_value: candidate.relevance,
                delta: candidate.relevance - baseline.relevance,
            },
            MetricComparison {
                metric: "latency_ms".into(),
                baseline_value: baseline.latency_ms as f64,
                candidate_value: candidate.latency_ms as f64,
                delta: candidate.latency_ms as f64 - baseline.latency_ms as f64,
            },
            MetricComparison {
                metric: "overall".into(),
                baseline_value: baseline.overall,
                candidate_value: candidate.overall,
                delta: candidate.overall - baseline.overall,
            },
        ];

        ComparisonResult {
            baseline: baseline.clone(),
            candidate: candidate.clone(),
            metrics,
        }
    }

    /// Find scorecards for a given session in the playground state.
    pub fn scorecards_for_session<'a>(
        state: &'a PlaygroundState,
        session_id: &str,
    ) -> Vec<&'a ScoreCard> {
        state
            .scorecards
            .iter()
            .filter(|sc| sc.session_id == session_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_computes_overall() {
        let card = ScorecardEvaluator::evaluate(
            "s1".into(),
            0.9,
            0.8,
            200,
            None,
        );
        assert!(card.overall > 0.0 && card.overall <= 1.0);
        assert_eq!(card.session_id, "s1");
        assert_eq!(card.cost_usd, None);
    }

    #[test]
    fn evaluate_clamps_scores() {
        let card = ScorecardEvaluator::evaluate(
            "s1".into(),
            1.5,
            -0.2,
            100,
            Some(0.05),
        );
        assert_eq!(card.accuracy, 1.0);
        assert_eq!(card.relevance, 0.0);
        assert_eq!(card.cost_usd, Some(0.05));
    }

    #[test]
    fn compare_produces_metrics() {
        let a = ScoreCard {
            session_id: "s1".into(),
            accuracy: 0.8,
            relevance: 0.7,
            latency_ms: 100,
            cost_usd: None,
            overall: 0.75,
        };
        let b = ScoreCard {
            session_id: "s1".into(),
            accuracy: 0.9,
            relevance: 0.8,
            latency_ms: 80,
            cost_usd: None,
            overall: 0.84,
        };

        let result = ScorecardEvaluator::compare(&a, &b);
        assert_eq!(result.metrics.len(), 4);
        assert!(result.metrics[0].delta > 0.0); // accuracy improved
    }
}
