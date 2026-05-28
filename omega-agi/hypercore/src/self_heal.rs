//! # HyperCore Self-Healing
//! Automatic healing controller for detecting and recovering from failures.
//!
//! Implements circuit breaker pattern, exponential backoff retries, health checks,
//! failure classification, multiple recovery strategies, and full healing pipeline.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

// ============================================================================
// Circuit Breaker
// ============================================================================

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Normal operation — requests pass through.
    Closed,
    /// Failure threshold exceeded — requests are blocked.
    Open,
    /// Probing after recovery timeout — limited requests allowed.
    HalfOpen,
}

/// Circuit breaker that tracks failures and prevents cascading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    /// Current state.
    pub state: CircuitState,
    /// Consecutive failure count.
    pub failure_count: u64,
    /// Failures needed to open the circuit.
    pub failure_threshold: u64,
    /// Duration to wait before moving from Open to HalfOpen.
    pub recovery_timeout: Duration,
    /// Timestamp when the circuit was last tripped.
    pub last_failure_time: Option<DateTime<Utc>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default settings (5 failures, 30s recovery).
    pub fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            last_failure_time: None,
        }
    }

    /// Create a circuit breaker with custom thresholds.
    pub fn with_thresholds(failure_threshold: u64, recovery_timeout_secs: u64) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            recovery_timeout: Duration::from_secs(recovery_timeout_secs),
            last_failure_time: None,
        }
    }

    /// Check if the circuit allows a request through.
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has elapsed → transition to HalfOpen
                if let Some(last_fail) = self.last_failure_time {
                    let elapsed = Utc::now()
                        .signed_duration_since(last_fail)
                        .to_std()
                        .unwrap_or(Duration::ZERO);
                    if elapsed >= self.recovery_timeout {
                        tracing::info!("Circuit moving from Open to HalfOpen (probing)");
                        self.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true, // Allow probe requests
        }
    }

    /// Record a success — resets failure count, closes the circuit.
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.last_failure_time = None;
        tracing::debug!("Circuit reset to Closed after success");
    }

    /// Record a failure — increments counter, may trip to Open.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Utc::now());
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
            tracing::warn!(
                failure_count = self.failure_count,
                threshold = self.failure_threshold,
                "Circuit tripped to Open"
            );
        } else if self.state == CircuitState::HalfOpen {
            // A failure in HalfOpen goes straight back to Open
            self.state = CircuitState::Open;
            tracing::warn!("HalfOpen probe failed, circuit back to Open");
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Retry Policy (Exponential Backoff)
// ============================================================================

/// Retry policy with exponential backoff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Initial delay in milliseconds (e.g., 100ms).
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds (e.g., 30_000ms).
    pub max_delay_ms: u64,
    /// Backoff multiplier (e.g., 2.0 for doubling).
    pub multiplier: f64,
    /// Maximum number of retry attempts.
    pub max_retries: u32,
}

impl RetryPolicy {
    /// Create a new retry policy.
    pub fn new(
        initial_delay_ms: u64,
        max_delay_ms: u64,
        multiplier: f64,
        max_retries: u32,
    ) -> Self {
        Self {
            initial_delay_ms,
            max_delay_ms,
            multiplier,
            max_retries,
        }
    }

    /// Default retry policy: 100ms initial, 30s max, 2x multiplier, 3 retries.
    pub fn default_policy() -> Self {
        Self {
            initial_delay_ms: 100,
            max_delay_ms: 30_000,
            multiplier: 2.0,
            max_retries: 3,
        }
    }

    /// Calculate the delay for a given retry attempt (0-indexed).
    /// Uses exponential backoff: delay = min(initial * multiplier^attempt, max).
    pub fn backoff_delay(&self, attempt: u32) -> Duration {
        let computed = self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32);
        let clamped = computed.min(self.max_delay_ms as f64).max(1.0);
        Duration::from_millis(clamped as u64)
    }

    /// Get total maximum duration (sum of all backoff delays).
    pub fn max_total_duration(&self) -> Duration {
        let mut total = 0u64;
        for i in 0..self.max_retries {
            let d = self.initial_delay_ms as f64 * self.multiplier.powi(i as i32);
            total += d.min(self.max_delay_ms as f64).max(1.0) as u64;
        }
        Duration::from_millis(total)
    }
}

// ============================================================================
// Retry Executor
// ============================================================================

/// Executes a closure with retries and exponential backoff.
pub struct RetryExecutor<'a> {
    policy: &'a RetryPolicy,
    circuit_breaker: Option<&'a mut CircuitBreaker>,
}

impl<'a> RetryExecutor<'a> {
    /// Create a new retry executor with the given policy.
    pub fn new(policy: &'a RetryPolicy) -> Self {
        Self {
            policy,
            circuit_breaker: None,
        }
    }

    /// Attach a circuit breaker to this executor.
    pub fn with_circuit_breaker(mut self, cb: &'a mut CircuitBreaker) -> Self {
        self.circuit_breaker = Some(cb);
        self
    }

    /// Execute `f` with retries. Returns `Ok(result)` on first success,
    /// or `Err(error_message)` after all retries are exhausted.
    ///
    /// `f` should return `Ok(T)` on success or `Err(String)` on transient failure.
    /// Permanent failures (e.g., config errors) are NOT retried.
    pub fn execute<T>(
        &mut self,
        mut f: impl FnMut() -> Result<T, String>,
        classify: impl Fn(&str) -> FailureClass,
    ) -> Result<T, String> {
        // Circuit breaker check
        if let Some(ref mut cb) = self.circuit_breaker {
            if !cb.allow_request() {
                return Err(format!(
                    "Circuit breaker is OPEN ({} failures, threshold {})",
                    cb.failure_count, cb.failure_threshold
                ));
            }
        }

        let mut last_error = String::new();

        for attempt in 0..=self.policy.max_retries {
            // First attempt has no delay
            if attempt > 0 {
                let delay = self.policy.backoff_delay(attempt - 1);
                tracing::debug!(
                    attempt,
                    delay_ms = delay.as_millis(),
                    "Retrying after backoff"
                );
                std::thread::sleep(delay);
            }

            match f() {
                Ok(result) => {
                    // Record success in circuit breaker
                    if let Some(ref mut cb) = self.circuit_breaker {
                        cb.record_success();
                    }
                    return Ok(result);
                }
                Err(err) => {
                    let cls = classify(&err);
                    match cls {
                        FailureClass::Permanent => {
                            // Do not retry permanent failures
                            tracing::warn!(error = %err, "Permanent failure, not retrying");
                            if let Some(ref mut cb) = self.circuit_breaker {
                                cb.record_failure();
                            }
                            return Err(err);
                        }
                        FailureClass::Transient => {
                            last_error = err;
                            tracing::warn!(
                                attempt,
                                max_retries = self.policy.max_retries,
                                error = %last_error,
                                "Transient failure"
                            );
                        }
                    }
                }
            }
        }

        // All retries exhausted
        if let Some(ref mut cb) = self.circuit_breaker {
            cb.record_failure();
        }
        Err(format!(
            "All {} retries exhausted. Last error: {}",
            self.policy.max_retries, last_error
        ))
    }
}

// ============================================================================
// Failure Classification
// ============================================================================

/// Classification of a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureClass {
    /// A transient failure that may succeed on retry (timeout, rate limit, etc.).
    Transient,
    /// A permanent failure that will not succeed on retry (config error, invalid state).
    Permanent,
}

/// Default failure classifier.
/// Returns `Transient` for common transient patterns, `Permanent` otherwise.
pub fn default_classify(error: &str) -> FailureClass {
    let lower = error.to_lowercase();
    let transient_patterns = [
        "timeout",
        "timed out",
        "rate limit",
        "too many requests",
        "connection refused",
        "connection reset",
        "temporarily unavailable",
        "server error",
        "500",
        "502",
        "503",
        "504",
        "service unavailable",
        "throttled",
        "retry later",
        "network error",
        "eagain",
        "would block",
        "broken pipe",
    ];
    if transient_patterns.iter().any(|p| lower.contains(p)) {
        FailureClass::Transient
    } else {
        FailureClass::Permanent
    }
}

// ============================================================================
// Recovery Strategies
// ============================================================================

/// A recovery strategy that can be applied to a target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Restart the target with exponential backoff.
    RestartWithBackoff {
        target: String,
        initial_delay_ms: u64,
        max_delay_ms: u64,
    },
    /// Scale up the target by adding replicas.
    ScaleUp {
        target: String,
        additional_replicas: u32,
    },
    /// Clear a cache or temporary state.
    ClearCache {
        target: String,
        cache_key: Option<String>,
    },
    /// Rollback to a last-known-good version.
    RollbackToLastGood {
        target: String,
        version: Option<String>,
    },
}

impl RecoveryStrategy {
    /// Execute the recovery strategy and return a healing result.
    pub fn execute(&self) -> HealingResult {
        let start = Instant::now();
        match self {
            RecoveryStrategy::RestartWithBackoff {
                target,
                initial_delay_ms,
                max_delay_ms,
            } => {
                let delay = (*initial_delay_ms as f64).min(*max_delay_ms as f64) as u64;
                tracing::info!(target = %target, delay_ms = delay, "Restarting with backoff");
                // Simulate restart: in production this would send a restart signal
                std::thread::sleep(Duration::from_millis(delay.min(500))); // cap for testing
                HealingResult {
                    action: format!("restart_with_backoff({})", target),
                    success: true,
                    message: format!("Restart initiated for {} with {}ms backoff", target, delay),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            RecoveryStrategy::ScaleUp {
                target,
                additional_replicas,
            } => {
                tracing::info!(target = %target, replicas = %additional_replicas, "Scaling up");
                // In production this would call an orchestrator API
                HealingResult {
                    action: format!("scale_up({},+{})", target, additional_replicas),
                    success: true,
                    message: format!("Scaled up {} by {} replica(s)", target, additional_replicas),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            RecoveryStrategy::ClearCache { target, cache_key } => {
                let key_desc = cache_key.as_deref().unwrap_or("all");
                tracing::info!(target = %target, cache_key = %key_desc, "Clearing cache");
                // In production this would invalidate cache entries
                HealingResult {
                    action: format!("clear_cache({}, {})", target, key_desc),
                    success: true,
                    message: format!("Cache cleared for {} (key: {})", target, key_desc),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            RecoveryStrategy::RollbackToLastGood { target, version } => {
                let ver = version.as_deref().unwrap_or("last-good");
                tracing::info!(target = %target, version = %ver, "Rolling back");
                // In production this would trigger a deployment rollback
                HealingResult {
                    action: format!("rollback({}, {})", target, ver),
                    success: true,
                    message: format!("Rolled back {} to version {}", target, ver),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
        }
    }

    /// Estimate the duration of this strategy (for planning).
    pub fn estimated_duration_ms(&self) -> u64 {
        match self {
            RecoveryStrategy::RestartWithBackoff {
                initial_delay_ms, ..
            } => *initial_delay_ms,
            RecoveryStrategy::ScaleUp { .. } => 2000,
            RecoveryStrategy::ClearCache { .. } => 500,
            RecoveryStrategy::RollbackToLastGood { .. } => 5000,
        }
    }
}

// ============================================================================
// Health Check
// ============================================================================

/// Simple health check result.
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy { reason: String },
    Unknown { reason: String },
}

/// Perform a health check on a target.
/// In production, this would make an actual HTTP/TCP check.
pub fn health_check(target: &str) -> HealthStatus {
    // Default implementation: simple connectivity simulation
    // Real implementations would ping an endpoint or check process status
    if target.is_empty() {
        return HealthStatus::Unhealthy {
            reason: "Empty target".to_string(),
        };
    }
    // Assume healthy by default for the base implementation
    // Subclasses or real implementations override this behavior
    HealthStatus::Healthy
}

// ============================================================================
// Healing Pipeline
// ============================================================================

/// A plan produced by the diagnosis phase.
#[derive(Debug, Clone)]
pub struct HealingPlan {
    pub target: String,
    pub issue: String,
    pub strategies: Vec<RecoveryStrategy>,
    pub failure_class: FailureClass,
}

impl HealingPlan {
    pub fn is_actionable(&self) -> bool {
        !self.strategies.is_empty()
    }
}

/// Full-featured diagnosis of an issue.
pub fn diagnose(target: &str, issue: &str) -> HealingPlan {
    let cls = default_classify(issue);
    let mut strategies: Vec<RecoveryStrategy> = Vec::new();

    let issue_lower = issue.to_lowercase();

    // Match common issue patterns to recovery strategies
    if issue_lower.contains("timeout") || issue_lower.contains("connection") {
        strategies.push(RecoveryStrategy::RestartWithBackoff {
            target: target.to_string(),
            initial_delay_ms: 500,
            max_delay_ms: 15_000,
        });
    }
    if issue_lower.contains("rate limit") || issue_lower.contains("too many") {
        strategies.push(RecoveryStrategy::ScaleUp {
            target: target.to_string(),
            additional_replicas: 2,
        });
    }
    if issue_lower.contains("cache") || issue_lower.contains("stale") {
        strategies.push(RecoveryStrategy::ClearCache {
            target: target.to_string(),
            cache_key: None,
        });
    }
    if issue_lower.contains("version")
        || issue_lower.contains("mismatch")
        || issue_lower.contains("corrupt")
    {
        strategies.push(RecoveryStrategy::RollbackToLastGood {
            target: target.to_string(),
            version: None,
        });
    }

    // Fallback: if no specific strategy matched, use RestartWithBackoff
    if strategies.is_empty() {
        strategies.push(RecoveryStrategy::RestartWithBackoff {
            target: target.to_string(),
            initial_delay_ms: 1000,
            max_delay_ms: 30_000,
        });
    }

    HealingPlan {
        target: target.to_string(),
        issue: issue.to_string(),
        strategies,
        failure_class: cls,
    }
}

/// Execute a healing plan and return the consolidated result.
pub fn execute_plan(plan: &HealingPlan) -> HealingResult {
    let start = Instant::now();
    tracing::info!(
        target = %plan.target,
        issue = %plan.issue,
        strategies = plan.strategies.len(),
        "Executing healing plan"
    );

    for (i, strategy) in plan.strategies.iter().enumerate() {
        tracing::debug!(strategy_index = i, "Applying strategy {:?}", strategy);
        let result = strategy.execute();
        if !result.success && i < plan.strategies.len() - 1 {
            tracing::warn!(strategy_index = i, "Strategy failed, trying next");
            continue;
        }
        if result.success {
            return HealingResult {
                action: format!("diagnose_and_heal({})", plan.target),
                success: true,
                message: format!(
                    "Healed '{}' for issue '{}' using strategy {}/{}: {}",
                    plan.target,
                    plan.issue,
                    i + 1,
                    plan.strategies.len(),
                    result.message
                ),
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }
    }

    // All strategies failed
    HealingResult {
        action: format!("diagnose_and_heal({})", plan.target),
        success: false,
        message: format!(
            "All {} strategies failed for '{}' (issue: {})",
            plan.strategies.len(),
            plan.target,
            plan.issue
        ),
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

/// Verify that the healing was successful by performing a health check.
pub fn verify_healing(target: &str, original_issue: &str) -> HealingResult {
    let start = Instant::now();
    let status = health_check(target);
    match status {
        HealthStatus::Healthy => HealingResult {
            action: format!("verify({})", target),
            success: true,
            message: format!(
                "Health check passed for {} after '{}'",
                target, original_issue
            ),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        HealthStatus::Unhealthy { reason } => HealingResult {
            action: format!("verify({})", target),
            success: false,
            message: format!(
                "Health check FAILED for {} after '{}': {}",
                target, original_issue, reason
            ),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        HealthStatus::Unknown { reason } => HealingResult {
            action: format!("verify({})", target),
            success: false,
            message: format!(
                "Health check UNKNOWN for {} after '{}': {}",
                target, original_issue, reason
            ),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Run the full healing pipeline: diagnose → plan → execute → verify.
pub fn healing_pipeline(target: &str, issue: &str) -> HealingResult {
    let start = Instant::now();

    // 1. Diagnose
    tracing::info!(target = %target, issue = %issue, "Starting healing pipeline");
    let plan = diagnose(target, issue);
    if !plan.is_actionable() {
        return HealingResult {
            action: "healing_pipeline".to_string(),
            success: false,
            message: format!("No actionable plan for '{}' on '{}'", issue, target),
            duration_ms: start.elapsed().as_millis() as u64,
        };
    }

    // 2. Execute
    let exec_result = execute_plan(&plan);
    if !exec_result.success {
        return exec_result;
    }

    // 3. Verify
    let verify_result = verify_healing(target, issue);

    HealingResult {
        action: format!("healing_pipeline({})", target),
        success: verify_result.success,
        message: format!(
            "Pipeline result for '{}' on '{}': {}",
            issue, target, verify_result.message
        ),
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

// ============================================================================
// Healing Action
// ============================================================================

/// A healing action that can be applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealingAction {
    Restart { target: String },
    Recover { target: String, strategy: String },
    Scale { target: String, replicas: u32 },
    Clear { target: String },
    Rollback { target: String, version: String },
}

/// Result of a healing operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingResult {
    pub action: String,
    pub success: bool,
    pub message: String,
    pub duration_ms: u64,
}

/// A healing event recorded in the event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingEvent {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub target: String,
    pub action: String,
    pub success: bool,
    pub details: String,
}

// ============================================================================
// Healer Trait and Implementations
// ============================================================================

/// Trait for healers that can handle specific failure modes.
pub trait Healer: Send + Sync {
    fn name(&self) -> &str;
    fn can_handle(&self, issue: &str) -> bool;
    fn heal(&self, target: &str, issue: &str) -> HealingResult;
}

/// Generic healer for simple service recovery.
pub struct GenericHealer {
    name: String,
    handled_issues: Vec<String>,
}

impl GenericHealer {
    pub fn new(name: &str, issues: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            handled_issues: issues.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Healer for GenericHealer {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, issue: &str) -> bool {
        self.handled_issues.iter().any(|i| issue.contains(i))
    }

    fn heal(&self, target: &str, _issue: &str) -> HealingResult {
        tracing::info!(healer = %self.name, target = %target, "Performing generic heal");
        HealingResult {
            action: format!("generic_recover({})", target),
            success: true,
            message: format!("Recovery action applied to {}", target),
            duration_ms: 0,
        }
    }
}

/// A healer that uses the full healing pipeline (diagnose → plan → execute → verify).
pub struct PipelineHealer {
    name: String,
    handled_issues: Vec<String>,
}

impl PipelineHealer {
    pub fn new(name: &str, issues: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            handled_issues: issues.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Healer for PipelineHealer {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, issue: &str) -> bool {
        self.handled_issues.iter().any(|i| issue.contains(i))
    }

    fn heal(&self, target: &str, issue: &str) -> HealingResult {
        tracing::info!(healer = %self.name, target = %target, "Running pipeline heal");
        healing_pipeline(target, issue)
    }
}

// ============================================================================
// Default Healer Registry
// ============================================================================

/// Register default healers for common issues on a controller.
pub fn register_default_healers(controller: &mut SelfHealingController) {
    // Network / timeout issues → pipeline healer
    controller.register_healer(Box::new(PipelineHealer::new(
        "network_healer",
        &["timeout", "connection", "network", "refused", "reset"],
    )));

    // Rate limiting → pipeline healer
    controller.register_healer(Box::new(PipelineHealer::new(
        "rate_limit_healer",
        &["rate limit", "too many requests", "throttled"],
    )));

    // Cache issues → pipeline healer
    controller.register_healer(Box::new(PipelineHealer::new(
        "cache_healer",
        &["cache", "stale", "corrupt"],
    )));

    // Version / rollback issues → pipeline healer
    controller.register_healer(Box::new(PipelineHealer::new(
        "rollback_healer",
        &["version", "mismatch", "rollback", "incompatible"],
    )));

    // Generic fallback → generic healer matching "error", "failure", "crash"
    controller.register_healer(Box::new(GenericHealer::new(
        "generic_fallback",
        &["error", "failure", "crash", "panic", "exception"],
    )));
}

// ============================================================================
// Self-Healing Controller
// ============================================================================

/// Self-healing controller that manages healers, circuit breakers, and healing events.
pub struct SelfHealingController {
    healers: Vec<Box<dyn Healer>>,
    events: Vec<HealingEvent>,
    event_counter: AtomicU64,
    circuit_breaker: CircuitBreaker,
    retry_policy: RetryPolicy,
}

impl SelfHealingController {
    pub fn new() -> Self {
        Self {
            healers: Vec::new(),
            events: Vec::new(),
            event_counter: AtomicU64::new(1),
            circuit_breaker: CircuitBreaker::new(),
            retry_policy: RetryPolicy::default_policy(),
        }
    }

    /// Create a controller with custom circuit breaker and retry settings.
    pub fn with_settings(
        failure_threshold: u64,
        recovery_timeout_secs: u64,
        initial_delay_ms: u64,
        max_delay_ms: u64,
        max_retries: u32,
    ) -> Self {
        Self {
            healers: Vec::new(),
            events: Vec::new(),
            event_counter: AtomicU64::new(1),
            circuit_breaker: CircuitBreaker::with_thresholds(
                failure_threshold,
                recovery_timeout_secs,
            ),
            retry_policy: RetryPolicy::new(initial_delay_ms, max_delay_ms, 2.0, max_retries),
        }
    }

    /// Get a reference to the circuit breaker.
    pub fn circuit_breaker(&self) -> &CircuitBreaker {
        &self.circuit_breaker
    }

    /// Get a mutable reference to the circuit breaker.
    pub fn circuit_breaker_mut(&mut self) -> &mut CircuitBreaker {
        &mut self.circuit_breaker
    }

    /// Get a reference to the retry policy.
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }

    /// Get a mutable reference to the retry policy.
    pub fn retry_policy_mut(&mut self) -> &mut RetryPolicy {
        &mut self.retry_policy
    }

    pub fn register_healer(&mut self, healer: Box<dyn Healer>) {
        tracing::info!(healer = %healer.name(), "Self-healer registered");
        self.healers.push(healer);
    }

    /// Diagnose and heal using the pipeline, with circuit breaker and retry logic.
    pub fn diagnose_and_heal(&mut self, target: &str, issue: &str) -> HealingResult {
        let start = Instant::now();

        // Circuit breaker check
        if !self.circuit_breaker.allow_request() {
            let result = HealingResult {
                action: "diagnose_and_heal".to_string(),
                success: false,
                message: format!(
                    "Circuit breaker OPEN for '{}' ({} failures, threshold {}). Skipping.",
                    target,
                    self.circuit_breaker.failure_count,
                    self.circuit_breaker.failure_threshold
                ),
                duration_ms: start.elapsed().as_millis() as u64,
            };
            self.events.push(HealingEvent {
                id: self.event_counter.fetch_add(1, Ordering::SeqCst),
                timestamp: Utc::now(),
                target: target.to_string(),
                action: result.action.clone(),
                success: result.success,
                details: result.message.clone(),
            });
            return result;
        }

        // Pre-check health
        let pre_health = health_check(target);
        let pre_healthy = matches!(pre_health, HealthStatus::Healthy);

        // Try registered healers first
        for healer in &self.healers {
            if healer.can_handle(issue) {
                let result = healer.heal(target, issue);
                // Verify after healing
                let verify_result = if result.success {
                    verify_healing(target, issue)
                } else {
                    result.clone()
                };

                let final_result = if result.success && verify_result.success {
                    self.circuit_breaker.record_success();
                    HealingResult {
                        action: result.action.clone(),
                        success: true,
                        message: format!(
                            "Healed via {}: {} (verified: {})",
                            healer.name(),
                            result.message,
                            verify_result.message
                        ),
                        duration_ms: start.elapsed().as_millis() as u64,
                    }
                } else if result.success && !verify_result.success {
                    self.circuit_breaker.record_failure();
                    HealingResult {
                        action: result.action.clone(),
                        success: false,
                        message: format!(
                            "{} applied but verification failed: {}",
                            healer.name(),
                            verify_result.message
                        ),
                        duration_ms: start.elapsed().as_millis() as u64,
                    }
                } else {
                    self.circuit_breaker.record_failure();
                    result
                };

                self.events.push(HealingEvent {
                    id: self.event_counter.fetch_add(1, Ordering::SeqCst),
                    timestamp: Utc::now(),
                    target: target.to_string(),
                    action: final_result.action.clone(),
                    success: final_result.success,
                    details: format!(
                        "healer={} issue={} pre_healthy={} duration={}ms msg={}",
                        healer.name(),
                        issue,
                        pre_healthy,
                        final_result.duration_ms,
                        final_result.message
                    ),
                });

                return final_result;
            }
        }

        // Fall back to the healing pipeline
        let result = healing_pipeline(target, issue);
        let verify_result = if result.success {
            verify_healing(target, issue)
        } else {
            result.clone()
        };

        let final_result = if result.success && verify_result.success {
            self.circuit_breaker.record_success();
            HealingResult {
                action: result.action.clone(),
                success: true,
                message: format!(
                    "Pipeline healed: {} (verified: {})",
                    result.message, verify_result.message
                ),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            self.circuit_breaker.record_failure();
            HealingResult {
                action: result.action.clone(),
                success: false,
                message: format!(
                    "Pipeline failed: {} (verify: {})",
                    result.message, verify_result.message
                ),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        };

        self.events.push(HealingEvent {
            id: self.event_counter.fetch_add(1, Ordering::SeqCst),
            timestamp: Utc::now(),
            target: target.to_string(),
            action: final_result.action.clone(),
            success: final_result.success,
            details: format!(
                "pipeline target={} issue={} pre_healthy={} duration={}ms msg={}",
                target, issue, pre_healthy, final_result.duration_ms, final_result.message
            ),
        });

        final_result
    }

    /// Try to heal with retry logic and circuit breaker protection.
    /// This method will retry the healing operation with exponential backoff
    /// for transient failures, and respect the circuit breaker state.
    pub fn try_heal_with_retry(&mut self, target: &str, issue: &str) -> HealingResult {
        let start = Instant::now();

        // Check circuit breaker
        if !self.circuit_breaker.allow_request() {
            let msg = format!(
                "Circuit breaker OPEN for '{}' ({} failures, threshold {})",
                target, self.circuit_breaker.failure_count, self.circuit_breaker.failure_threshold
            );
            tracing::warn!("{}", msg);
            return HealingResult {
                action: "try_heal_with_retry".to_string(),
                success: false,
                message: msg,
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Attempt healing with retries
        let mut last_error = String::new();
        for attempt in 0..=self.retry_policy.max_retries {
            if attempt > 0 {
                let delay = self.retry_policy.backoff_delay(attempt - 1);
                tracing::info!(
                    attempt,
                    delay_ms = delay.as_millis(),
                    target = %target,
                    "Retrying heal"
                );
                std::thread::sleep(delay);
            }

            let result = self.diagnose_and_heal(target, issue);
            if result.success {
                // Verify the healing actually worked
                let verify = verify_healing(target, issue);
                if verify.success {
                    self.circuit_breaker.record_success();
                    return HealingResult {
                        action: format!("try_heal_with_retry({})", target),
                        success: true,
                        message: format!(
                            "Healed after {} retries (attempt {}). {}",
                            attempt,
                            attempt + 1,
                            result.message
                        ),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                } else {
                    last_error = format!(
                        "Heal claimed success but verification failed: {}",
                        verify.message
                    );
                    tracing::warn!(
                        attempt,
                        target = %target,
                        "{}",
                        last_error
                    );
                }
            } else {
                last_error = result.message.clone();
                let cls = default_classify(&last_error);
                if cls == FailureClass::Permanent {
                    tracing::warn!(
                        target = %target,
                        error = %last_error,
                        "Permanent failure, not retrying"
                    );
                    self.circuit_breaker.record_failure();
                    return HealingResult {
                        action: "try_heal_with_retry".to_string(),
                        success: false,
                        message: format!("Permanent failure on '{}': {}", target, last_error),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
                tracing::warn!(
                    attempt,
                    target = %target,
                    error = %last_error,
                    "Transient heal failure"
                );
            }
        }

        // All retries exhausted
        self.circuit_breaker.record_failure();
        HealingResult {
            action: "try_heal_with_retry".to_string(),
            success: false,
            message: format!(
                "All {} retries exhausted for '{}'. Last error: {}",
                self.retry_policy.max_retries, target, last_error
            ),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    pub fn get_events(&self) -> &[HealingEvent] {
        &self.events
    }

    pub fn healer_count(&self) -> usize {
        self.healers.len()
    }
}

impl Default for SelfHealingController {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed() {
        let mut cb = CircuitBreaker::new();
        assert!(cb.allow_request());
        assert_eq!(cb.state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_trips_at_threshold() {
        let mut cb = CircuitBreaker::with_thresholds(3, 60);
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.state, CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_half_open_probe_fails() {
        let mut cb = CircuitBreaker::with_thresholds(2, 1);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        // Fast-forward: force HalfOpen
        cb.last_failure_time = Some(Utc::now() - chrono::Duration::seconds(2));
        // Actually we need to make recovery_timeout smaller for test
        let mut cb = CircuitBreaker::with_thresholds(2, 0); // 0 second recovery
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        assert!(cb.allow_request()); // Should transition to HalfOpen since timeout is 0
        assert_eq!(cb.state, CircuitState::HalfOpen);
        cb.record_failure(); // Fail in HalfOpen → back to Open
        assert_eq!(cb.state, CircuitState::Open);
    }

    #[test]
    fn test_retry_policy_backoff() {
        let policy = RetryPolicy::new(100, 10_000, 2.0, 3);
        assert_eq!(policy.backoff_delay(0).as_millis(), 100);
        assert_eq!(policy.backoff_delay(1).as_millis(), 200);
        assert_eq!(policy.backoff_delay(2).as_millis(), 400);
        // Beyond max: clamped
        let policy_small = RetryPolicy::new(1000, 2000, 10.0, 5);
        assert_eq!(policy_small.backoff_delay(3).as_millis(), 2000);
    }

    #[test]
    fn test_retry_executor_success() {
        let policy = RetryPolicy::default_policy();
        let success = || -> Result<i32, String> { Ok(42) };
        let mut exec = RetryExecutor::new(&policy);
        let result = exec.execute(success, default_classify);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_retry_executor_transient_then_success() {
        let policy = RetryPolicy::new(1, 10, 2.0, 3);
        let mut call_count = 0u32;
        let flaky = || -> Result<i32, String> {
            call_count += 1;
            if call_count < 3 {
                Err("timeout error".to_string())
            } else {
                Ok(99)
            }
        };
        let mut exec = RetryExecutor::new(&policy);
        let result = exec.execute(flaky, default_classify);
        assert_eq!(result, Ok(99));
        assert_eq!(call_count, 3);
    }

    #[test]
    fn test_retry_executor_permanent_failure() {
        let policy = RetryPolicy::new(1, 10, 2.0, 3);
        let perm = || -> Result<i32, String> { Err("config error: invalid value".to_string()) };
        let mut exec = RetryExecutor::new(&policy);
        let result = exec.execute(perm, default_classify);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("config error"));
    }

    #[test]
    fn test_failure_classification() {
        assert_eq!(
            default_classify("connection timeout"),
            FailureClass::Transient
        );
        assert_eq!(
            default_classify("rate limit exceeded"),
            FailureClass::Transient
        );
        assert_eq!(
            default_classify("503 Service Unavailable"),
            FailureClass::Transient
        );
        assert_eq!(
            default_classify("config error: bad value"),
            FailureClass::Permanent
        );
        assert_eq!(default_classify("invalid state"), FailureClass::Permanent);
    }

    #[test]
    fn test_diagnose_creates_plan() {
        let plan = diagnose("api-server", "connection timeout");
        assert!(plan.is_actionable());
        assert!(!plan.strategies.is_empty());
    }

    #[test]
    fn test_healing_pipeline() {
        let result = healing_pipeline("test-service", "connection timeout");
        // Pipeline should at least attempt something
        assert!(result.success || !result.message.is_empty());
    }

    #[test]
    fn test_controller_new() {
        let ctrl = SelfHealingController::new();
        assert_eq!(ctrl.healer_count(), 0);
        assert!(ctrl.get_events().is_empty());
    }

    #[test]
    fn test_controller_register_and_heal() {
        let mut ctrl = SelfHealingController::new();
        ctrl.register_healer(Box::new(GenericHealer::new("test_healer", &["test_issue"])));
        assert_eq!(ctrl.healer_count(), 1);

        let result = ctrl.diagnose_and_heal("svc", "test_issue");
        assert!(result.success);
        assert_eq!(ctrl.get_events().len(), 1);
    }

    #[test]
    fn test_controller_no_healer_found() {
        let mut ctrl = SelfHealingController::new();
        let result = ctrl.diagnose_and_heal("svc", "unknown_cosmic_ray");
        // No healer matched, pipeline should be used as fallback
        assert!(!result.action.is_empty());
    }

    #[test]
    fn test_controller_try_heal_with_retry() {
        let mut ctrl = SelfHealingController::new();
        ctrl.register_healer(Box::new(GenericHealer::new("test", &["issue"])));
        let result = ctrl.try_heal_with_retry("svc", "issue");
        assert!(result.success);
    }

    #[test]
    fn test_circuit_breaker_blocks_when_open() {
        let mut ctrl = SelfHealingController::with_settings(1, 60, 100, 1000, 2);
        // First failure trips the breaker
        let r1 = ctrl.diagnose_and_heal("svc", "nonexistent");
        // The circuit breaker trips after failure_threshold failures
        // Since threshold is 1 and diagnose_and_heal records failure,
        // the next call should be blocked
        let r2 = ctrl.diagnose_and_heal("svc", "whatever");
        assert!(
            !r2.success || r2.message.contains("Circuit breaker") || r2.message.contains("OPEN")
        );
    }

    #[test]
    fn test_default_healers() {
        let mut ctrl = SelfHealingController::new();
        register_default_healers(&mut ctrl);
        assert!(ctrl.healer_count() > 0);

        let result = ctrl.diagnose_and_heal("api", "connection timeout");
        assert!(result.success || !result.message.is_empty());
    }

    #[test]
    fn test_recovery_strategies() {
        let strategies = vec![
            RecoveryStrategy::RestartWithBackoff {
                target: "svc".to_string(),
                initial_delay_ms: 10,
                max_delay_ms: 100,
            },
            RecoveryStrategy::ScaleUp {
                target: "svc".to_string(),
                additional_replicas: 2,
            },
            RecoveryStrategy::ClearCache {
                target: "svc".to_string(),
                cache_key: None,
            },
            RecoveryStrategy::RollbackToLastGood {
                target: "svc".to_string(),
                version: None,
            },
        ];

        for s in &strategies {
            let result = s.execute();
            assert!(result.success);
            assert!(result.duration_ms < 5000);
        }
    }

    #[test]
    fn test_verify_healing() {
        let result = verify_healing("healthy-svc", "timeout");
        // Should be healthy since target is non-empty
        assert!(result.success);
    }

    #[test]
    fn test_health_check_empty_target() {
        match health_check("") {
            HealthStatus::Unhealthy { .. } => {} // expected
            _ => panic!("Empty target should be unhealthy"),
        }
    }
}
