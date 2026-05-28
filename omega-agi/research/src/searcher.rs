//! Web search and content discovery engine
//!
//! Searches for relevant sources, simulates web fetches,
//! and scores results by relevance to the research topic.

use std::collections::HashMap;

/// A discovered source
#[derive(Debug, Clone)]
pub struct Source {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub relevance: f64,
}

/// Content fetcher result
#[derive(Debug, Clone)]
pub struct FetchedContent {
    pub url: String,
    pub title: String,
    pub content: String,
    pub word_count: usize,
}

/// The Searcher — discovers and fetches research sources
pub struct Searcher {
    search_depth: usize,
    /// Built-in knowledge base for offline research
    knowledge_base: HashMap<String, Vec<(String, String)>>,
}

impl Searcher {
    pub fn new(search_depth: usize) -> Self {
        let mut kb = HashMap::new();

        // Seed knowledge base with general AGI/AI topics
        kb.insert("agi".to_string(), vec![
            ("What is AGI?".into(), "Artificial General Intelligence (AGI) is a hypothetical type of intelligence that can understand, learn, and apply knowledge across a wide range of tasks at a level equal to or beyond human capability. Unlike narrow AI, which excels at specific tasks, AGI would possess generalized cognitive abilities.".into()),
            ("AGI Architecture".into(), "Modern AGI architectures typically use layered approaches combining neural networks, symbolic reasoning, and evolutionary algorithms. Key components include memory systems, attention mechanisms, and self-improvement loops.".into()),
        ]);
        kb.insert("self evolution".to_string(), vec![
            ("Self-Evolving Systems".into(), "Self-evolving AI systems use genetic algorithms, reinforcement learning, and meta-learning to continuously improve their own architecture and parameters. The APEX*∞ formula (Φ_APEX*∞) represents a mathematical framework for measuring and guiding this evolution.".into()),
        ]);
        kb.insert("rust".to_string(), vec![
            ("Rust Programming".into(), "Rust is a systems programming language focused on safety, speed, and concurrency. Its ownership model ensures memory safety without a garbage collector, making it ideal for high-performance AI systems.".into()),
        ]);

        Self { search_depth, knowledge_base: kb }
    }

    /// Search for sources on a topic
    pub async fn search(&self, topic: &str) -> Vec<Source> {
        let topic_lower = topic.to_lowercase();
        let mut sources = Vec::new();

        // Check knowledge base
        for (key, entries) in &self.knowledge_base {
            if topic_lower.contains(key) || key.contains(&topic_lower) {
                for (i, (title, content)) in entries.iter().enumerate() {
                    sources.push(Source {
                        url: format!("knowledge://{}/entry_{}", key, i),
                        title: title.clone(),
                        snippet: content[..100.min(content.len())].to_string(),
                        relevance: 0.8 + (i as f64 * 0.05).min(0.15),
                    });
                }
            }
        }

        // Add generic results based on topic keywords
        let words: Vec<&str> = topic_lower.split_whitespace().collect();
        for word in &words {
            if word.len() > 3 {
                sources.push(Source {
                    url: format!("https://research.omega-agi.org/search?q={}", word),
                    title: format!("Research results for '{}'", word),
                    snippet: format!("Autonomous research findings related to {}...", word),
                    relevance: 0.5,
                });
            }
        }

        // Deduplicate and sort by relevance
        sources.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        sources.truncate(self.search_depth * 5);
        sources
    }

    /// Fetch full content from a source
    pub async fn fetch(&self, source: &Source) -> Option<FetchedContent> {
        // For knowledge base sources, return the stored content
        if source.url.starts_with("knowledge://") {
            let path = source.url.trim_start_matches("knowledge://");
            let parts: Vec<&str> = path.split("/entry_").collect();
            if parts.len() == 2 {
                if let Some(entries) = self.knowledge_base.get(parts[0]) {
                    let idx: usize = parts[1].parse().unwrap_or(0);
                    if idx < entries.len() {
                        return Some(FetchedContent {
                            url: source.url.clone(),
                            title: entries[idx].0.clone(),
                            content: entries[idx].1.clone(),
                            word_count: entries[idx].1.split_whitespace().count(),
                        });
                    }
                }
            }
        }

        // For external URLs, return snippet as content
        Some(FetchedContent {
            url: source.url.clone(),
            title: source.title.clone(),
            content: source.snippet.clone(),
            word_count: source.snippet.split_whitespace().count(),
        })
    }

    /// Search and fetch in one call
    pub async fn search_and_fetch(&self, topic: &str) -> Vec<FetchedContent> {
        let sources = self.search(topic).await;
        let mut contents = Vec::new();
        for source in &sources {
            if let Some(content) = self.fetch(source).await {
                contents.push(content);
            }
        }
        contents
    }
}
