// OMEGA Engineering Library
// Layer 3 - Code Engineering & Quality Assurance

pub mod code_generator;
pub mod pr_manager;
pub mod quality_gate;
pub mod test_runner;

pub use code_generator::{
    CodeContext, CodeGenerator, CodeQuality, GenError, GeneratedCode, Language,
};
pub use pr_manager::{CheckRun, PRError, PRManager, PRState, PRStatus, Review, ReviewState};
pub use quality_gate::{GateContext, GateResult, PhaseResult, QualityGate, QualityGateRunner};
pub use test_runner::{
    PythonTestCase, RustTestCase, TestError, TestHarness, TestResult, TestSummary, TimeoutConfig,
};

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
