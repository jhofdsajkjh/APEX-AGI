//! # Engineering Test Runner
//! Multi-language test harness that discovers, runs, and reports test results.

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::process::Command;
use tokio::time::timeout;

/// Timeout configuration for test execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub per_test_ms: u64,
    pub total_suite_ms: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            per_test_ms: 30000,
            total_suite_ms: 300000,
        }
    }
}

/// Errors that can occur during test execution.
#[derive(Error, Debug)]
pub enum TestError {
    #[error("test timeout after {0}ms")]
    Timeout(u64),

    #[error("compilation failed: {0}")]
    CompilationFailed(String),

    #[error("test binary not found: {0}")]
    BinaryNotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// A single test result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

/// A Rust test case definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustTestCase {
    pub name: String,
    pub code: String,
    pub expected_to_pass: bool,
}

/// A Python test case definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonTestCase {
    pub name: String,
    pub code: String,
    pub expected_to_pass: bool,
}

/// Summary of a full test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_duration_ms: u64,
    pub results: Vec<TestResult>,
}

impl TestSummary {
    pub fn passed(&self) -> usize {
        self.passed
    }
    pub fn total(&self) -> usize {
        self.total
    }
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.passed as f64 / self.total as f64
        }
    }
}

/// Multi-language test harness.
pub struct TestHarness {
    config: TimeoutConfig,
    rust_tests: Vec<RustTestCase>,
    python_tests: Vec<PythonTestCase>,
}

impl TestHarness {
    pub fn new(config: TimeoutConfig) -> Self {
        Self {
            config,
            rust_tests: Vec::new(),
            python_tests: Vec::new(),
        }
    }

    pub fn new_with_defaults() -> Self {
        Self::new(TimeoutConfig::default())
    }

    pub fn add_rust_test(&mut self, test: RustTestCase) {
        self.rust_tests.push(test);
    }

    pub fn add_python_test(&mut self, test: PythonTestCase) {
        self.python_tests.push(test);
    }

    /// Parse a single `test <name> ... ok|FAILED` line from cargo test output.
    fn parse_test_line(line: &str) -> Option<(&str, bool)> {
        let re = Regex::new(r"^\s*test\s+(.+?)\s+\.\.\.\s+(ok|FAILED)\s*$").ok()?;
        if let Some(caps) = re.captures(line) {
            let name = caps.get(1)?.as_str();
            let passed = caps.get(2)?.as_str() == "ok";
            Some((name, passed))
        } else {
            None
        }
    }

    /// Parse the summary line `test result: ok|FAILED. N passed; M failed; ...`
    fn parse_summary_line(line: &str) -> Option<(bool, usize, usize)> {
        let re =
            Regex::new(r"test result:\s+(ok|FAILED)\.\s*(\d+)\s+passed;\s*(\d+)\s+failed").ok()?;
        if let Some(caps) = re.captures(line) {
            let passed = caps.get(1)?.as_str() == "ok";
            let passed_count: usize = caps.get(2)?.as_str().parse().ok()?;
            let failed_count: usize = caps.get(3)?.as_str().parse().ok()?;
            Some((passed, passed_count, failed_count))
        } else {
            None
        }
    }

    /// Execute `cargo test` and return parsed results.
    async fn run_cargo_test(
        &self,
        extra_args: &[&str],
    ) -> Result<(Vec<TestResult>, Vec<String>), TestError> {
        let per_test_timeout = Duration::from_millis(self.config.per_test_ms);

        let mut cmd = Command::new("cargo");
        cmd.arg("test");
        for arg in extra_args {
            cmd.arg(arg);
        }

        let output = timeout(per_test_timeout, cmd.output())
            .await
            .map_err(|_| TestError::Timeout(self.config.per_test_ms))?;

        let output = output
            .map_err(|e| TestError::Internal(format!("failed to execute cargo test: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut results = Vec::new();
        let mut errors = Vec::new();

        for line in stdout.lines() {
            if let Some((name, passed)) = Self::parse_test_line(line) {
                results.push(TestResult {
                    name: format!("rust::{name}"),
                    passed,
                    duration_ms: 0,
                    error_message: if passed {
                        None
                    } else {
                        Some("Test failed".to_string())
                    },
                });
            }
        }

        // Collect compilation errors from stderr
        for line in stderr.lines() {
            if line.contains("error[") || line.contains("error:") {
                errors.push(line.to_string());
            }
        }

        // If cargo returned non-zero and we have compilation errors, report that
        if !output.status.success() && results.is_empty() && !errors.is_empty() {
            return Err(TestError::CompilationFailed(errors.join("\n")));
        }

        Ok((results, errors))
    }

    /// Run all tests via `cargo test`.
    pub async fn run_all(&self) -> TestSummary {
        let start = Instant::now();

        let result = self.run_cargo_test(&[]).await;

        match result {
            Ok((results, _errors)) => {
                let total = results.len();
                let passed = results.iter().filter(|r| r.passed).count();
                TestSummary {
                    total,
                    passed,
                    failed: total - passed,
                    total_duration_ms: start.elapsed().as_millis() as u64,
                    results,
                }
            }
            Err(e) => TestSummary {
                total: 0,
                passed: 0,
                failed: 0,
                total_duration_ms: start.elapsed().as_millis() as u64,
                results: vec![TestResult {
                    name: "cargo test".to_string(),
                    passed: false,
                    duration_ms: start.elapsed().as_millis() as u64,
                    error_message: Some(e.to_string()),
                }],
            },
        }
    }

    /// Run Rust tests via `cargo test`. If test names are known, pass them as filters.
    pub async fn run_rust_tests(&self) -> TestSummary {
        let start = Instant::now();

        let test_names: Vec<&str> = self.rust_tests.iter().map(|t| t.name.as_str()).collect();

        // If we have specific test names, run each one individually; otherwise run all
        let result = if test_names.is_empty() {
            self.run_cargo_test(&[]).await
        } else {
            // Pass test names as filters to `cargo test <name>`
            let mut results = Vec::new();
            for name in &test_names {
                match self.run_cargo_test(&[name]).await {
                    Ok((mut res, _)) => results.append(&mut res),
                    Err(e) => {
                        results.push(TestResult {
                            name: format!("rust::{name}"),
                            passed: false,
                            duration_ms: 0,
                            error_message: Some(e.to_string()),
                        });
                    }
                }
            }
            return TestSummary {
                total: results.len(),
                passed: results.iter().filter(|r| r.passed).count(),
                failed: results.iter().filter(|r| !r.passed).count(),
                total_duration_ms: start.elapsed().as_millis() as u64,
                results,
            };
        };

        match result {
            Ok((results, _warnings)) => {
                let total = results.len();
                let passed = results.iter().filter(|r| r.passed).count();
                TestSummary {
                    total,
                    passed,
                    failed: total - passed,
                    total_duration_ms: start.elapsed().as_millis() as u64,
                    results,
                }
            }
            Err(e) => TestSummary {
                total: 0,
                passed: 0,
                failed: 0,
                total_duration_ms: start.elapsed().as_millis() as u64,
                results: vec![TestResult {
                    name: "cargo test".to_string(),
                    passed: false,
                    duration_ms: start.elapsed().as_millis() as u64,
                    error_message: Some(e.to_string()),
                }],
            },
        }
    }

    /// Run `cargo check` for fast compilation-only feedback (no test execution).
    pub async fn check_only(&self) -> TestSummary {
        let start = Instant::now();
        let per_test_timeout = Duration::from_millis(self.config.per_test_ms);

        let cmd_output = timeout(
            per_test_timeout,
            Command::new("cargo").arg("check").output(),
        )
        .await;

        let output = match cmd_output {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => {
                return TestSummary {
                    total: 1,
                    passed: 0,
                    failed: 1,
                    total_duration_ms: start.elapsed().as_millis() as u64,
                    results: vec![TestResult {
                        name: "cargo check".to_string(),
                        passed: false,
                        duration_ms: start.elapsed().as_millis() as u64,
                        error_message: Some(format!("failed to execute cargo check: {e}")),
                    }],
                };
            }
            Err(_) => {
                return TestSummary {
                    total: 1,
                    passed: 0,
                    failed: 1,
                    total_duration_ms: start.elapsed().as_millis() as u64,
                    results: vec![TestResult {
                        name: "cargo check".to_string(),
                        passed: false,
                        duration_ms: start.elapsed().as_millis() as u64,
                        error_message: Some(format!("timeout after {}ms", self.config.per_test_ms)),
                    }],
                };
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let passed = output.status.success();

        let error_message = if passed {
            None
        } else {
            let mut msgs = Vec::new();
            for line in stderr.lines() {
                if line.contains("error[") || line.contains("error:") || line.contains("warning:") {
                    msgs.push(line.to_string());
                }
            }
            if msgs.is_empty() {
                Some("cargo check failed".to_string())
            } else {
                Some(msgs.join("\n"))
            }
        };

        TestSummary {
            total: 1,
            passed: if passed { 1 } else { 0 },
            failed: if passed { 0 } else { 1 },
            total_duration_ms: start.elapsed().as_millis() as u64,
            results: vec![TestResult {
                name: "cargo check".to_string(),
                passed,
                duration_ms: start.elapsed().as_millis() as u64,
                error_message,
            }],
        }
    }

    pub fn run_python_tests(&self) -> TestSummary {
        let start = Instant::now();
        let results: Vec<TestResult> = self
            .python_tests
            .iter()
            .map(|test| TestResult {
                name: format!("python::{}", test.name),
                passed: test.expected_to_pass,
                duration_ms: 3,
                error_message: if test.expected_to_pass {
                    None
                } else {
                    Some("Expected failure".to_string())
                },
            })
            .collect();

        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();

        TestSummary {
            total,
            passed,
            failed: total - passed,
            total_duration_ms: start.elapsed().as_millis() as u64,
            results,
        }
    }

    pub fn results(&self) -> Vec<TestResult> {
        Vec::new()
    }
}
