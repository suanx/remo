//! Topological-sort workflow executor with parallel layer execution.

use std::collections::{HashMap, HashSet, VecDeque};

use futures::future::BoxFuture;
use tokio::sync::Semaphore;

use crate::dsl::{NodeSpec, WorkflowSpec};
use crate::state::{NodeResult, WorkflowState, WorkflowStatus};
use std::sync::Arc;

/// DAG-based workflow executor.
///
/// Performs topological sort of nodes, then executes layer by layer,
/// running nodes within each layer in parallel (up to `max_parallel`).
pub struct WorkflowExecutor {
    /// Maximum number of nodes to execute concurrently.
    max_parallel: usize,
}

impl WorkflowExecutor {
    /// Create a new executor with the given concurrency limit.
    pub fn new(max_parallel: usize) -> Self {
        Self { max_parallel }
    }

    /// Execute a workflow specification, updating state as nodes complete.
    ///
    /// `executor_fn` is called for each node to produce its output. The function
    /// receives a [`NodeSpec`] and returns a future that resolves to the
    /// node's output value or an error string.
    pub async fn execute(
        &self,
        spec: &WorkflowSpec,
        state: &mut WorkflowState,
        executor_fn: impl Fn(NodeSpec) -> BoxFuture<'static, Result<serde_json::Value, String>>
            + Send
            + Sync
            + 'static,
    ) -> Result<(), String> {
        // Wrap executor_fn so it can be shared across tokio tasks.
        let executor_fn = Arc::new(executor_fn);

        // Build adjacency and in-degree maps from edges + depends_on.
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut children: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize all nodes with in-degree 0.
        for node in &spec.nodes {
            in_degree.entry(node.id.clone()).or_insert(0);
            children.entry(node.id.clone()).or_insert_with(Vec::new);
        }

        // Process explicit edges.
        for edge in &spec.edges {
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
            children
                .entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());
        }

        // Also honour depends_on declarations (supplement edges if not already present).
        let edges_set: HashSet<(&str, &str)> = spec
            .edges
            .iter()
            .map(|e| (e.from.as_str(), e.to.as_str()))
            .collect();

        for node in &spec.nodes {
            for dep in &node.depends_on {
                if !edges_set.contains(&(dep.as_str(), node.id.as_str())) {
                    *in_degree.entry(node.id.clone()).or_insert(0) += 1;
                    children
                        .entry(dep.clone())
                        .or_insert_with(Vec::new)
                        .push(node.id.clone());
                }
            }
        }

        // Collect nodes by ID for quick lookup.
        let node_map: HashMap<String, NodeSpec> = spec
            .nodes
            .iter()
            .map(|n| (n.id.clone(), n.clone()))
            .collect();

        // Seed the ready queue with nodes that have zero in-degree.
        let mut ready: VecDeque<String> = VecDeque::new();
        for (id, &deg) in &in_degree {
            if deg == 0 {
                ready.push_back(id.clone());
            }
        }

        let semaphore = Arc::new(Semaphore::new(self.max_parallel));
        let mut completed_count: HashSet<String> = HashSet::new();

        // Process layers until all nodes are done.
        while !ready.is_empty() {
            // Drain the current layer from the ready queue.
            let current_layer: Vec<String> = ready.drain(..).collect();

            // Mark all nodes in this layer as running.
            for id in &current_layer {
                state.node_results.insert(
                    id.clone(),
                    NodeResult {
                        node_id: id.clone(),
                        status: WorkflowStatus::Running,
                        output: None,
                    },
                );
            }

            // Execute all nodes in this layer concurrently, respecting the semaphore.
            let mut handles = Vec::new();

            for id in &current_layer {
                let node = node_map
                    .get(id)
                    .cloned()
                    .ok_or_else(|| format!("Node {id} not found in spec"))?;
                let permit = semaphore
                    .clone()
                    .acquire_owned()
                    .await
                    .map_err(|e| format!("Semaphore closed: {e}"))?;

                let ef = Arc::clone(&executor_fn);
                let node_id = id.clone();

                let handle = tokio::spawn(async move {
                    let _permit = permit; // Hold permit for duration of execution.
                    let result = (ef)(node).await;
                    (node_id, result)
                });

                handles.push(handle);
            }

            // Collect results and update state.
            for handle in handles {
                let (node_id, result) = handle
                    .await
                    .map_err(|e| format!("Task join error: {e}"))?;

                match result {
                    Ok(output) => {
                        state.node_results.insert(
                            node_id.clone(),
                            NodeResult {
                                node_id: node_id.clone(),
                                status: WorkflowStatus::Completed,
                                output: Some(output),
                            },
                        );
                        completed_count.insert(node_id.clone());

                        // Unlock children.
                        if let Some(child_ids) = children.get(&node_id) {
                            for child_id in child_ids {
                                if let Some(deg) = in_degree.get_mut(child_id) {
                                    *deg -= 1;
                                    if *deg == 0 && !completed_count.contains(child_id.as_str()) {
                                        ready.push_back(child_id.clone());
                                    }
                                }
                            }
                        }
                    }
                    Err(error) => {
                        state.node_results.insert(
                            node_id.clone(),
                            NodeResult {
                                node_id: node_id.clone(),
                                status: WorkflowStatus::Failed {
                                    error: error.clone(),
                                },
                                output: None,
                            },
                        );
                        completed_count.insert(node_id.clone());

                        // Mark downstream nodes as cancelled (fail-fast).
                        cancel_downstream(
                            &node_id,
                            &children,
                            state,
                            &mut completed_count,
                            &mut ready,
                        );
                    }
                }
            }
        }

        // Determine overall status.
        let has_failure = state.node_results.values().any(|r| {
            matches!(
                r.status,
                WorkflowStatus::Failed { .. } | WorkflowStatus::Cancelled
            )
        });

        if has_failure {
            let error_msg = state
                .node_results
                .values()
                .find_map(|r| {
                    if let WorkflowStatus::Failed { error } = &r.status {
                        Some(error.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Unknown workflow failure".to_string());
            state.status = WorkflowStatus::Failed {
                error: error_msg,
            };
        } else {
            state.status = WorkflowStatus::Completed;
        }
        state.completed_at = Some(now_ms());

        Ok(())
    }
}

/// Cancel all downstream nodes of a failed node (fail-fast strategy).
fn cancel_downstream(
    failed_id: &str,
    children: &HashMap<String, Vec<String>>,
    state: &mut WorkflowState,
    completed: &mut HashSet<String>,
    ready: &mut VecDeque<String>,
) {
    if let Some(child_ids) = children.get(failed_id) {
        for child_id in child_ids {
            if completed.contains(child_id) {
                continue;
            }
            // Remove from ready queue if present.
            ready.retain(|id| id != child_id);

            state.node_results.insert(
                child_id.clone(),
                NodeResult {
                    node_id: child_id.clone(),
                    status: WorkflowStatus::Cancelled,
                    output: None,
                },
            );
            completed.insert(child_id.clone());

            // Recursively cancel further downstream.
            cancel_downstream(child_id, children, state, completed, ready);
        }
    }
}

/// Current time in milliseconds since Unix epoch.
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::{EdgeSpec, NodeType, NodeSpec, WorkflowSpec};

    #[tokio::test]
    async fn test_linear_dag() {
        let spec = WorkflowSpec {
            id: "test-linear".to_string(),
            name: "Linear".to_string(),
            nodes: vec![
                NodeSpec {
                    id: "a".to_string(),
                    node_type: NodeType::Passthrough,
                    depends_on: vec![],
                },
                NodeSpec {
                    id: "b".to_string(),
                    node_type: NodeType::Passthrough,
                    depends_on: vec!["a".to_string()],
                },
                NodeSpec {
                    id: "c".to_string(),
                    node_type: NodeType::Passthrough,
                    depends_on: vec!["b".to_string()],
                },
            ],
            edges: vec![
                EdgeSpec {
                    from: "a".to_string(),
                    to: "b".to_string(),
                },
                EdgeSpec {
                    from: "b".to_string(),
                    to: "c".to_string(),
                },
            ],
        };

        let mut state = WorkflowState::default();
        let executor = WorkflowExecutor::new(4);

        executor
            .execute(&spec, &mut state, |node| {
                Box::pin(async move { Ok(serde_json::json!({ "node": node.id })) })
            })
            .await
            .unwrap();

        assert_eq!(state.status, WorkflowStatus::Completed);
        assert_eq!(state.node_results.len(), 3);
        assert_eq!(
            state.node_results["a"].status,
            WorkflowStatus::Completed
        );
        assert_eq!(
            state.node_results["b"].status,
            WorkflowStatus::Completed
        );
        assert_eq!(
            state.node_results["c"].status,
            WorkflowStatus::Completed
        );
    }

    #[tokio::test]
    async fn test_parallel_dag() {
        let spec = WorkflowSpec {
            id: "test-parallel".to_string(),
            name: "Parallel".to_string(),
            nodes: vec![
                NodeSpec {
                    id: "a".to_string(),
                    node_type: NodeType::Passthrough,
                    depends_on: vec![],
                },
                NodeSpec {
                    id: "b".to_string(),
                    node_type: NodeType::Passthrough,
                    depends_on: vec![],
                },
                NodeSpec {
                    id: "c".to_string(),
                    node_type: NodeType::Passthrough,
                    depends_on: vec!["a".to_string(), "b".to_string()],
                },
            ],
            edges: vec![
                EdgeSpec {
                    from: "a".to_string(),
                    to: "c".to_string(),
                },
                EdgeSpec {
                    from: "b".to_string(),
                    to: "c".to_string(),
                },
            ],
        };

        let mut state = WorkflowState::default();
        let executor = WorkflowExecutor::new(4);

        executor
            .execute(&spec, &mut state, |node| {
                Box::pin(async move { Ok(serde_json::json!({ "node": node.id })) })
            })
            .await
            .unwrap();

        assert_eq!(state.status, WorkflowStatus::Completed);
        assert_eq!(state.node_results.len(), 3);
    }

    #[tokio::test]
    async fn test_failure_cancels_downstream() {
        let spec = WorkflowSpec {
            id: "test-fail".to_string(),
            name: "Fail".to_string(),
            nodes: vec![
                NodeSpec {
                    id: "a".to_string(),
                    node_type: NodeType::Passthrough,
                    depends_on: vec![],
                },
                NodeSpec {
                    id: "b".to_string(),
                    node_type: NodeType::Passthrough,
                    depends_on: vec!["a".to_string()],
                },
            ],
            edges: vec![EdgeSpec {
                from: "a".to_string(),
                to: "b".to_string(),
            }],
        };

        let mut state = WorkflowState::default();
        let executor = WorkflowExecutor::new(4);

        executor
            .execute(&spec, &mut state, |node| {
                Box::pin(async move {
                    if node.id == "a" {
                        Err("node a failed".to_string())
                    } else {
                        Ok(serde_json::json!({ "node": node.id }))
                    }
                })
            })
            .await
            .unwrap();

        assert!(matches!(state.status, WorkflowStatus::Failed { .. }));
        assert!(matches!(
            state.node_results["a"].status,
            WorkflowStatus::Failed { .. }
        ));
        assert_eq!(
            state.node_results["b"].status,
            WorkflowStatus::Cancelled
        );
    }
}
