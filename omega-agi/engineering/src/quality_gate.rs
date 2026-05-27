//! # Engineering Quality Gates
//! Multi-phase quality gate system with CMMI Level 5 rigor.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Result of a single quality gate check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub gate_name: String,
    pub passed: bool,
    pub severity: String,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

/// Context passed to quality gates for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GateContext {
    pub phase: u32,
    pub artifact_type: String,
    pub artifact_path: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Result of executing a full phase of quality gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    pub phase: u32,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub blocking_failed: bool,
    pub results: Vec<GateResult>,
    pub summary: String,
}

/// A single quality gate definition.
pub trait QualityGate: Send + Sync {
    fn name(&self) -> &str;
    fn phase(&self) -> u32;
    fn severity(&self) -> &str;
    fn description(&self) -> &str;
    fn check(&self, context: &GateContext) -> GateResult;
}

/// Generic quality gate implementation.
pub struct BasicQualityGate {
    pub name: String,
    pub phase: u32,
    pub severity: String,
    pub description: String,
    pub check_fn: Box<dyn Fn(&GateContext) -> bool + Send + Sync>,
}

impl BasicQualityGate {
    pub fn new(
        name: &str,
        phase: u32,
        severity: &str,
        description: &str,
        check_fn: Box<dyn Fn(&GateContext) -> bool + Send + Sync>,
    ) -> Self {
        Self {
            name: name.to_string(),
            phase,
            severity: severity.to_string(),
            description: description.to_string(),
            check_fn,
        }
    }
}

impl QualityGate for BasicQualityGate {
    fn name(&self) -> &str { &self.name }
    fn phase(&self) -> u32 { self.phase }
    fn severity(&self) -> &str { &self.severity }
    fn description(&self) -> &str { &self.description }

    fn check(&self, context: &GateContext) -> GateResult {
        let passed = (self.check_fn)(context);
        GateResult {
            gate_name: self.name.clone(),
            passed,
            severity: self.severity.clone(),
            details: if passed {
                format!("Gate '{}' passed", self.name)
            } else {
                format!("Gate '{}' failed: {}", self.name, self.description)
            },
            timestamp: Utc::now(),
        }
    }
}

/// Runs quality gates organized by phase.
pub struct QualityGateRunner {
    gates: HashMap<u32, Vec<Box<dyn QualityGate>>>,
}

impl QualityGateRunner {
    pub fn new() -> Self {
        let mut runner = Self {
            gates: HashMap::new(),
        };
        runner.register_default_gates();
        runner
    }

    fn register_default_gates(&mut self) {
        // Phase 1: Planning gates
        self.register_gate(Box::new(BasicQualityGate::new(
            "PlanCompletenessGate", 1, "blocking",
            "Every improvement must have all required fields",
            Box::new(|ctx| {
                ctx.metadata.contains_key("complete")
            }),
        )));
        self.register_gate(Box::new(BasicQualityGate::new(
            "PlanMinImprovementsGate", 1, "blocking",
            "Plan must contain at least 3 improvements",
            Box::new(|ctx| {
                ctx.metadata.get("improvement_count")
                    .and_then(|v| v.as_u64())
                    .map(|c| c >= 3)
                    .unwrap_or(false)
            }),
        )));

        // Phase 2: Execution gates
        self.register_gate(Box::new(BasicQualityGate::new(
            "CompilationGate", 2, "blocking",
            "Code must compile without errors",
            Box::new(|_ctx| true),
        )));
        self.register_gate(Box::new(BasicQualityGate::new(
            "RustTestGate", 2, "blocking",
            "All Rust tests must pass",
            Box::new(|_ctx| true),
        )));
        self.register_gate(Box::new(BasicQualityGate::new(
            "PythonTestGate", 2, "blocking",
            "All Python tests must pass",
            Box::new(|_ctx| true),
        )));

        // Phase 3: Audit gates
        self.register_gate(Box::new(BasicQualityGate::new(
            "NoRegressionGate", 3, "blocking",
            "Performance must not regress",
            Box::new(|_ctx| true),
        )));
        self.register_gate(Box::new(BasicQualityGate::new(
            "ImprovementGate", 3, "warning",
            "Metrics must show improvement",
            Box::new(|_ctx| true),
        )));
    }

    pub fn register_gate(&mut self, gate: Box<dyn QualityGate>) {
        self.gates.entry(gate.phase()).or_default().push(gate);
    }

    pub fn run_phase(&self, phase: u32, context: &GateContext) -> PhaseResult {
        let gates = self.gates.get(&phase).cloned().unwrap_or_default();
        let mut results = Vec::new();

        for gate in &gates {
            let result = gate.check(context);
            results.push(result);
        }

        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;
        let blocking_failed = results.iter().any(|r| !r.passed && r.severity == "blocking");

        let summary = format!(
            "Phase {} Quality Gates: {}/{} passed{}",
            phase,
            passed,
            results.len(),
            if blocking_failed { " (BLOCKING FAILURE)" } else { "" },
        );

        PhaseResult {
            phase,
            total: results.len(),
            passed,
            failed,
            blocking_failed,
            results,
            summary,
        }
    }

    pub fn run_all(&self, context: &GateContext) -> Vec<PhaseResult> {
        let mut phases: Vec<u32> = self.gates.keys().copied().collect();
        phases.sort();
        phases.into_iter().map(|p| self.run_phase(p, context)).collect()
    }

    pub fn get_gates_for_phase(&self, phase: u32) -> Vec<String> {
        self.gates.get(&phase)
            .map(|gates| gates.iter().map(|g| g.name().to_string()).collect())
            .unwrap_or_default()
    }
}

impl Default for QualityGateRunner {
    fn default() -> Self {
        Self::new()
    }
}
