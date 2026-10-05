//! Offline information-retrieval metrics for admission tests and benchmarks.
//!
//! These helpers measure ranking quality only. They never participate in runtime
//! retrieval, mutate project memory, or grant canon/human-review authority.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RetrievalMetrics {
    pub precision_at_k: f64,
    pub recall_at_k: f64,
    pub reciprocal_rank: f64,
    pub ndcg_at_k: f64,
}

pub fn precision_at_k(retrieved: &[String], relevant: &HashSet<String>, k: usize) -> f64 {
    if k == 0 {
        return 0.0;
    }
    let hits = retrieved
        .iter()
        .take(k)
        .filter(|id| relevant.contains(id.as_str()))
        .count();
    hits as f64 / k as f64
}

pub fn recall_at_k(retrieved: &[String], relevant: &HashSet<String>, k: usize) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    let hits = retrieved
        .iter()
        .take(k)
        .filter(|id| relevant.contains(id.as_str()))
        .collect::<HashSet<_>>()
        .len();
    hits as f64 / relevant.len() as f64
}

pub fn reciprocal_rank(retrieved: &[String], relevant: &HashSet<String>) -> f64 {
    retrieved
        .iter()
        .position(|id| relevant.contains(id))
        .map(|index| 1.0 / (index + 1) as f64)
        .unwrap_or(0.0)
}

pub fn dcg_at_k(retrieved: &[String], graded_relevance: &HashMap<String, u32>, k: usize) -> f64 {
    retrieved
        .iter()
        .take(k)
        .enumerate()
        .map(|(index, id)| {
            let relevance = f64::from(*graded_relevance.get(id).unwrap_or(&0));
            if relevance <= 0.0 {
                return 0.0;
            }
            let gain = 2_f64.powf(relevance) - 1.0;
            gain / ((index + 2) as f64).log2()
        })
        .sum()
}

pub fn ndcg_at_k(retrieved: &[String], graded_relevance: &HashMap<String, u32>, k: usize) -> f64 {
    if k == 0 || graded_relevance.is_empty() {
        return 0.0;
    }
    let actual = dcg_at_k(retrieved, graded_relevance, k);
    let mut ideal = graded_relevance
        .iter()
        .map(|(id, relevance)| (id.clone(), *relevance))
        .collect::<Vec<_>>();
    ideal.sort_by(|(left_id, left_rel), (right_id, right_rel)| {
        right_rel.cmp(left_rel).then_with(|| left_id.cmp(right_id))
    });
    let ideal_ids = ideal.into_iter().map(|(id, _)| id).collect::<Vec<_>>();
    let ideal_score = dcg_at_k(&ideal_ids, graded_relevance, k);
    if ideal_score <= f64::EPSILON {
        0.0
    } else {
        actual / ideal_score
    }
}

pub fn evaluate_ranking(
    retrieved: &[String],
    relevant: &HashSet<String>,
    graded_relevance: &HashMap<String, u32>,
    k: usize,
) -> RetrievalMetrics {
    RetrievalMetrics {
        precision_at_k: precision_at_k(retrieved, relevant, k),
        recall_at_k: recall_at_k(retrieved, relevant, k),
        reciprocal_rank: reciprocal_rank(retrieved, relevant),
        ndcg_at_k: ndcg_at_k(retrieved, graded_relevance, k),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn ranking_metrics_reward_earlier_relevant_results() {
        let relevant = ids(&["d1", "d3"]).into_iter().collect::<HashSet<_>>();
        let graded = HashMap::from([("d1".to_string(), 3), ("d3".to_string(), 2)]);

        let good = evaluate_ranking(&ids(&["d1", "d2", "d3"]), &relevant, &graded, 3);
        let degraded = evaluate_ranking(&ids(&["d2", "d3", "d1"]), &relevant, &graded, 3);

        assert!((good.precision_at_k - (2.0 / 3.0)).abs() < 1e-9);
        assert!((good.recall_at_k - 1.0).abs() < 1e-9);
        assert!((good.reciprocal_rank - 1.0).abs() < 1e-9);
        assert!(good.ndcg_at_k > degraded.ndcg_at_k);
        assert!(good.reciprocal_rank > degraded.reciprocal_rank);
    }

    #[test]
    fn empty_inputs_are_fail_closed_zero_scores() {
        let relevant = HashSet::new();
        let graded = HashMap::new();
        let metrics = evaluate_ranking(&[], &relevant, &graded, 5);
        assert_eq!(metrics.precision_at_k, 0.0);
        assert_eq!(metrics.recall_at_k, 0.0);
        assert_eq!(metrics.reciprocal_rank, 0.0);
        assert_eq!(metrics.ndcg_at_k, 0.0);
    }
}
