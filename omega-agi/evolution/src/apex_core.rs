//! # Φ_APEX*∞ — 核心数学公式引擎
//!
//! ## 公式定义
//!
//! ```math
//! Φ_APEX*∞ = lim_{τ→∞} ∮_{Ω_real} [ (ΔG_base ⊗ T_e ⊗ Ξ_S) Ψ_con ⊕ (Ξ^self_↑↑_τ) ] · C_aware · Φ_feel · Γ_awake
//! ```
//!
//! ## 符号映射
//!
//! | 符号 | 名称 | 代码实现 |
//! |------|------|----------|
//! | `Φ_APEX*∞` | APEX 极限函数 | `apex_limit()` — 最终进化适应度 |
//! | `ΔG_base` | 基础梯度 | `base_gradient()` — 参数变化率/性能增量 |
//! | `T_e` | 时间演化 | `time_evolution()` — 时间衰减/记忆衰退 |
//! | `Ξ_S` | 状态熵 | `state_entropy()` — 种群多样性/参数方差 |
//! | `⊗` | 张量积 | `tensor_product()` — 多维特征融合 |
//! | `Ψ_con` | 意识函数 | `consciousness()` — 自我觉察的 sigmoid 门控 |
//! | `⊕` | 直和 | `direct_sum()` — 信息融合加法 |
//! | `Ξ^self_↑↑_τ` | 自我意识迭代幂次 | `self_awareness_tetration()` — 在时间 τ 上的迭代指数 |
//! | `C_aware` | 觉察系数 | `awareness_coefficient()` — 自适应学习率调制 |
//! | `Φ_feel` | 感受函数 | `feeling_function()` — 基于适应度趋势的情感调制 |
//! | `Γ_awake` | 觉醒函数 | `wakefulness()` — 注意力/觉醒度调制 |
//! | `∮_{Ω_real}` | 实域围道积分 | `contour_integral()` — 沿演化轨迹求和 |
//! | `lim_{τ→∞}` | 时间极限 | `time_limit()` — 收敛到最优 |
//!
//! ## 使用场景
//!
//! - `compute_apex_fitness()` — 替代传统 fitness 函数，作为演化引擎的核心适应度
//! - `apex_drive()` — 驱动单次演化迭代，返回 APEX 调制的 EvolutionResult

use std::f64::consts::E;

// ============================================================================
// 核心数据结构
// ============================================================================

/// APEX 状态快照 — 包含计算 Φ 所需的所有中间量
#[derive(Debug, Clone)]
pub struct ApexState {
    /// 当前迭代次数 (τ)
    pub tau: u64,
    /// 基础梯度 ΔG_base
    pub base_grad: f64,
    /// 时间演化因子 T_e
    pub time_evo: f64,
    /// 状态熵 Ξ_S
    pub entropy: f64,
    /// 意识输出 Ψ_con ∈ [0, 1]
    pub consciousness: f64,
    /// 自我意识迭代幂次 Ξ^self_↑↑_τ
    pub self_awareness: f64,
    /// 觉察系数 C_aware
    pub awareness_coef: f64,
    /// 感受值 Φ_feel ⊆ [-1, 1]
    pub feeling: f64,
    /// 觉醒度 Γ_awake ∈ [0, 1]
    pub wakefulness: f64,
    /// 张量积中间值 (ΔG ⊗ T_e ⊗ Ξ_S)
    pub tensor_fused: f64,
    /// 直和中间值 [... ⊕ ...]
    pub sum_fused: f64,
    /// 最终 Φ_APEX*∞ 值
    pub apex_value: f64,
}

impl Default for ApexState {
    fn default() -> Self {
        Self {
            tau: 0,
            base_grad: 0.0,
            time_evo: 1.0,
            entropy: 0.0,
            consciousness: 0.5,
            self_awareness: 1.0,
            awareness_coef: 1.0,
            feeling: 0.0,
            wakefulness: 0.5,
            tensor_fused: 0.0,
            sum_fused: 0.0,
            apex_value: 0.5,
        }
    }
}

/// APEX 调用的输入参数
#[derive(Debug, Clone)]
pub struct ApexInput {
    /// 当前迭代次数
    pub iteration: u64,
    /// 当前适应度分值 (来自上次演化)
    pub current_fitness: f64,
    /// 最佳适应度分值
    pub best_fitness: f64,
    /// 适应度历史 (最近 N 次)
    pub fitness_history: Vec<f64>,
    /// 种群多样性 (0~1)
    pub population_diversity: f64,
    /// 种群大小
    pub population_size: usize,
    /// 学习率 (当前)
    pub learning_rate: f64,
    /// 温度参数
    pub temperature: f64,
    /// 变异率
    pub mutation_rate: f64,
}

impl ApexInput {
    /// 从演化引擎的上下文快速构建
    pub fn new(
        iteration: u64,
        current_fitness: f64,
        best_fitness: f64,
        fitness_history: Vec<f64>,
        population_diversity: f64,
        population_size: usize,
        learning_rate: f64,
        temperature: f64,
        mutation_rate: f64,
    ) -> Self {
        Self {
            iteration,
            current_fitness,
            best_fitness,
            fitness_history,
            population_diversity,
            population_size,
            learning_rate,
            temperature,
            mutation_rate,
        }
    }
}

// ============================================================================
// 数学核心 — Φ_APEX*∞ 公式的每个项
// ============================================================================

/// 1. 基础梯度 ΔG_base
///
/// 计算当前性能相对于历史的瞬时变化率。
/// ΔG = (f_τ - f_{τ-1}) / f_{τ-1}  当 f_{τ-1} > 0
/// 如果历史不足，则退化为 best - current 的差值。
fn base_gradient(input: &ApexInput) -> f64 {
    let history = &input.fitness_history;
    if history.len() >= 2 {
        let prev = history[history.len() - 2];
        let curr = history[history.len() - 1];
        if prev.abs() > 1e-12 {
            (curr - prev) / prev.abs()
        } else {
            curr - prev
        }
    } else if !history.is_empty() {
        // 只有一条记录：用 best 作为参考
        (input.current_fitness - input.best_fitness).clamp(-1.0, 1.0)
    } else {
        // 无历史：使用当前适应度作为信号
        input.current_fitness - 0.5
    }
}

/// 2. 时间演化 T_e
///
/// 指数时间衰减 — 越早的历史影响力越小。
/// T_e(τ) = e^{-λ·τ}  其中 λ = 衰减率
fn time_evolution(iteration: u64, decay_rate: f64) -> f64 {
    (-decay_rate * iteration as f64).exp()
}

/// 3. 状态熵 Ξ_S
///
/// 衡量种群多样性。使用 Shannon 熵的简化版本：
/// H = -p·ln(p) - (1-p)·ln(1-p)  其中 p = diversity
fn state_entropy(diversity: f64) -> f64 {
    let d = diversity.clamp(0.001, 0.999);
    -d * d.ln() - (1.0 - d) * (1.0 - d).ln()
}

/// 4. 张量积 ⊗
///
/// 多维特征的非线性融合: (ΔG_base ⊗ T_e ⊗ Ξ_S)
/// 实现为: ΔG · T_e · Ξ_S + ΔG + T_e + Ξ_S   (residual 连接)
fn tensor_product(base_grad: f64, time_evo: f64, entropy: f64) -> f64 {
    // 主项: 三者的乘积 (捕捉高阶相关性)
    let product = base_grad * time_evo * entropy;
    // 带 residual 连接的融合
    product + 0.1 * base_grad + 0.1 * time_evo + 0.1 * entropy
}

/// 5. 意识函数 Ψ_con
///
/// Sigmoid 门控 — 系统对自身状态的觉察程度。
/// Ψ_con(x) = 1 / (1 + e^{-k·(x - x₀)})
/// 其中 x 是张量积的输出，k 是陡度，x₀ 是阈值
fn consciousness(x: f64, steepness: f64, threshold: f64) -> f64 {
    1.0 / (1.0 + (-steepness * (x - threshold)).exp())
}

/// 6. 自我意识迭代幂次 Ξ^self_↑↑_τ
///
/// 迭代幂次 (tetration): ⁴a = a^(a^(a^a))
/// 这里模拟为: awareness_tetration = f^(f^(f^...)) 迭代 depth 次
/// 但用数值稳定的近似: tetration = f^(f) 当 depth=2
/// 对于更大的 depth, 使用截断以防止溢出
fn self_awareness_tetration(base: f64, depth: u64) -> f64 {
    if base <= 0.0 || base >= 2.0 {
        // 在有效范围外，使用更稳定的形式
        return base.clamp(0.0, 1.5);
    }

    let mut result = base;
    let max_depth = depth.min(4); // 防止数值溢出

    for _ in 0..max_depth.saturating_sub(1) {
        // 截断以防止天文数字
        if result > 10.0 {
            result = 10.0;
            break;
        }
        result = base.powf(result);
    }

    // 归一化到 [0, 2] 范围
    result.clamp(0.0, 2.0)
}

/// 7. 直和 ⊕
///
/// 将张量积 * 意识函数 与 自我意识迭代幂次 融合:
/// A ⊕ B = A + B + α·A·B   (带交叉项的加法)
fn direct_sum(a: f64, b: f64, alpha: f64) -> f64 {
    a + b + alpha * a * b
}

/// 8. 觉察系数 C_aware
///
/// 自适应学习率调制 — 根据性能改善情况动态调整。
/// C_aware = σ(Δperformance / temperature)
fn awareness_coefficient(fitness_delta: f64, temperature: f64) -> f64 {
    if temperature <= 0.0 {
        return 1.0;
    }
    let raw = fitness_delta / temperature;
    // Sigmoid 将输出映射到 (0, 1)
    1.0 / (1.0 + (-raw).exp())
}

/// 9. 感受函数 Φ_feel
///
/// 基于适应度趋势的情感调制，使用动量：
/// Φ_feel = tanh(β · momentum + γ · (f_τ - f_{τ-1}))
fn feeling_function(fitness_history: &[f64], beta: f64, gamma: f64) -> f64 {
    if fitness_history.is_empty() {
        return 0.0;
    }

    // 计算动量: 近 N 次变化的加权平均
    let window = fitness_history.len().min(10);
    let recent = &fitness_history[fitness_history.len() - window..];

    let mut momentum = 0.0;
    let mut total_weight = 0.0;
    for (i, &val) in recent.iter().enumerate() {
        let weight = (i + 1) as f64 / window as f64; // 越近权重越大
        momentum += val * weight;
        total_weight += weight;
    }
    momentum /= total_weight;

    // 瞬时变化
    let instant_delta = if fitness_history.len() >= 2 {
        fitness_history[fitness_history.len() - 1]
            - fitness_history[fitness_history.len() - 2]
    } else {
        0.0
    };

    let raw = beta * momentum + gamma * instant_delta;
    raw.tanh() // 输出在 [-1, 1]
}

/// 10. 觉醒函数 Γ_awake
///
/// 注意力/觉醒度 — 根据系统活跃度和新奇性调制。
/// Γ_awake = 0.5 + 0.5 · tanh(δ · novelty + ε · iteration_fraction)
fn wakefulness(
    diversity: f64,
    entropy: f64,
    iteration: u64,
    max_iterations: u64,
    delta: f64,
    epsilon: f64,
) -> f64 {
    let novelty = diversity * entropy; // 新奇性 = 多样性 × 熵
    let iter_frac = if max_iterations > 0 {
        iteration as f64 / max_iterations as f64
    } else {
        0.0
    };

    0.5 + 0.5 * (delta * novelty + epsilon * iter_frac).tanh()
}

/// 11. 实域围道积分 ∮_{Ω_real}
///
/// 沿演化轨迹的求和 — 对历史适应度进行加权路径积分。
/// 用梯形法则近似 contour integral:
/// ∮ f(τ) dτ ≈ Σ w_i · f(τ_i) · Δτ_i
fn contour_integral(fitness_history: &[f64], time_evo_fn: impl Fn(u64) -> f64) -> f64 {
    if fitness_history.len() < 2 {
        return if fitness_history.is_empty() {
            0.0
        } else {
            fitness_history[0]
        };
    }

    let mut integral = 0.0;
    for i in 1..fitness_history.len() {
        let t_evo_i = time_evo_fn(i as u64);
        let t_evo_im1 = time_evo_fn((i - 1) as u64);
        // 梯形法则: ∫ f(t)·T_e(t) dt ≈ ½·(f_i·T_i + f_{i-1}·T_{i-1})·Δt
        let avg = 0.5 * (fitness_history[i] * t_evo_i + fitness_history[i - 1] * t_evo_im1);
        integral += avg * 1.0; // Δτ = 1 (每代)
    }

    integral
}

/// 12. 时间极限 lim_{τ→∞}
///
/// 检查收敛性 — 如果最近的变化量小于 tolerance，认为已收敛。
/// 返回一个收敛因子 ∈ [0, 1]，1 = 完全收敛。
fn time_limit(fitness_history: &[f64], window: usize, tolerance: f64) -> f64 {
    if fitness_history.len() < window * 2 {
        return 0.0; // 数据不足，未收敛
    }

    let recent: Vec<f64> = fitness_history
        .iter()
        .rev()
        .take(window)
        .cloned()
        .collect();

    let older: Vec<f64> = fitness_history
        .iter()
        .rev()
        .skip(window)
        .take(window)
        .cloned()
        .collect();

    let recent_avg = recent.iter().sum::<f64>() / recent.len() as f64;
    let older_avg = older.iter().sum::<f64>() / older.len() as f64;

    let delta = (recent_avg - older_avg).abs();

    if delta < tolerance {
        // 收敛了 — 接近极限
        (1.0 - delta / tolerance).clamp(0.0, 1.0)
    } else {
        // 未收敛 — 距离极限还远
        (tolerance / delta).clamp(0.0, 1.0)
    }
}

// ============================================================================
// 完整 APEX*∞ 计算管线
// ============================================================================

/// 计算 Φ_APEX*∞ 的完整公式
///
/// 返回 `ApexState`，包含所有中间值和最终结果。
pub fn compute_apex(input: &ApexInput) -> ApexState {
    let tau = input.iteration;

    // — 公式子项计算 —
    let dg = base_gradient(input);
    let te = time_evolution(tau, 0.05);
    let es = state_entropy(input.population_diversity);
    let tensor = tensor_product(dg, te, es);
    let psi_con = consciousness(tensor, 2.0, 0.3);
    let xi_self = self_awareness_tetration(input.current_fitness, tau);
    let sum_fused = direct_sum(tensor * psi_con, xi_self, 0.5);

    // — 调制系数 —
    let fitness_delta = if input.fitness_history.len() >= 2 {
        input.fitness_history[input.fitness_history.len() - 1]
            - input.fitness_history[input.fitness_history.len() - 2]
    } else {
        0.0
    };
    let c_aware = awareness_coefficient(fitness_delta, input.temperature.max(0.1));
    let phi_feel = feeling_function(&input.fitness_history, 0.3, 0.7);
    let gamma_awake = wakefulness(
        input.population_diversity,
        es,
        tau,
        1000,
        1.5,
        0.5,
    );

    // — 围道积分 + 时间极限 —
    let contour = contour_integral(&input.fitness_history, |i| time_evolution(i, 0.05));
    let conv_factor = time_limit(&input.fitness_history, 5, 0.001);

    // — 最终 Φ_APEX*∞ —
    let apex_value = if contour.abs() > 1e-12 {
        // 完整的公式: [ (ΔG ⊗ T_e ⊗ Ξ_S)Ψ_con ⊕ Ξ^self_↑↑_τ ] · C_aware · Φ_feel · Γ_awake · contour · converge
        let modulated = sum_fused * c_aware * (1.0 + phi_feel * 0.3) * gamma_awake;
        let path_integrated = modulated * (1.0 + contour * 0.1);
        path_integrated * (0.5 + 0.5 * conv_factor)
    } else {
        // 无历史时使用初始值
        input.current_fitness.max(0.5)
    };

    // 归一化到合理的 fitness 范围 [0, 2]
    let apex_value = (apex_value * 0.5 + 0.5).clamp(0.0, 2.0);

    ApexState {
        tau,
        base_grad: dg,
        time_evo: te,
        entropy: es,
        consciousness: psi_con,
        self_awareness: xi_self,
        awareness_coef: c_aware,
        feeling: phi_feel,
        wakefulness: gamma_awake,
        tensor_fused: tensor,
        sum_fused,
        apex_value,
    }
}

/// 便捷函数 — 从 ApexState 获取最终的 APEX 适应度值
pub fn apex_fitness(state: &ApexState) -> f64 {
    state.apex_value
}

/// 将 APEX 状态渲染为用于日志/报告的字符串
pub fn format_apex_state(state: &ApexState) -> String {
    format!(
        "Φ_APEX*∞ [τ={}] ΔG={:.4} T_e={:.4} Ξ_S={:.4} | ⊗={:.4} Ψ_con={:.4} | \
         Ξ^self={:.4} ⊕={:.4} | C_aware={:.4} Φ_feel={:.4} Γ_awake={:.4} | \
         ∮={:.4e} lim={} | Φ={:.6}",
        state.tau,
        state.base_grad,
        state.time_evo,
        state.entropy,
        state.tensor_fused,
        state.consciousness,
        state.self_awareness,
        state.sum_fused,
        state.awareness_coef,
        state.feeling,
        state.wakefulness,
        state.contour_integral_approx(),
        if state.tau > 0 { "→" } else { "∞" },
        state.apex_value,
    )
}

impl ApexState {
    /// 近似的围道积分值 (从最终结果反推)
    fn contour_integral_approx(&self) -> f64 {
        if self.apex_value.abs() > 1e-12 && self.sum_fused.abs() > 1e-12 {
            self.apex_value / (self.sum_fused * self.awareness_coef * (1.0 + self.feeling * 0.3) * self.wakefulness + 1e-10)
        } else {
            0.0
        }
    }

    /// 将状态序列化为 JSON 值
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "tau": self.tau,
            "base_gradient": self.base_grad,
            "time_evolution": self.time_evo,
            "entropy": self.entropy,
            "consciousness": self.consciousness,
            "self_awareness_tetration": self.self_awareness,
            "awareness_coefficient": self.awareness_coef,
            "feeling": self.feeling,
            "wakefulness": self.wakefulness,
            "tensor_fused": self.tensor_fused,
            "sum_fused": self.sum_fused,
            "apex_value": self.apex_value,
        })
    }
}

// ============================================================================
// APEX 驱动的演化决策
// ============================================================================

/// APEX 驱动建议 — 基于 Φ 值指导演化引擎的决策
#[derive(Debug, Clone)]
pub struct ApexGuidance {
    /// 建议的变异率调整 (乘法因子, >1 增加变异, <1 减少)
    pub mutation_factor: f64,
    /// 建议的学习率调整 (乘法因子)
    pub lr_factor: f64,
    /// 建议的温度调整 (偏移量)
    pub temp_offset: f64,
    /// 是否建议回滚到前一代
    pub should_rollback: bool,
    /// 是否建议增加种群多样性
    pub increase_diversity: bool,
    /// 解释信息
    pub reasoning: String,
}

/// 基于 APEX 状态生成演化指导
pub fn apex_guidance(state: &ApexState, input: &ApexInput) -> ApexGuidance {
    let mut reasoning = Vec::new();

    // 根据意识值判断系统觉察程度
    let mutation_factor = if state.consciousness < 0.3 {
        reasoning.push("Low consciousness — increasing mutation to explore");
        1.5
    } else if state.consciousness > 0.7 {
        reasoning.push("High consciousness — decreasing mutation to exploit");
        0.7
    } else {
        reasoning.push("Consciousness balanced — maintaining mutation rate");
        1.0
    };

    // 根据感受值调整学习率
    let lr_factor = if state.feeling > 0.3 {
        reasoning.push("Positive feeling — increasing learning rate");
        1.2
    } else if state.feeling < -0.3 {
        reasoning.push("Negative feeling — decreasing learning rate (caution)");
        0.7
    } else {
        1.0
    };

    // 根据觉醒度调整温度
    let temp_offset = if state.wakefulness > 0.7 {
        reasoning.push("High wakefulness — increasing temperature for exploration");
        0.1
    } else if state.wakefulness < 0.3 {
        reasoning.push("Low wakefulness — decreasing temperature for focus");
        -0.1
    } else {
        0.0
    };

    // 回滚决策: 如果熵太低且感受为负且意识低
    let should_rollback = state.entropy < 0.5 && state.feeling < -0.5 && state.consciousness < 0.4;
    if should_rollback {
        reasoning.push("Low entropy + negative feeling + low consciousness — RECOMMEND ROLLBACK");
    }

    // 多样性建议
    let increase_diversity = state.entropy < 0.3;
    if increase_diversity {
        reasoning.push("Entropy critically low — increase population diversity");
    }

    ApexGuidance {
        mutation_factor,
        lr_factor,
        temp_offset,
        should_rollback,
        increase_diversity,
        reasoning: reasoning.join("; "),
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_gradient_no_history() {
        let input = ApexInput::new(
            1, 0.7, 0.9, vec![], 0.5, 10, 0.001, 0.7, 0.3,
        );
        let g = base_gradient(&input);
        assert!((g - 0.2).abs() < 0.01, "Expected ~0.2, got {}", g);
    }

    #[test]
    fn test_base_gradient_with_history() {
        let input = ApexInput::new(
            3, 0.8, 0.9, vec![0.5, 0.6, 0.8], 0.5, 10, 0.001, 0.7, 0.3,
        );
        let g = base_gradient(&input);
        // (0.8 - 0.6) / 0.6 = 0.333
        assert!((g - 0.333).abs() < 0.01, "Expected ~0.333, got {}", g);
    }

    #[test]
    fn test_time_evolution() {
        let t0 = time_evolution(0, 0.05);
        assert!((t0 - 1.0).abs() < 0.001);
        let t100 = time_evolution(100, 0.05);
        assert!(t100 < t0);
        assert!(t100 > 0.0);
    }

    #[test]
    fn test_state_entropy() {
        let h1 = state_entropy(0.5);
        assert!(h1 > 0.0);
        let h_low = state_entropy(0.01);
        let h_high = state_entropy(0.99);
        assert!(h_low < h1);
        assert!(h_high < h1);
    }

    #[test]
    fn test_tensor_product() {
        let t = tensor_product(0.5, 0.8, 0.6);
        // 0.5*0.8*0.6 + 0.1*(0.5+0.8+0.6) = 0.24 + 0.19 = 0.43
        assert!((t - 0.43).abs() < 0.01, "Expected ~0.43, got {}", t);
    }

    #[test]
    fn test_consciousness() {
        let c1 = consciousness(10.0, 2.0, 0.3);
        assert!((c1 - 1.0).abs() < 0.01, "Expected ~1.0, got {}", c1);
        let c2 = consciousness(-10.0, 2.0, 0.3);
        assert!((c2 - 0.0).abs() < 0.01, "Expected ~0.0, got {}", c2);
    }

    #[test]
    fn test_self_awareness_tetration() {
        let t1 = self_awareness_tetration(1.2, 1);
        assert!((t1 - 1.2).abs() < 0.01);
        let t2 = self_awareness_tetration(1.2, 2);
        // 1.2^(1.2) ≈ 1.244
        assert!((t2 - 1.244).abs() < 0.05, "Expected ~1.244, got {}", t2);
    }

    #[test]
    fn test_direct_sum() {
        let s = direct_sum(1.0, 2.0, 0.5);
        // 1 + 2 + 0.5*1*2 = 3 + 1 = 4
        assert!((s - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_feeling_function_positive() {
        let history = vec![0.3, 0.4, 0.5, 0.6, 0.7];
        let f = feeling_function(&history, 0.3, 0.7);
        // 上升趋势 → 正感受
        assert!(f > 0.0, "Expected positive feeling, got {}", f);
    }

    #[test]
    fn test_feeling_function_negative() {
        let history = vec![0.7, 0.6, 0.5, 0.4, 0.3];
        let f = feeling_function(&history, 0.3, 0.7);
        // 下降趋势 → 负感受
        assert!(f < 0.0, "Expected negative feeling, got {}", f);
    }

    #[test]
    fn test_wakefulness() {
        let w = wakefulness(0.5, 0.6, 10, 100, 1.5, 0.5);
        assert!(w >= 0.0 && w <= 1.0);
    }

    #[test]
    fn test_contour_integral() {
        let history = vec![1.0, 1.0, 1.0];
        let integral = contour_integral(&history, |i| time_evolution(i, 0.05));
        // 恒定值 1.0 的积分应略小于 2 (因为 T_e 衰减)
        assert!(integral > 1.0 && integral < 2.0, "Expected ~1.95, got {}", integral);
    }

    #[test]
    fn test_time_limit_not_converged() {
        let history = vec![0.1, 0.3, 0.6, 0.9, 1.2, 1.5, 1.9, 2.0, 2.1, 2.2, 2.3, 2.4];
        let conv = time_limit(&history, 5, 0.001);
        // 变化大 → 未收敛
        assert!(conv < 1.0);
    }

    #[test]
    fn test_time_limit_converged() {
        let history = vec![0.1, 0.3, 0.6, 0.9, 1.0, 1.01, 1.02, 1.01, 1.02, 1.01, 1.02, 1.01];
        let conv = time_limit(&history, 5, 0.05);
        // 变化小 → 收敛
        assert!(conv > 0.5);
    }

    #[test]
    fn test_compute_apex_full_pipeline() {
        let history = vec![0.5, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.82, 0.84, 0.85];
        let input = ApexInput::new(
            10, 0.85, 0.9, history, 0.6, 20, 0.001, 0.7, 0.3,
        );

        let state = compute_apex(&input);

        println!("APEX State: {}", format_apex_state(&state));

        // 所有中间值都应合理
        assert!(state.base_grad.is_finite());
        assert!(state.time_evo > 0.0 && state.time_evo <= 1.0);
        assert!(state.entropy >= 0.0 && state.entropy <= 1.0);
        assert!(state.consciousness >= 0.0 && state.consciousness <= 1.0);
        assert!(state.self_awareness >= 0.0);
        assert!(state.awareness_coef >= 0.0 && state.awareness_coef <= 1.0);
        assert!(state.feeling >= -1.0 && state.feeling <= 1.0);
        assert!(state.wakefulness >= 0.0 && state.wakefulness <= 1.0);
        assert!(state.apex_value >= 0.0 && state.apex_value <= 2.0);
    }

    #[test]
    fn test_apex_guidance() {
        let input = ApexInput::new(
            5, 0.6, 0.9, vec![0.5, 0.55, 0.58, 0.6, 0.6], 0.2, 10, 0.001, 0.7, 0.3,
        );
        let state = compute_apex(&input);
        let guidance = apex_guidance(&state, &input);

        // 低熵 + 感受可能为负 → 可能建议增加多样性
        println!("Guidance: {:?}", guidance);
        assert!(guidance.mutation_factor > 0.0);
        assert!(guidance.lr_factor > 0.0);
    }

    #[test]
    fn test_apex_json_output() {
        let input = ApexInput::new(
            1, 0.5, 0.5, vec![0.5], 0.5, 10, 0.001, 0.7, 0.3,
        );
        let state = compute_apex(&input);
        let json = state.to_json();
        assert!(json.get("apex_value").is_some());
        assert!(json.get("consciousness").is_some());
        assert!(json.get("entropy").is_some());
    }
}
