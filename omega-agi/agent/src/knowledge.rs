//! # KnowledgeBase — Structured knowledge management for Agents
//!
//! Combines three retrieval paradigms:
//! 1. **Vector Store** — semantic search over document chunks
//! 2. **Knowledge Graph** — entity-relation-entity triples
//! 3. **Document Store** — raw document storage with metadata
//!
//! This is what gives the Agent the ability to **learn** from its experiences
//! and **recall** relevant information across sessions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::inference::InferenceEngine;

// ---------------------------------------------------------------------------
// Knowledge types
// ---------------------------------------------------------------------------

/// A chunk of knowledge with embedding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    pub id: String,
    pub source: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A relationship triple in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
    pub source: String,
}

/// A document with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source: String,
    pub content_type: String,
    pub chunk_ids: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Search result with relevance score.
#[derive(Debug, Clone)]
pub struct KnowledgeResult {
    pub chunk: KnowledgeChunk,
    pub score: f32,
    pub matched_by: String, // "vector" | "graph" | "keyword"
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct KnowledgeConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_chunks_per_doc: usize,
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,   // chars per chunk
            chunk_overlap: 64, // overlap between chunks
            max_chunks_per_doc: 100,
        }
    }
}

// ---------------------------------------------------------------------------
// KnowledgeBase
// ---------------------------------------------------------------------------

/// The full knowledge management system.
pub struct KnowledgeBase {
    config: KnowledgeConfig,
    embedder: Arc<dyn InferenceEngine>,

    /// All knowledge chunks (in-memory index).
    chunks: RwLock<Vec<KnowledgeChunk>>,
    /// Knowledge graph triples.
    graph: RwLock<Vec<Triple>>,
    /// Raw documents.
    documents: RwLock<Vec<Document>>,
    /// Simple inverted keyword index for fallback search.
    keyword_index: RwLock<HashMap<String, Vec<usize>>>,
}

impl KnowledgeBase {
    pub fn new(config: KnowledgeConfig, embedder: Arc<dyn InferenceEngine>) -> Self {
        Self {
            config,
            embedder,
            chunks: RwLock::new(Vec::new()),
            graph: RwLock::new(Vec::new()),
            documents: RwLock::new(Vec::new()),
            keyword_index: RwLock::new(HashMap::new()),
        }
    }

    /// Learn from a text source: chunk → embed → store.
    pub async fn learn(&self, source: &str, content: &str) -> anyhow::Result<Vec<String>> {
        let chunks = self.chunk_text(content);
        let texts: Vec<&str> = chunks.iter().map(|c| c.as_str()).collect();

        let embeddings = self.embedder.embed(&texts).await.ok();

        let mut stored_ids = Vec::new();
        let mut chunk_entries = Vec::new();
        let mut kw_index = self.keyword_index.write().await;

        for (i, chunk_text) in chunks.iter().enumerate() {
            let id = format!(
                "kb-{}-{}",
                source
                    .chars()
                    .filter(|&c| c.is_alphanumeric())
                    .take(16)
                    .collect::<String>(),
                i
            );
            let emb = embeddings
                .as_ref()
                .and_then(|r| r.embeddings.get(i))
                .map(|e| e.vector.clone());

            let chunk = KnowledgeChunk {
                id: id.clone(),
                source: source.to_string(),
                content: chunk_text.clone(),
                embedding: emb,
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
            };

            // Build keyword index
            for word in chunk_text.split_whitespace() {
                let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
                if clean.len() > 2 {
                    kw_index.entry(clean.to_lowercase()).or_default().push(i);
                }
            }

            stored_ids.push(id);
            chunk_entries.push(chunk);
        }

        let mut chunks = self.chunks.write().await;
        chunks.extend(chunk_entries);

        Ok(stored_ids)
    }

    /// Recall knowledge relevant to a query.
    pub async fn recall(&self, query: &str, top_k: usize) -> Vec<KnowledgeResult> {
        let mut results: Vec<KnowledgeResult> = Vec::new();

        // 1. Vector search
        if let Ok(emb_res) = self.embedder.embed(&[query]).await {
            if let Some(q_emb) = emb_res.embeddings.first() {
                let chunks = self.chunks.read().await;
                for chunk in chunks.iter() {
                    if let Some(ref c_emb) = chunk.embedding {
                        let score = cosine_similarity(&q_emb.vector, c_emb);
                        results.push(KnowledgeResult {
                            chunk: chunk.clone(),
                            score,
                            matched_by: "vector".into(),
                        });
                    }
                }
            }
        }

        // 2. Graph search — find triples matching query terms
        let query_lower = query.to_lowercase();
        let graph = self.graph.read().await;
        for triple in graph.iter() {
            if triple.subject.to_lowercase().contains(&query_lower)
                || triple.object.to_lowercase().contains(&query_lower)
                || triple.predicate.to_lowercase().contains(&query_lower)
            {
                let chunk = KnowledgeChunk {
                    id: format!(
                        "graph-{}-{}-{}",
                        triple.subject, triple.predicate, triple.object
                    ),
                    source: triple.source.clone(),
                    content: format!(
                        "{} → {} → {}",
                        triple.subject, triple.predicate, triple.object
                    ),
                    embedding: None,
                    metadata: HashMap::new(),
                    created_at: chrono::Utc::now(),
                };
                results.push(KnowledgeResult {
                    chunk,
                    score: triple.confidence * 0.7,
                    matched_by: "graph".into(),
                });
            }
        }
        drop(graph);

        // 3. Keyword fallback
        let kw_index = self.keyword_index.read().await;
        let query_words: Vec<String> = query_lower
            .split_whitespace()
            .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|w: &String| w.len() > 2)
            .collect();

        let chunks = self.chunks.read().await;
        for word in &query_words {
            if let Some(indices) = kw_index.get(word) {
                for &idx in indices {
                    if let Some(chunk) = chunks.get(idx) {
                        results.push(KnowledgeResult {
                            chunk: chunk.clone(),
                            score: 0.3,
                            matched_by: "keyword".into(),
                        });
                    }
                }
            }
        }
        drop(chunks);

        // Deduplicate by chunk ID, keep highest score
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut seen = std::collections::HashSet::new();
        results.retain(|r| seen.insert(r.chunk.id.clone()));
        results.truncate(top_k);

        results
    }

    /// Add a triple to the knowledge graph.
    pub fn add_triple(&self, subject: &str, predicate: &str, object: &str, source: &str) {
        let triple = Triple {
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: object.to_string(),
            confidence: 1.0,
            source: source.to_string(),
        };
        // Use blocking spawn for sync code in async context
        let graph = self.graph.blocking_write();
        // Deduplicate
        if !graph.iter().any(|t| {
            t.subject == triple.subject
                && t.predicate == triple.predicate
                && t.object == triple.object
        }) {
            // TODO: use blocking_write
        }
    }

    /// Add a document and auto-chunk it.
    pub async fn add_document(
        &self,
        title: &str,
        content: &str,
        source: &str,
        content_type: &str,
    ) -> anyhow::Result<String> {
        let doc_id = format!("doc-{}", chrono::Utc::now().timestamp_nanos());
        let chunk_ids = self.learn(&doc_id, content).await?;

        let doc = Document {
            id: doc_id.clone(),
            title: title.to_string(),
            content: content.to_string(),
            source: source.to_string(),
            content_type: content_type.to_string(),
            chunk_ids,
            created_at: chrono::Utc::now(),
        };

        self.documents.write().await.push(doc);
        Ok(doc_id)
    }

    /// Search for documents by title.
    pub async fn search_documents(&self, query: &str) -> Vec<Document> {
        let q = query.to_lowercase();
        self.documents
            .read()
            .await
            .iter()
            .filter(|d| {
                d.title.to_lowercase().contains(&q) || d.content.to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }

    /// Get knowledge base statistics.
    pub async fn stats(&self) -> serde_json::Value {
        let chunks = self.chunks.read().await.len();
        let graph = self.graph.read().await.len();
        let docs = self.documents.read().await.len();
        serde_json::json!({
            "chunks": chunks,
            "graph_triples": graph,
            "documents": docs,
        })
    }

    /// Split text into chunks.
    fn chunk_text(&self, text: &str) -> Vec<String> {
        if text.len() <= self.config.chunk_size {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut start = 0;
        let overlap = self.config.chunk_overlap;

        while start < text.len() && chunks.len() < self.config.max_chunks_per_doc {
            let end = std::cmp::min(start + self.config.chunk_size, text.len());
            // Try to break at a sentence boundary
            let mut break_at = end;
            if end < text.len() {
                if let Some(nl_pos) = text[end.saturating_sub(100)..end].rfind('\n') {
                    break_at = end.saturating_sub(100) + nl_pos;
                } else if let Some(period_pos) = text[end.saturating_sub(100)..end].rfind(". ") {
                    break_at = end.saturating_sub(100) + period_pos + 1;
                }
            }
            chunks.push(text[start..break_at].to_string());
            start = break_at.saturating_sub(overlap);
        }

        chunks
    }
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::MockEngine;

    #[tokio::test]
    async fn test_learn_and_recall() {
        let embedder = Arc::new(MockEngine::new());
        let kb = KnowledgeBase::new(KnowledgeConfig::default(), embedder);

        kb.learn(
            "test",
            "Rust is a systems programming language focused on safety and performance.",
        )
        .await
        .unwrap();

        let results = kb.recall("programming language", 5).await;
        assert!(!results.is_empty(), "Should find at least one result");
        assert_eq!(results[0].chunk.source, "test");
    }

    #[tokio::test]
    async fn test_document_add_and_search() {
        let embedder = Arc::new(MockEngine::new());
        let kb = KnowledgeBase::new(KnowledgeConfig::default(), embedder);

        kb.add_document(
            "Rust Book",
            "Rust programming language introduction",
            "book",
            "text/markdown",
        )
        .await
        .unwrap();
        let docs = kb.search_documents("Rust").await;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "Rust Book");
    }

    #[test]
    fn test_chunking() {
        let embedder = MockEngine::new();
        let kb = KnowledgeBase::new(
            KnowledgeConfig {
                chunk_size: 20,
                chunk_overlap: 5,
                max_chunks_per_doc: 10,
            },
            Arc::new(embedder),
        );

        let text =
            "This is a longer text that should be split into multiple chunks for processing.";
        let chunks = kb.chunk_text(text);
        assert!(chunks.len() > 1, "Text should be split into chunks");
        assert!(chunks.iter().all(|c| c.len() <= 25)); // chunk_size + small slack
    }

    #[test]
    fn test_small_text_no_chunking() {
        let embedder = MockEngine::new();
        let kb = KnowledgeBase::new(KnowledgeConfig::default(), Arc::new(embedder));

        let text = "Short text.";
        let chunks = kb.chunk_text(text);
        assert_eq!(chunks.len(), 1);
    }
}
