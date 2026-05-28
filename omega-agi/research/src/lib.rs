//! # OMEGA AGI - Autoresearch Engine (Layer 6)
//!
//! Autonomous research system with:
//! - **Searcher**: web search and content discovery
//! - **Extractor**: intelligent content extraction and summarization
//! - **Reporter**: automatic report generation (Markdown/JSON)
//! - **Knowledge**: memory graph for cross-reference insights
//!
//! ## Architecture
//!
//! ```text
//! Researcher
//! ├── Searcher   (discovery + fetch)
//! ├── Extractor  (summary + key points)
//! ├── Reporter   (format + output)
//! └── Knowledge  (graph + cross-ref)
//! ```

pub mod extractor;
pub mod reporter;
pub mod searcher;

pub mod knowledge;

use extractor::Extractor;
use knowledge::KnowledgeGraph;
use reporter::{Report, ReportFormat, Reporter};
use searcher::{FetchedContent, Searcher};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

/// Configuration for the research engine
#[derive(Debug, Clone)]
pub struct ResearchConfig {
    /// Max concurrent research threads
    pub max_concurrent: usize,
    /// Default search depth (pages to follow)
    pub search_depth: usize,
    /// Minimum relevance score (0.0-1.0)
    pub min_relevance: f64,
    /// Auto-generate reports on completion
    pub auto_report: bool,
    /// Report output format
    pub report_format: ReportFormat,
    /// Enable knowledge graph cross-referencing
    pub enable_knowledge_graph: bool,
    /// Research timeout in seconds
    pub timeout_secs: u64,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 3,
            search_depth: 2,
            min_relevance: 0.3,
            auto_report: true,
            report_format: ReportFormat::Markdown,
            enable_knowledge_graph: true,
            timeout_secs: 120,
        }
    }
}

/// A single research result entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResearchResult {
    /// Unique research ID
    pub id: String,
    /// Research query/topic
    pub topic: String,
    /// When the research was conducted
    pub timestamp: String,
    /// Number of sources found
    pub sources_found: usize,
    /// Summary of findings
    pub summary: String,
    /// Key points extracted
    pub key_points: Vec<String>,
    /// Source URLs or references
    pub sources: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Relevance confidence score
    pub relevance_score: f64,
    /// Full generated report (if auto_report enabled)
    pub report: Option<String>,
    /// Error message if research failed
    pub error: Option<String>,
}

impl ResearchResult {
    pub fn failed(topic: &str, error: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            topic: topic.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            sources_found: 0,
            summary: String::new(),
            key_points: vec![],
            sources: vec![],
            tags: vec![],
            relevance_score: 0.0,
            report: None,
            error: Some(error.to_string()),
        }
    }
}

/// The main research engine — orchestrates search, extraction, and reporting
pub struct Researcher {
    config: ResearchConfig,
    searcher: Searcher,
    extractor: Extractor,
    reporter: Reporter,
    knowledge: Mutex<KnowledgeGraph>,
    history: Arc<RwLock<Vec<ResearchResult>>>,
}

impl Researcher {
    /// Create a new Researcher with default configuration
    pub fn new() -> Self {
        Self::with_config(ResearchConfig::default())
    }

    /// Create a new Researcher with custom configuration
    pub fn with_config(config: ResearchConfig) -> Self {
        Self {
            searcher: Searcher::new(config.search_depth),
            extractor: Extractor::new(config.min_relevance),
            reporter: Reporter::new(config.report_format.clone()),
            knowledge: Mutex::new(KnowledgeGraph::new(config.enable_knowledge_graph)),
            history: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Conduct autonomous research on a topic
    pub async fn research(&self, topic: &str) -> ResearchResult {
        tracing::info!(topic = %topic, "Starting autonomous research");

        // Step 1: Discover sources
        let sources = self.searcher.search(topic).await;
        if sources.is_empty() {
            return ResearchResult::failed(topic, "No sources found");
        }

        // Convert sources to FetchedContent for downstream consumers
        let fetched: Vec<FetchedContent> = sources
            .iter()
            .map(|s| FetchedContent {
                url: s.url.clone(),
                title: s.title.clone(),
                content: s.snippet.clone(),
                word_count: s.snippet.split_whitespace().count(),
            })
            .collect();

        // Step 2: Extract content & summarize
        let extraction = self.extractor.extract(topic, &fetched).await;

        // Step 3: Update knowledge graph
        if self.config.enable_knowledge_graph {
            self.knowledge
                .lock()
                .unwrap()
                .ingest(topic, &fetched, &extraction.key_points);
        }

        // Step 4: Generate report
        let report = if self.config.auto_report {
            Some(self.reporter.generate(topic, &extraction, &fetched))
        } else {
            None
        };

        let result = ResearchResult {
            id: uuid::Uuid::new_v4().to_string(),
            topic: topic.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            sources_found: sources.len(),
            summary: extraction.summary,
            key_points: extraction.key_points.clone(),
            sources: sources.iter().map(|s| s.url.clone()).collect(),
            tags: extract_tags(topic, &extraction.key_points),
            relevance_score: extraction.relevance,
            report,
            error: None,
        };

        // Store in history
        {
            let mut history = self.history.write().await;
            history.push(result.clone());
            if history.len() > 100 {
                history.remove(0);
            }
        }

        tracing::info!(topic = %topic, sources = %result.sources_found, "Research complete");
        result
    }

    /// Conduct multi-topic research and produce a comparative report
    pub async fn compare(&self, topics: &[&str]) -> Vec<ResearchResult> {
        let mut results = Vec::new();
        for topic in topics {
            let result = self.research(topic).await;
            results.push(result);
        }
        results
    }

    /// Get research history
    pub async fn get_history(&self) -> Vec<ResearchResult> {
        self.history.read().await.clone()
    }

    /// Get knowledge graph insights
    pub fn get_knowledge_insights(&self) -> Vec<String> {
        self.knowledge.lock().unwrap().get_insights()
    }

    /// Version string
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl Default for Researcher {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_tags(topic: &str, key_points: &[String]) -> Vec<String> {
    let mut tags: Vec<String> = topic
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .map(|w| w.to_lowercase())
        .collect();
    for point in key_points.iter().take(5) {
        for word in point.split_whitespace() {
            let w = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if w.len() > 4 && !tags.contains(&w) {
                tags.push(w);
            }
        }
    }
    tags.truncate(10);
    tags
}
