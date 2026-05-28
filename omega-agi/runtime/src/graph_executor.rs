//! # Runtime Graph Executor
//! Executes task graphs (DAGs) for complex multi-step workflows.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

/// Unique identifier for a graph node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

/// Result of executing a single node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult {
    pub node_id: NodeId,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Errors that can occur during graph execution.
#[derive(Error, Debug)]
pub enum GraphExecutorError {
    #[error("cycle detected in task graph")]
    CycleDetected,

    #[error("node not found: {0}")]
    NodeNotFound(String),

    #[error("execution failed for node {node}: {details}")]
    ExecutionFailed { node: String, details: String },

    #[error("dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),
}

/// A dependency graph / task graph for multi-step execution.
#[derive(Debug, Clone)]
pub struct TaskGraph {
    pub name: String,
    nodes: HashMap<NodeId, Vec<NodeId>>, // node -> dependencies
    node_data: HashMap<NodeId, String>,  // node -> description
}

impl TaskGraph {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nodes: HashMap::new(),
            node_data: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: NodeId, description: &str) {
        self.nodes.entry(id.clone()).or_default();
        self.node_data.insert(id, description.to_string());
    }

    pub fn add_dependency(&mut self, from: NodeId, to: NodeId) -> Result<(), GraphExecutorError> {
        // Check for immediate cycles
        if from == to {
            return Err(GraphExecutorError::CycleDetected);
        }
        self.nodes.entry(from.clone()).or_default();
        self.nodes.get_mut(&from).unwrap().push(to);
        Ok(())
    }

    pub fn topological_sort(&self) -> Result<Vec<NodeId>, GraphExecutorError> {
        let mut in_degree: HashMap<&NodeId, usize> = HashMap::new();
        let mut adj: HashMap<&NodeId, Vec<&NodeId>> = HashMap::new();

        for (node, deps) in &self.nodes {
            in_degree.entry(node).or_insert(0);
            for dep in deps {
                *in_degree.entry(dep).or_insert(0) += 1;
                adj.entry(dep).or_default().push(node);
            }
        }

        let mut queue: VecDeque<&NodeId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(node, _)| *node)
            .collect();

        let mut result = Vec::new();
        while let Some(node) = queue.pop_front() {
            result.push(node.clone());
            for next in adj.get(node).into_iter().flatten() {
                if let Some(deg) = in_degree.get_mut(next) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(next);
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err(GraphExecutorError::CycleDetected);
        }

        Ok(result)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_description(&self, id: &NodeId) -> Option<&str> {
        self.node_data.get(id).map(|s| s.as_str())
    }
}

/// Executes task graphs by traversing nodes in dependency order.
pub struct GraphExecutor {
    graphs: HashMap<String, TaskGraph>,
}

impl GraphExecutor {
    pub fn new() -> Self {
        Self {
            graphs: HashMap::new(),
        }
    }

    pub fn register_graph(&mut self, graph: TaskGraph) {
        self.graphs.insert(graph.name.clone(), graph);
    }

    pub fn execute_graph(&self, name: &str) -> Result<Vec<NodeResult>, GraphExecutorError> {
        let graph = self
            .graphs
            .get(name)
            .ok_or_else(|| GraphExecutorError::NodeNotFound(name.to_string()))?;

        let order = graph.topological_sort()?;
        let mut results = Vec::new();

        for node_id in &order {
            let start = std::time::Instant::now();
            let description = graph.get_description(node_id).unwrap_or("unknown");

            let result = NodeResult {
                node_id: node_id.clone(),
                success: true,
                output: Some(serde_json::json!({
                    "node": node_id.0,
                    "description": description,
                })),
                error: None,
                duration_ms: start.elapsed().as_millis() as u64,
            };

            tracing::info!(node = %node_id, "Graph node executed in {}ms", result.duration_ms);
            results.push(result);
        }

        Ok(results)
    }

    pub fn get_graph(&self, name: &str) -> Option<&TaskGraph> {
        self.graphs.get(name)
    }

    pub fn list_graphs(&self) -> Vec<String> {
        self.graphs.keys().cloned().collect()
    }
}

impl Default for GraphExecutor {
    fn default() -> Self {
        Self::new()
    }
}
