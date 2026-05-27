// OMEGA Engineering Library
// Layer 3 - Code Engineering & Quality Assurance

pub mod code_generator;
pub mod test_runner;
pub mod pr_manager;
pub mod quality_gate;

pub use code_generator::{CodeGenerator, GeneratedCode, Language, CodeQuality, CodeContext, GenError};
pub use test_runner::{TestHarness, TestResult, RustTestCase, PythonTestCase, TestSummary, TestError, TimeoutConfig};
pub use pr_manager::{PRManager, PRState, PRStatus, Review, ReviewState, CheckRun, PRError};
pub use quality_gate::{QualityGate, GateResult, GateContext, QualityGateRunner, PhaseResult};

/// Engineering version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Top-level Engineering wrapper.
pub struct Engineering {
    pub generator: CodeGenerator,
    pub test_runner: TestHarness,
    pub quality_gates: QualityGateRunner,
}

impl Engineering {
    pub fn new() -> Self {
        Self {
            generator: CodeGenerator::new(),
            test_runner: TestHarness::new_with_defaults(),
            quality_gates: QualityGateRunner::new(),
        }
    }
}

impl Default for Engineering {
    fn default() -> Self {
        Self::new()
    }
}
