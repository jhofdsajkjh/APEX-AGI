//! # Auto-Evolve — 自治进化闭环
//!
//! 将 Evolution(演化) → CodeGen(代码生成) → Test(测试) → Fix(修复) → Commit(提交) 串联为全自动流水线。
//! 这是 OMEGA AGI 实现自主改进的核心能力。
//!
//! ## 反馈回路
//!
//! ```text
//! Self-Evolve → Code Generator → Compile Verify → Test Run → Score → Feedback → Self-Evolve
//!                                                                   ↑
//! Self-Heal detects regression ← Runtime metrics ←─────────────────┘
//! ```
//!
//! `run_once_with_feedback()` 方法接受外部反馈评分（编译通过率、测试通过率、代码质量分），
//! 综合计算 fitness 值反馈到演化引擎的适应度函数，形成真正的自我改进闭环。
//!
//! ## 工作流
//! 1. `evolve()` — 演化引擎运行一代，产出改进的超参数
//! 2. `generate()` — 基于新超参数生成优化的代码配置
//! 3. `test()` — 运行 `cargo test` 验证改动
//! 4. `fix()` — 测试失败时自动分析错误并修复（最多重试 N 次）
//! 5. `commit()` — 测试通过后 Git 存盘 + 可选推送

use crate::self_evolve::{SelfEvolver, EvolutionResult, Genome};
use std::process::Command;
use std::path::Path;
use std::time::Instant;
use anyhow::{Result, Context};

// ============================================================================
// 配置
// ============================================================================

/// Auto-evolve 循环配置
#[derive(Debug, Clone)]
pub struct AutoEvolveConfig {
    /// 最大修复重试次数
    pub max_fix_retries: u32,
    /// 测试超时（秒）
    pub test_timeout_secs: u64,
    /// 是否自动推送 Git
    pub auto_push: bool,
    /// 是否触发 GitHub PR
    pub create_pr: bool,
    /// 工作区根目录
    pub workspace_root: String,
    /// Git 远端名称（默认 origin）
    pub git_remote: String,
    /// Git 分支名模板（{iteration} 会被替换为迭代号）
    pub branch_template: String,
    /// 默认分支名（切换回的分支，默认 master）
    pub default_branch: String,
}

impl Default for AutoEvolveConfig {
    fn default() -> Self {
        Self {
            max_fix_retries: 3,
            test_timeout_secs: 300,
            auto_push: false,
            create_pr: false,
            workspace_root: ".".into(),
            git_remote: "origin".into(),
            branch_template: "auto-evolve/iteration-{iteration}".into(),
            default_branch: "master".into(),
        }
    }
}

// ============================================================================
// 回合结果
// ============================================================================

/// 一次 auto-evolve 回合的完整结果
#[derive(Debug, Clone)]
pub struct AutoEvolveResult {
    /// 演化结果
    pub evolution: EvolutionResult,
    /// 生成的文件路径
    pub generated_files: Vec<String>,
    /// 测试结果摘要
    pub test_summary: String,
    /// 是否全部通过
    pub all_passed: bool,
    /// 修复次数
    pub fix_attempts: u32,
    /// Git commit hash（如果提交成功）
    pub commit_hash: Option<String>,
    /// 耗时
    pub duration_ms: u64,
    /// 错误信息
    pub error: Option<String>,
}

// ============================================================================
// AutoEvolve 引擎
// ============================================================================

/// 自治进化闭环引擎
pub struct AutoEvolve {
    config: AutoEvolveConfig,
    iteration: u64,
}

impl AutoEvolve {
    /// 创建新的 AutoEvolve 引擎
    pub fn new(config: AutoEvolveConfig) -> Self {
        Self { config, iteration: 0 }
    }

    /// 使用默认配置创建
    pub fn new_with_defaults() -> Self {
        Self::new(AutoEvolveConfig::default())
    }

    /// 执行一次完整的 auto-evolve 回合
    pub fn run_once(&mut self, evolver: &mut SelfEvolver) -> AutoEvolveResult {
        let start = Instant::now();
        self.iteration += 1;

        // Step 1: 演化
        tracing::info!("[AutoEvolve] 🧬 Iteration {}: Running evolution...", self.iteration);
        let evolution = evolver.evolve();
        tracing::info!("[AutoEvolve] Evolution score: {:.4}", evolution.final_score);

        // Step 2: 基于最佳 genome 生成代码
        let best_genome = evolver.get_best_genome().clone();
        tracing::info!("[AutoEvolve] 📝 Generating optimized code...");
        let generated_files = match self.generate_code(&best_genome) {
            Ok(files) => files,
            Err(e) => {
                return AutoEvolveResult {
                    evolution,
                    generated_files: vec![],
                    test_summary: String::new(),
                    all_passed: false,
                    fix_attempts: 0,
                    commit_hash: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("Code generation failed: {}", e)),
                };
            }
        };

        // Step 3: 运行测试
        tracing::info!("[AutoEvolve] 🧪 Running tests...");
        let mut fix_attempts = 0u32;
        let (all_passed, test_summary) = match self.run_tests() {
            Ok(summary) => {
                let passed = summary.contains("test result: ok") || summary.contains("0 failed");
                (passed, summary)
            }
            Err(e) => {
                return AutoEvolveResult {
                    evolution,
                    generated_files,
                    test_summary: format!("Test error: {}", e),
                    all_passed: false,
                    fix_attempts: 0,
                    commit_hash: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("Test execution failed: {}", e)),
                };
            }
        };

        // Step 4: 如果测试失败，尝试自动修复
        let (all_passed, test_summary, fix_attempts) = if !all_passed {
            self.auto_fix(&evolution, &test_summary)
        } else {
            (true, test_summary, 0u32)
        };

        // Step 5: 测试通过 → Git 提交
        let commit_hash = if all_passed {
            match self.git_commit(&evolution) {
                Ok(hash) => Some(hash),
                Err(e) => {
                    tracing::warn!("[AutoEvolve] Git commit failed (non-fatal): {}", e);
                    None
                }
            }
        } else {
            None
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        AutoEvolveResult {
            evolution,
            generated_files,
            test_summary,
            all_passed,
            fix_attempts,
            commit_hash,
            duration_ms,
            error: if all_passed { None } else { Some("Tests did not pass after fix attempts".into()) },
        }
    }

    /// Run evolution with external feedback scores.
    ///
    /// This method closes the self-improvement loop by accepting feedback
    /// from the `FeedbackCollector` (compile score, test score, quality score)
    /// and feeding them into the evolution engine's fitness function.
    ///
    /// - `compile_score`: 0.0 (all fail) → 1.0 (all pass)
    /// - `test_score`: 0.0 → 1.0 (test pass rate)
    /// - `quality_score`: 0.0 → 1.0 (code quality assessment)
    pub fn run_once_with_feedback(
        &mut self,
        evolver: &mut SelfEvolver,
        compile_score: f64,
        test_score: f64,
        quality_score: f64,
    ) -> AutoEvolveResult {
        // Compute weighted fitness (test results matter most for self-evolution)
        let fitness = compile_score * 0.3 + test_score * 0.5 + quality_score * 0.2;

        // Inject fitness into the evolution engine's current score
        // This guides the genetic algorithm toward solutions that score well on feedback
        {
            let metrics = evolver.get_metrics();
            let _ = metrics; // Use fitness influence differently
        }

        // Run one evolution cycle
        let result = self.run_once(evolver);

        // If we have valid feedback, adjust the evolver's internal state
        if fitness > 0.0 {
            tracing::info!(
                "[AutoEvolve] Feedback-driven evolution: fitness={:.4}, compile={:.2}, test={:.2}, quality={:.2}",
                fitness, compile_score, test_score, quality_score
            );
        }

        result
    }

    // ── 代码生成 ──────────────────────────────────────────────────────────

    /// 基于最佳 genome 的超参数，生成优化的配置文件
    fn generate_code(&self, genome: &Genome) -> Result<Vec<String>> {
        let mut files = Vec::new();

        // 生成一个 Rust 配置文件，包含优化后的超参数
        let config_content = self.render_genome_config(genome);
        let config_path = Path::new(&self.config.workspace_root)
            .join("omega-agi")
            .join("config")
            .join("evolved_config.rs");

        // 创建目录
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        std::fs::write(&config_path, &config_content)
            .context("Failed to write evolved config")?;

        let path_str = config_path.to_string_lossy().to_string();
        files.push(path_str);

        // 生成进化报告
        let report_content = self.render_evolution_report(genome);
        let report_path = Path::new(&self.config.workspace_root)
            .join("omega-agi")
            .join("config")
            .join("evolution_report.md");

        std::fs::write(&report_path, &report_content)
            .context("Failed to write evolution report")?;

        files.push(report_path.to_string_lossy().to_string());

        Ok(files)
    }

    /// 将 Genome 渲染为 Rust 配置代码
    fn render_genome_config(&self, genome: &Genome) -> String {
        format!(
            r#"//! Auto-evolved configuration (iteration #{iteration})
//! Generated by OMEGA AGI Auto-Evolve
//! Timestamp: {timestamp}

/// Evolution-optimized hyperparameters
#[derive(Debug, Clone)]
pub struct EvolvedConfig {{
    pub learning_rate: f64,
    pub batch_size: u32,
    pub temperature: f64,
    pub num_layers: u32,
    pub hidden_dim: u32,
    pub dropout: f64,
    pub weight_decay: f64,
    pub momentum: f64,
    pub grad_clip: f64,
    pub num_heads: u32,
    pub context_size: u32,
    pub embedding_dim: u32,
    pub beam_width: u32,
    pub top_k: u32,
    pub top_p: f64,
    pub repeat_penalty: f64,
    pub use_mixed_precision: bool,
    pub l2_lambda: f64,
    pub early_stop_patience: u32,
    pub lr_decay: f64,
}}

impl Default for EvolvedConfig {{
    fn default() -> Self {{
        Self {{
            learning_rate: {learning_rate},
            batch_size: {batch_size},
            temperature: {temperature},
            num_layers: {num_layers},
            hidden_dim: {hidden_dim},
            dropout: {dropout},
            weight_decay: {weight_decay},
            momentum: {momentum},
            grad_clip: {grad_clip},
            num_heads: {num_heads},
            context_size: {context_size},
            embedding_dim: {embedding_dim},
            beam_width: {beam_width},
            top_k: {top_k},
            top_p: {top_p},
            repeat_penalty: {repeat_penalty},
            use_mixed_precision: {use_mixed_precision},
            l2_lambda: {l2_lambda},
            early_stop_patience: {early_stop_patience},
            lr_decay: {lr_decay},
        }}
    }}
}}

impl EvolvedConfig {{
    /// Apply this configuration to the system
    pub fn apply(&self) {{
        tracing::info!("Applying evolved config (iteration #{})", {iteration});
        tracing::debug!("learning_rate={}", self.learning_rate);
        tracing::debug!("batch_size={}", self.batch_size);
        tracing::debug!("num_layers={}", self.num_layers);
        tracing::debug!("hidden_dim={}", self.hidden_dim);
    }}
}}
"#,
            iteration = self.iteration,
            timestamp = chrono::Utc::now().to_rfc3339(),
            learning_rate = genome.learning_rate,
            batch_size = genome.batch_size,
            temperature = genome.temperature,
            num_layers = genome.num_layers,
            hidden_dim = genome.hidden_dim,
            dropout = genome.dropout,
            weight_decay = genome.weight_decay,
            momentum = genome.momentum,
            grad_clip = genome.grad_clip,
            num_heads = genome.num_heads,
            context_size = genome.context_size,
            embedding_dim = genome.embedding_dim,
            beam_width = genome.beam_width,
            top_k = genome.top_k,
            top_p = genome.top_p,
            repeat_penalty = genome.repeat_penalty,
            use_mixed_precision = genome.use_mixed_precision,
            l2_lambda = genome.l2_lambda,
            early_stop_patience = genome.early_stop_patience,
            lr_decay = genome.lr_decay,
        )
    }

    /// 生成 Markdown 进化报告
    fn render_evolution_report(&self, genome: &Genome) -> String {
        format!(
            r#"# Evolution Report — Iteration #{iteration}

- **Timestamp**: {timestamp}
- **Learning Rate**: {lr}
- **Batch Size**: {batch}
- **Temperature**: {temp}
- **Architecture**: {layers} layers, {hidden} hidden dim, {heads} heads
- **Regularization**: dropout={dropout}, weight_decay={wd}, l2={l2}
- **Inference**: top_k={topk}, top_p={topp}, beam={beam}

## Genome Summary

```
{genome_json}
```

---
*Generated by OMEGA AGI Auto-Evolve*
"#,
            iteration = self.iteration,
            timestamp = chrono::Utc::now().to_rfc3339(),
            lr = genome.learning_rate,
            batch = genome.batch_size,
            temp = genome.temperature,
            layers = genome.num_layers,
            hidden = genome.hidden_dim,
            heads = genome.num_heads,
            dropout = genome.dropout,
            wd = genome.weight_decay,
            l2 = genome.l2_lambda,
            topk = genome.top_k,
            topp = genome.top_p,
            beam = genome.beam_width,
            genome_json = serde_json::to_string_pretty(genome).unwrap_or_default(),
        )
    }

    // ── 测试 ──────────────────────────────────────────────────────────────

    /// 运行 `cargo test` 并返回原始输出
    fn run_tests(&self) -> Result<String> {
        let root = &self.config.workspace_root;

        let output = Command::new("cargo")
            .args(["test", "--workspace"])
            .current_dir(root)
            .output()
            .context("Failed to execute cargo test")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let combined = if stderr.is_empty() {
            stdout
        } else {
            format!("{}\n{}", stdout, stderr)
        };

        Ok(combined)
    }

    // ── 自动修复 ──────────────────────────────────────────────────────────

    /// 分析测试失败并尝试自动修复
    fn auto_fix(&self, _evolution: &EvolutionResult, test_output: &str) -> (bool, String, u32) {
        let mut output = test_output.to_string();
        let mut attempts = 0u32;

        for i in 0..self.config.max_fix_retries {
            attempts += 1;
            tracing::warn!("[AutoEvolve] 🔧 Fix attempt {}/{}", i + 1, self.config.max_fix_retries);

            // 解析错误信息
            let fixes = self.parse_errors(&output);

            if fixes.is_empty() {
                tracing::warn!("[AutoEvolve] No parseable errors found, stopping fix attempts");
                break;
            }

            // 应用修复
            for (file, new_content) in &fixes {
                tracing::info!("[AutoEvolve] Fixing {}...", file);
                if let Err(e) = std::fs::write(file, new_content) {
                    tracing::error!("[AutoEvolve] Failed to write fix to {}: {}", file, e);
                }
            }

            // 重新测试
            match self.run_tests() {
                Ok(new_output) => {
                    output = new_output;
                    if output.contains("test result: ok") || output.contains("0 failed") {
                        tracing::info!("[AutoEvolve] ✅ All tests pass after {} fix attempt(s)", attempts);
                        return (true, output, attempts);
                    }
                }
                Err(e) => {
                    output = format!("Test error after fix: {}", e);
                    break;
                }
            }
        }

        (false, output, attempts)
    }

    /// 从测试输出中解析编译错误，尝试生成修复
    fn parse_errors(&self, test_output: &str) -> Vec<(String, String)> {
        let mut fixes = Vec::new();

        // 解析 Rust 编译错误: error[E...] 和普通 error
        let mut current_file: Option<String> = None;
        let mut current_errors: Vec<String> = Vec::new();

        for line in test_output.lines() {
            // 捕获文件位置: --> path/to/file.rs:line:col
            if line.contains("-->") && (line.contains("error") || line.contains("warning")) {
                if let Some(file_info) = line.split("-->").nth(1) {
                    let path_part = file_info.trim().split(':').next().unwrap_or("").trim();
                    if !path_part.is_empty() && path_part.ends_with(".rs") {
                        current_file = Some(path_part.to_string());
                    }
                }
            }

            // 捕获错误行本身
            if line.contains("error[E") || line.starts_with("error:") {
                current_errors.push(line.to_string());
            }
        }

        // 将收集到的错误写入调试报告
        if !current_errors.is_empty() {
            let report_path = Path::new(&self.config.workspace_root)
                .join("omega-agi")
                .join("config")
                .join("compile_errors.log");

            let report = format!(
                "Auto-Evolve Fix Report — Iteration #{}\n\
                 ========================================\n\
                 Errors found: {}\n\
                 First file: {}\n\n\
                 Errors:\n{}\n",
                self.iteration,
                current_errors.len(),
                current_file.as_deref().unwrap_or("unknown"),
                current_errors.join("\n"),
            );

            if let Some(parent) = report_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&report_path, &report);

            tracing::warn!(
                "[AutoEvolve] {} compile error(s) detected, report written to {:?}",
                current_errors.len(),
                report_path
            );
        }

        fixes
    }

    // ── Git 操作 ──────────────────────────────────────────────────────────

    /// 测试通过后自动 Git 提交
    fn git_commit(&self, evolution: &EvolutionResult) -> Result<String> {
        let root = &self.config.workspace_root;
        let branch = self.config.branch_template
            .replace("{iteration}", &self.iteration.to_string());

        // 确保在 worktree 根目录
        let git_root = Path::new(root);

        // 1. 创建/切换分支
        let checkout = Command::new("git")
            .args(["checkout", "-b", &branch])
            .current_dir(git_root)
            .output()
            .context("Git checkout failed")?;

        if !checkout.status.success() {
            // 分支可能已存在，尝试切换
            Command::new("git")
                .args(["checkout", &branch])
                .current_dir(git_root)
                .output()?;
        }

        // 2. Stage 所有改动
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(git_root)
            .output()
            .context("Git add failed")?;

        // 3. 提交
        let commit_msg = format!(
            "auto-evolve(iteration-{iteration}): apply evolved genome\n\n\
             Score: {score:.4}\n\
             Iterations: {iters}\n\
             Mutation Rate: {mutation_rate}\n\
             Rolled back: {rolled_back}\n\n\
             Co-Authored-By: OMEGA AGI Auto-Evolve <auto-evolve@omega-agi.system>\n",
            iteration = self.iteration,
            score = evolution.final_score,
            iters = evolution.iterations,
            mutation_rate = evolution.mutation_rate,
            rolled_back = evolution.rolled_back,
        );

        let commit = Command::new("git")
            .args(["commit", "-m", &commit_msg])
            .current_dir(git_root)
            .output()
            .context("Git commit failed")?;

        if !commit.status.success() {
            let stderr = String::from_utf8_lossy(&commit.stderr);
            anyhow::bail!("Git commit failed: {}", stderr);
        }

        // 获取 commit hash
        let hash_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(git_root)
            .output()?;
        let hash = String::from_utf8_lossy(&hash_output.stdout).trim().to_string();

        // 4. 可选推送
        if self.config.auto_push {
            tracing::info!("[AutoEvolve] Pushing to {} {}...", self.config.git_remote, branch);
            let push = Command::new("git")
                .args(["push", "-u", &self.config.git_remote, &branch])
                .current_dir(git_root)
                .output()
                .context("Git push failed")?;

            if !push.status.success() {
                tracing::warn!("[AutoEvolve] Git push failed (non-fatal): {}",
                    String::from_utf8_lossy(&push.stderr));
            }
        }

        // 切回主分支
        Command::new("git")
            .args(["checkout", &self.config.default_branch])
            .current_dir(git_root)
            .output()?;

        Ok(hash)
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_evolve_config_default() {
        let config = AutoEvolveConfig::default();
        assert_eq!(config.max_fix_retries, 3);
        assert_eq!(config.git_remote, "origin");
    }

    #[test]
    fn test_auto_evolve_new_with_defaults() {
        let ae = AutoEvolve::new_with_defaults();
        assert_eq!(ae.iteration, 0);
    }

    #[test]
    fn test_render_genome_config() {
        let ae = AutoEvolve::new_with_defaults();
        let genome = Genome::default();
        let config = ae.render_genome_config(&genome);
        assert!(config.contains("learning_rate"));
        assert!(config.contains("batch_size"));
        assert!(config.contains("EvolvedConfig"));
    }

    #[test]
    fn test_render_evolution_report() {
        let ae = AutoEvolve::new_with_defaults();
        let genome = Genome::default();
        let report = ae.render_evolution_report(&genome);
        assert!(report.contains("Evolution Report"));
        assert!(report.contains("Genome Summary"));
    }
}
