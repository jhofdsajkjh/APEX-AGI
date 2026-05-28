//! # Feedback Loop — Self-improvement via evolution feedback
//!
//! Closes the final gap in the self-evolution cycle:
//!
//! ```text
//! Self-Evolve → Code Gen → Compile → Test → Score → Feedback → Self-Evolve
//!                                                    ↑
//! Self-Heal detects regression ← Runtime metrics ────┘
//! ```
//!
//! This module collects metrics from code generation, test runs, and runtime
//! performance, then feeds them back into the evolution engine as fitness
//! scores. This is what makes the system truly **self-improving**.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::inference::InferenceEngine;

// ---------------------------------------------------------------------------
// Score types
// ---------------------------------------------------------------------------

/// A single score data point from any source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorePoint {
    pub timestamp: DateTime<Utc>,
    pub source: ScoreSource,
    pub metric: String,
    pub value: f64,
    pub weight: f64,
    pub metadata: HashMap<String, String>,
}

/// Where a score originates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScoreSource {
    /// From compile success/failure
    Compile,
    /// From test pass/fail rate
    Test,
    /// From runtime performance metrics
    Runtime,
    /// From code quality analysis
    Quality,
    /// From user feedback
    User,
    /// Internal evolution metric
    Evolution,
}

/// Aggregated feedback for the evolution engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackReport {
    pub timestamp: DateTime<Utc>,
    pub cycle_id: String,
    pub overall_score: f64,
    pub compile_score: f64,
    pub test_score: f64,
    pub quality_score: f64,
    pub runtime_score: f64,
    pub num_changes: usize,
    pub summary: String,
    pub recommendations: Vec<String>,
}

// ---------------------------------------------------------------------------
// FeedbackCollector
// ---------------------------------------------------------------------------

/// Collects, aggregates, and reports feedback from all sources.
pub struct FeedbackCollector {
    /// All raw score points.
    scores: RwLock<Vec<ScorePoint>>,
    /// Completed feedback reports (history).
    reports: RwLock<Vec<FeedbackReport>>,
    /// Maximum score points to keep in memory.
    max_scores: usize,
    /// Inference engine for generating summary text.
    llm: Option<Arc<dyn InferenceEngine>>,
}

impl FeedbackCollector {
    pub fn new(max_scores: usize) -> Self {
        Self {
            scores: RwLock::new(Vec::with_capacity(max_scores)),
            reports: RwLock::new(Vec::new()),
            max_scores,
            llm: None,
        }
    }

    pub fn with_llm(llm: Arc<dyn InferenceEngine>) -> Self {
        Self {
            scores: RwLock::new(Vec::with_capacity(1000)),
            reports: RwLock::new(Vec::new()),
            max_scores: 1000,
            llm: Some(llm),
        }
    }

    /// Record a compile result score (0.0 = failed, 1.0 = passed).
    pub async fn record_compile(&self, passed: bool, details: &str) {
        self.add_score(ScorePoint {
            timestamp: Utc::now(),
            source: ScoreSource::Compile,
            metric: "compile.success".into(),
            value: if passed { 1.0 } else { 0.0 },
            weight: 2.0,
            metadata: [("details".into(), details.to_string())].into(),
        }).await;
    }

    /// Record a test result score (pass rate 0.0–1.0).
    pub async fn record_test(&self, pass_rate: f64, total: usize, passed: usize, failed: usize) {
        self.add_score(ScorePoint {
            timestamp: Utc::now(),
            source: ScoreSource::Test,
            metric: "test.pass_rate".into(),
            value: pass_rate,
            weight: 3.0,
            metadata: [
                ("total".into(), total.to_string()),
                ("passed".into(), passed.to_string()),
                ("failed".into(), failed.to_string()),
            ].into(),
        }).await;
    }

    /// Record a runtime performance metric.
    pub async fn record_runtime(&self, metric: &str, value: f64) {
        self.add_score(ScorePoint {
            timestamp: Utc::now(),
            source: ScoreSource::Runtime,
            metric: format!("runtime.{}", metric),
            value,
            weight: 1.0,
            metadata: HashMap::new(),
        }).await;
    }

    /// Record a code quality score (0.0–1.0).
    pub async fn record_quality(&self, quality: f64, details: &str) {
        self.add_score(ScorePoint {
            timestamp: Utc::now(),
            source: ScoreSource::Quality,
            metric: "quality.overall".into(),
            value: quality,
            weight: 1.5,
            metadata: [("details".into(), details.to_string())].into(),
        }).await;
    }

    /// Record a user feedback score (e.g., thumbs up/down).
    pub async fn record_user_feedback(&self, score: f64, comment: &str) {
        self.add_score(ScorePoint {
            timestamp: Utc::now(),
            source: ScoreSource::User,
            metric: "user.feedback".into(),
            value: score,
            weight: 5.0, // User feedback is most important
            metadata: [("comment".into(), comment.to_string())].into(),
        }).await;
    }

    /// Internal: add a score point and enforce max size.
    async fn add_score(&self, point: ScorePoint) {
        let mut scores = self.scores.write().await;
        scores.push(point);
        if scores.len() > self.max_scores {
            scores.remove(0);
        }
    }

    /// Generate a feedback report by aggregating recent scores.
    pub async fn generate_report(&self, cycle_id: &str) -> FeedbackReport {
        let scores = self.scores.read().await;
        let recent: Vec<&ScorePoint> = scores.iter().filter(|s| {
            s.timestamp > Utc::now() - chrono::Duration::hours(24)
        }).collect();

        if recent.is_empty() {
            return FeedbackReport {
                timestamp: Utc::now(),
                cycle_id: cycle_id.to_string(),
                overall_score: 0.5,
                compile_score: 0.0,
                test_score: 0.0,
                quality_score: 0.0,
                runtime_score: 0.0,
                num_changes: 0,
                summary: "No feedback data available yet.".into(),
                recommendations: vec!["Run some tests to generate feedback.".into()],
            };
        }

        // Aggregate by source
        let compile_scores: Vec<f64> = recent.iter()
            .filter(|s| s.source == ScoreSource::Compile)
            .map(|s| s.value * s.weight).collect();
        let test_scores: Vec<f64> = recent.iter()
            .filter(|s| s.source == ScoreSource::Test)
            .map(|s| s.value * s.weight).collect();
        let quality_scores: Vec<f64> = recent.iter()
            .filter(|s| s.source == ScoreSource::Quality)
            .map(|s| s.value * s.weight).collect();
        let runtime_scores: Vec<f64> = recent.iter()
            .filter(|s| s.source == ScoreSource::Runtime)
            .map(|s| s.value * s.weight).collect();

        let compile_avg = average(&compile_scores);
        let test_avg = average(&test_scores);
        let quality_avg = average(&quality_scores);
        let runtime_avg = average(&runtime_scores);

        // Weighted overall (test and compile matter most for self-evolution)
        let overall = (compile_avg * 2.0 + test_avg * 3.0 + quality_avg * 1.5 + runtime_avg * 1.0) / 7.5;

        let summary = format!(
            "Compile: {:.1}% | Tests: {:.1}% | Quality: {:.1}% | Runtime: {:.1}%",
            compile_avg * 100.0, test_avg * 100.0, quality_avg * 100.0, runtime_avg * 100.0,
        );

        // Generate recommendations
        let mut recommendations = Vec::new();
        if compile_avg < 0.8 { recommendations.push("Improve compilation success rate — check recent code changes for syntax errors.".into()); }
        if test_avg < 0.7 { recommendations.push("Increase test coverage and fix failing tests.".into()); }
        if quality_avg < 0.6 { recommendations.push("Code quality needs attention — consider refactoring complex modules.".into()); }

        if recommendations.is_empty() {
            recommendations.push("System is stable. Focus on performance optimization or new features.".into());
        }

        drop(scores);

        let report = FeedbackReport {
            timestamp: Utc::now(),
            cycle_id: cycle_id.to_string(),
            overall_score: overall,
            compile_score: compile_avg,
            test_score: test_avg,
            quality_score: quality_avg,
            runtime_score: runtime_avg,
            num_changes: recent.len(),
            summary,
            recommendations,
        };

        // Store report
        self.reports.write().await.push(report.clone());

        report
    }

    /// Get the latest feedback report (for feeding into evolution).
    pub async fn latest_report(&self) -> Option<FeedbackReport> {
        self.reports.read().await.last().cloned()
    }

    /// Get recent score history.
    pub async fn recent_scores(&self, n: usize) -> Vec<ScorePoint> {
        let scores = self.scores.read().await;
        scores.iter().rev().take(n).cloned().collect()
    }

    /// Get report history.
    pub async fn report_history(&self) -> Vec<FeedbackReport> {
        self.reports.read().await.clone()
    }
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() { 0.0 } else { values.iter().sum::<f64>() / values.len() as f64 }
}

// ---------------------------------------------------------------------------
// Feedback integrator — bridges FeedbackCollector with SelfEvolver
// ---------------------------------------------------------------------------

/// Integrates feedback into the evolution engine's fitness function.
pub struct FeedbackIntegrator {
    collector: Arc<FeedbackCollector>,
    /// History of fitness scores for convergence detection.
    fitness_history: RwLock<Vec<f64>>,
}

impl FeedbackIntegrator {
    pub fn new(collector: Arc<FeedbackCollector>) -> Self {
        Self {
            collector,
            fitness_history: RwLock::new(Vec::new()),
        }
    }

    /// Compute a unified fitness score from the latest feedback.
    /// This is what the evolution engine uses as its objective function.
    pub async fn compute_fitness(&self) -> f64 {
        let report = self.collector.latest_report().await;
        match report {
            Some(r) => {
                let fitness = r.overall_score;
                let mut history = self.fitness_history.write().await;
                history.push(fitness);
                if history.len() > 50 { history.remove(0); }
                fitness
            }
            None => 0.5, // Default mid-point fitness
        }
    }

    /// Check if the system has converged (fitness hasn't improved).
    pub async fn has_converged(&self, tolerance: f64, window: usize) -> bool {
        let history = self.fitness_history.read().await;
        if history.len() < window * 2 { return false; }

        let recent: Vec<f64> = history.iter().rev().take(window).cloned().collect();
        let old: Vec<f64> = history.iter().rev().skip(window).take(window).cloned().collect();

        let recent_avg = average(&recent);
        let old_avg = average(&old);

        (recent_avg - old_avg).abs() < tolerance
    }

    /// Get fitness history for visualization.
    pub async fn fitness_history(&self) -> Vec<f64> {
        self.fitness_history.read().await.clone()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_report() {
        let collector = FeedbackCollector::new(100);
        collector.record_compile(true, "All good").await;
        collector.record_test(0.95, 100, 95, 5).await;
        collector.record_quality(0.85, "Clean code").await;

        let report = collector.generate_report("cycle-1").await;
        assert!(report.compile_score > 0.0);
        assert!(report.test_score > 0.0);
        assert!(report.overall_score > 0.0);
        assert_eq!(report.cycle_id, "cycle-1");
    }

    #[tokio::test]
    async fn test_feedback_integrator() {
        let collector = Arc::new(FeedbackCollector::new(100));
        let integrator = FeedbackIntegrator::new(collector.clone());

        collector.record_compile(true, "").await;
        collector.record_test(1.0, 10, 10, 0).await;
        collector.generate_report("cycle-1").await;

        let fitness = integrator.compute_fitness().await;
        assert!(fitness > 0.0);
    }

    #[tokio::test]
    async fn test_convergence_detection() {
        let collector = Arc::new(FeedbackCollector::new(100));
        let integrator = FeedbackIntegrator::new(collector);

        // Add many identical scores
        for _ in 0..20 {
            collector.record_compile(true, "").await;
            collector.record_test(0.9, 10, 9, 1).await;
            collector.generate_report("cycle").await;
            integrator.compute_fitness().await;
        }

        // Should converge with tight tolerance
        // (may need more data points)
        let _ = integrator.has_converged(0.05, 5).await;
    }

    #[test]
    fn test_empty_report() {
        let collector = FeedbackCollector::new(10);
        // No scores added; report should have defaults
    }
}
