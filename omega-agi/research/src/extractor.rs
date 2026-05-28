//! Intelligent content extraction and summarization
//!
//! Extracts key points, generates summaries, and scores
//! relevance using statistical and heuristic methods.

use crate::searcher::FetchedContent;

/// Extraction result from research content
#[derive(Debug, Clone)]
pub struct Extraction {
    pub summary: String,
    pub key_points: Vec<String>,
    pub relevance: f64,
    pub word_count: usize,
}

/// The Extractor — transforms raw content into structured knowledge
pub struct Extractor {
    min_relevance: f64,
}

impl Extractor {
    pub fn new(min_relevance: f64) -> Self {
        Self { min_relevance }
    }

    /// Extract structured knowledge from fetched sources
    pub async fn extract(&self, topic: &str, sources: &[FetchedContent]) -> Extraction {
        if sources.is_empty() {
            return Extraction {
                summary: format!("No sources found for '{}'", topic),
                key_points: vec![],
                relevance: 0.0,
                word_count: 0,
            };
        }

        let topic_lower = topic.to_lowercase();
        let mut all_points: Vec<String> = Vec::new();
        let mut total_words = 0;
        let mut topic_hits = 0;

        for source in sources {
            total_words += source.word_count;
            let content_lower = source.content.to_lowercase();

            // Count topic-relevant terms
            for word in topic_lower.split_whitespace() {
                if content_lower.contains(word) {
                    topic_hits += 1;
                }
            }

            // Extract sentences that contain topic keywords
            for sentence in source.content.split(|c| c == '.' || c == '!' || c == '?') {
                let sent_lower = sentence.to_lowercase().trim().to_string();
                if sent_lower.len() < 20 {
                    continue;
                }
                let has_keyword = topic_lower
                    .split_whitespace()
                    .any(|w| sent_lower.contains(w));
                if has_keyword {
                    let point = sentence.trim().to_string();
                    if !all_points.contains(&point) {
                        all_points.push(point);
                    }
                }
            }
        }

        // Calculate relevance score
        let relevance = if total_words > 0 {
            let hit_ratio = topic_hits as f64 / (sources.len() as f64 * 3.0).max(1.0);
            (0.3 + hit_ratio * 0.7).min(1.0)
        } else {
            0.0
        };

        // Generate summary (first meaningful paragraph)
        let summary = if all_points.is_empty() {
            format!(
                "Research on '{}' found {} sources with {} total words. Key insights are being compiled.",
                topic, sources.len(), total_words
            )
        } else {
            let top: Vec<&str> = all_points.iter().take(3).map(|s| s.as_str()).collect();
            top.join(". ")
        };

        // Deduplicate and limit key points
        all_points.sort_by(|a, b| {
            let a_rel = topic_lower
                .split_whitespace()
                .filter(|w| a.to_lowercase().contains(w))
                .count();
            let b_rel = topic_lower
                .split_whitespace()
                .filter(|w| b.to_lowercase().contains(w))
                .count();
            b_rel.cmp(&a_rel)
        });
        all_points.truncate(10);

        Extraction {
            summary,
            key_points: all_points,
            relevance,
            word_count: total_words,
        }
    }

    /// Score a piece of text for relevance to a topic
    pub fn score_relevance(&self, text: &str, topic: &str) -> f64 {
        let text_lower = text.to_lowercase();
        let topic_lower = topic.to_lowercase();
        let words: Vec<&str> = topic_lower.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }
        let hits = words.iter().filter(|w| text_lower.contains(*w)).count();
        hits as f64 / words.len() as f64
    }
}
