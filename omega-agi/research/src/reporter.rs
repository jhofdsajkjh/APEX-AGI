//! Automatic report generation
//!
//! Generates well-formatted research reports in Markdown or JSON format,
//! with proper citations, structure, and metadata.

use crate::extractor::Extraction;
use crate::searcher::FetchedContent;

/// Output format for reports
#[derive(Debug, Clone)]
pub enum ReportFormat {
    Markdown,
    Json,
    Text,
}

impl Default for ReportFormat {
    fn default() -> Self {
        Self::Markdown
    }
}

/// A generated research report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub title: String,
    pub format: String,
    pub content: String,
    pub timestamp: String,
    pub word_count: usize,
}

/// The Reporter — generates formatted research reports
pub struct Reporter {
    format: ReportFormat,
}

impl Reporter {
    pub fn new(format: ReportFormat) -> Self {
        Self { format }
    }

    /// Generate a complete research report
    pub fn generate(&self, topic: &str, extraction: &Extraction, sources: &[FetchedContent]) -> String {
        match self.format {
            ReportFormat::Markdown => self.generate_markdown(topic, extraction, sources),
            ReportFormat::Json => self.generate_json(topic, extraction, sources),
            ReportFormat::Text => self.generate_text(topic, extraction, sources),
        }
    }

    fn generate_markdown(&self, topic: &str, extraction: &Extraction, sources: &[FetchedContent]) -> String {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let mut report = String::new();

        report.push_str(&format!("# 🔬 Research Report: {}\n\n", topic));
        report.push_str(&format!("> Generated: {}  \n", timestamp));
        report.push_str(&format!("> Sources: {}  \n", sources.len()));
        report.push_str(&format!("> Relevance: {:.1}%  \n\n", extraction.relevance * 100.0));

        report.push_str("## 📋 Executive Summary\n\n");
        report.push_str(&extraction.summary);
        report.push_str("\n\n");

        report.push_str("## 🔑 Key Findings\n\n");
        for (i, point) in extraction.key_points.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, point));
        }
        report.push_str("\n");

        report.push_str("## 📚 Sources\n\n");
        for (i, source) in sources.iter().enumerate() {
            report.push_str(&format!("{}. **{}**  \n", i + 1, source.title));
            report.push_str(&format!("   {}  \n", source.url));
            report.push_str(&format!("   Words: {}\n\n", source.word_count));
        }

        report
    }

    fn generate_json(&self, topic: &str, extraction: &Extraction, sources: &[FetchedContent]) -> String {
        let json = serde_json::json!({
            "report": {
                "topic": topic,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "summary": extraction.summary,
                "key_points": extraction.key_points,
                "relevance_score": extraction.relevance,
                "word_count": extraction.word_count,
                "sources": sources.iter().map(|s| {
                    serde_json::json!({
                        "title": s.title,
                        "url": s.url,
                        "word_count": s.word_count,
                    })
                }).collect::<Vec<_>>(),
            }
        });
        serde_json::to_string_pretty(&json).unwrap_or_default()
    }

    fn generate_text(&self, topic: &str, extraction: &Extraction, sources: &[FetchedContent]) -> String {
        let mut report = String::new();
        report.push_str(&format!("=== RESEARCH REPORT: {} ===\n\n", topic.to_uppercase()));
        report.push_str(&format!("Sources: {}\n", sources.len()));
        report.push_str(&format!("Relevance: {:.1}%\n\n", extraction.relevance * 100.0));
        report.push_str("SUMMARY:\n");
        report.push_str(&extraction.summary);
        report.push_str("\n\nKEY FINDINGS:\n");
        for (i, point) in extraction.key_points.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, point));
        }
        report
    }
}
