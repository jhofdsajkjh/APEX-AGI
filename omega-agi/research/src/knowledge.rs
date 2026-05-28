//! Knowledge graph for cross-referencing and insight discovery
//!
//! Maintains relationships between topics, sources, and key points,
//! enabling the research engine to discover non-obvious connections.

use crate::searcher::FetchedContent;
use std::collections::{HashMap, HashSet};

/// A node in the knowledge graph
#[derive(Debug, Clone)]
struct KnowledgeNode {
    label: String,
    node_type: NodeType,
    connections: Vec<String>,
    weight: f64,
}

#[derive(Debug, Clone, PartialEq)]
enum NodeType {
    Topic,
    Source,
    Concept,
}

/// The KnowledgeGraph — learns from research and discovers patterns
pub struct KnowledgeGraph {
    enabled: bool,
    nodes: HashMap<String, KnowledgeNode>,
    insights: Vec<String>,
}

impl KnowledgeGraph {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            nodes: HashMap::new(),
            insights: Vec::new(),
        }
    }

    /// Ingest research results into the knowledge graph
    pub fn ingest(&mut self, topic: &str, sources: &[FetchedContent], key_points: &[String]) {
        if !self.enabled {
            return;
        }

        // Add topic node
        self.nodes.entry(topic.to_string()).or_insert_with(|| KnowledgeNode {
            label: topic.to_string(),
            node_type: NodeType::Topic,
            connections: Vec::new(),
            weight: 1.0,
        });

        // Connect sources to topic
        for source in sources {
            let source_key = source.url.clone();
            let topic_node = self.nodes.get_mut(topic);
            if let Some(tn) = topic_node {
                if !tn.connections.contains(&source_key) {
                    tn.connections.push(source_key.clone());
                }
            }

            self.nodes.entry(source_key.clone()).or_insert_with(|| KnowledgeNode {
                label: source.title.clone(),
                node_type: NodeType::Source,
                connections: vec![topic.to_string()],
                weight: 0.8,
            });
        }

        // Extract and connect concepts
        for point in key_points {
            let words: Vec<&str> = point
                .split_whitespace()
                .filter(|w| w.len() > 5)
                .collect();
            for word in words {
                let concept = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
                if concept.len() < 4 {
                    continue;
                }

                if let Some(node) = self.nodes.get_mut(&concept) {
                    if !node.connections.contains(&topic.to_string()) {
                        node.connections.push(topic.to_string());
                    }
                    node.weight = (node.weight + 0.1).min(1.0);
                } else {
                    self.nodes.insert(concept.clone(), KnowledgeNode {
                        label: concept.clone(),
                        node_type: NodeType::Concept,
                        connections: vec![topic.to_string()],
                        weight: 0.3,
                    });
                }
            }
        }

        // Discover cross-topic insights
        self.discover_insights();
    }

    fn discover_insights(&mut self) {
        let mut new_insights = Vec::new();

        // Find concepts that connect multiple topics
        let topics: HashSet<String> = self.nodes.iter()
            .filter(|(_, n)| n.node_type == NodeType::Topic)
            .map(|(k, _)| k.clone())
            .collect();

        for (label, node) in &self.nodes {
            if node.node_type != NodeType::Concept {
                continue;
            }
            let connected_topics: Vec<&String> = node.connections
                .iter()
                .filter(|c| topics.contains(c.as_str()))
                .collect();
            if connected_topics.len() >= 2 {
                let insight = format!(
                    "Cross-topic connection: '{}' links [{}]",
                    label,
                    connected_topics.join(", ")
                );
                if !self.insights.contains(&insight) {
                    new_insights.push(insight);
                }
            }
        }

        self.insights.extend(new_insights);
    }

    /// Get discovered insights
    pub fn get_insights(&self) -> Vec<String> {
        self.insights.clone()
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
