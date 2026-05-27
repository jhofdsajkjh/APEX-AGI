//! OMEGA AGI Basic Usage Example
//! 
//! Demonstrates core functionality of the OMEGA AGI system.

use omega_agi::{OmegaAGI, Config};
use omega_agi::engineering::{CodeGenerator, Language, CodeContext, TestHarness, TestResult, TestSummary};

#[tokio::main]
async fn main() {
    println!("🌀 OMEGA AGI Supremacy - Basic Example");
    println!("=====================================\n");

    // Initialize configuration
    let config = Config::default()
        .with_github_token("your_github_token_here")
        .with_log_level("info");

    // Create OMEGA AGI instance
    let omega = OmegaAGI::new(config).expect("Failed to initialize OMEGA AGI");

    println!("✅ OMEGA AGI initialized successfully");
    println!("   Version: {}", omega.version());
    println!("   Layers: {}", omega.layer_count());

    // Example: Code generation
    let generator = CodeGenerator::new();
    let context = CodeContext::default();
    let code = generator.generate_with_context("Hello World program in Rust", &context)
        .expect("Code generation failed");
    println!("\n📝 Generated Code:\n{}", code.code);

    // Show quality analysis
    let quality = code.compute_quality_score();
    println!("\n📊 Code Quality:");
    println!("   Overall:   {:.2}", quality.overall_score);
    println!("   Safety:    {:.2}", quality.safety);
    println!("   Readability: {:.2}", quality.readability);
    println!("   Performance: {:.2}", quality.performance);

    // Check for anti-patterns
    let warnings = code.check_rust_antipatterns();
    if !warnings.is_empty() {
        println!("\n⚠️  Warnings:");
        for w in &warnings {
            println!("   - {}", w);
        }
    }

    // Example: Test harness
    let mut harness = TestHarness::new_with_defaults();
    harness.add_rust_test(omega_agi::engineering::test_runner::RustTestCase {
        name: "test_hello".to_string(),
        code: "fn test() { assert_eq!(2 + 2, 4); }".to_string(),
        expected_to_pass: true,
    });
    harness.add_python_test(omega_agi::engineering::test_runner::PythonTestCase {
        name: "test_python_hello".to_string(),
        code: "assert 2 + 2 == 4".to_string(),
        expected_to_pass: true,
    });

    println!("\n🧪 Running tests...");
    let results = harness.run_all();
    println!("   Tests passed: {}/{}", results.passed, results.total);
    println!("   Success rate: {:.1}%", results.success_rate() * 100.0);

    println!("\n✨ OMEGA AGI Basic Example Complete!");
}
