# 🧬 OMEGA AGI Supremacy

[![CI](https://github.com/your-org/omega-agi-supremacy/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/omega-agi-supremacy/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
[![AGI Level](https://img.shields.io/badge/AGI-Layer%205%20Agent-brightgreen)](https://github.com/your-org/omega-agi-supremacy)

**六层 AGI 自治系统 — 从超内核到自主 Agent 的全栈实现**

```
 ┌──────────────────────────────────────────────────────────────┐
 │  🌟 Layer 5: Agent              LLM + ReAct + Tool System    │
 │  ┌────────────────────────────────────────────────────────┐   │
 │  │ 🌐 Layer 4: Adapters  OpenClaw · Hermes · OpenHuman   │   │
 │  │ ┌──────────────────────────────────────────────────┐   │   │
 │  │ │ 🧬 Layer 3: Evolution  自演化引擎 + 自动进化闭环 │   │   │
 │  │ │ ┌────────────────────────────────────────────┐   │   │   │
 │  │ │ │ ⚙️ Layer 2: Engineering  代码生成 · 测试   │   │   │   │
 │  │ │ │ ┌──────────────────────────────────────┐   │   │   │   │
 │  │ │ │ │ 🔄 Layer 1: Runtime  WASM · Actor    │   │   │   │   │
 │  │ │ │ │ ┌────────────────────────────────┐   │   │   │   │   │
 │  │ │ │ │ │ ⚡ Layer 0: HyperCore 零分配   │   │   │   │   │   │
 │  │ │ │ │ │ 异步 · 内存 · 安全              │   │   │   │   │   │
 │  │ │ │ │ └────────────────────────────────┘   │   │   │   │   │
 │  │ │ │ └──────────────────────────────────────┘   │   │   │   │
 │  │ │ └────────────────────────────────────────────┘   │   │   │
 │  │ └──────────────────────────────────────────────────┘   │   │
 │  └────────────────────────────────────────────────────────┘   │
 └──────────────────────────────────────────────────────────────┘
```

---

## 🚀 快速开始

### 前置要求

- **Rust** 1.75+（[rustup.rs](https://rustup.rs)）
- **Git**（用于自动提交功能）

### 安装

```bash
# 克隆仓库
git clone https://github.com/your-org/omega-agi-supremacy.git
cd omega-agi-supremacy

# 复制环境配置
cp .env.example .env
# 编辑 .env 填入你的 LLM API Key（Omni 或 OpenAI 兼容）

# 编译
cargo build --release -p omega-agi
```

### 运行

```bash
# 系统健康检查
cargo run --release -p omega-agi -- check

# 运行 AGI Agent 任务
cargo run --release -p omega-agi -- run "请检查系统健康状态"

# 执行自演化闭环
cargo run --release -p omega-agi -- run "自动演化一迭代并提交改进"

# 查看版本
cargo run --release -p omega-agi -- version
```

---

## 🏗️ 架构概览

### 六层架构

| 层级 | 名称 | 职责 | 状态 |
|:----:|------|------|:----:|
| 0 | **HyperCore** | 零分配异步运行时 · 持久化内存 · 能力安全 | ✅ |
| 1 | **Runtime** | Actor 系统 · WASM 沙箱 · 效果系统 · 图执行器 | ✅ |
| 2 | **Engineering** | 代码生成 · 测试框架 · 质量门禁 · PR 自动化 | ✅ |
| 3 | **Evolution** | 自演化引擎 · 遗传算法 · A/B 测试 · **自动进化闭环** | ✅ |
| 4 | **Adapters** | OpenClaw · Hermes · OpenHuman · 飞书 协议适配 | ✅ |
| 5 | **Agent** | LLM 驱动 · ReAct 推理 · 工具调用 · 自主任务执行 | ✅ |

### 核心流程

```
用户任务
    │
    ▼
┌─────────────────────────────────┐
│  Agent (Layer 5)                │
│  ┌───────────────────────────┐  │
│  │  ReAct Loop               │  │
│  │  Thought → Action → Obs   │  │
│  │  ↕                        │  │
│  │  LLM Client (OpenAI 兼容) │  │
│  └──────────┬────────────────┘  │
└─────────────┼───────────────────┘
              │
              ▼
┌─────────────────────────────────┐
│  ToolContext  — 12 种工具       │
│  health · diagnose · heal       │
│  codegen · evolve · evolve_full │
│  read · write · search · ls     │
│  bash · think                   │
└──────────┬──────────────────────┘
           │
           ▼
    ┌───────────┬───────────┬───────────┐
    ▼           ▼           ▼           ▼
 HyperCore  Runtime  Engineering  Evolution
 (Layer 0)  (Layer 1)  (Layer 2)   (Layer 3)
                              
    ┌───────────┐
    ▼           ▼
 Adapters   Git/PR
 (Layer 4)  输出
```

---

## 🧬 自演化闭环（核心特色）

OMEGA AGI 的杀手级功能：**全自动进化 → 编码 → 测试 → 修复 → 提交流水线**。

```
         ┌───────────────────────────┐
         │  1. 🧬 Evolution          │
         │     遗传算法优化超参数    │
         └───────────┬───────────────┘
                     ▼
         ┌───────────────────────────┐
         │  2. 📝 Code Generation    │
         │     基于最优 Genom 生成   │
         │     优化后的配置代码      │
         └───────────┬───────────────┘
                     ▼
         ┌───────────────────────────┐
         │  3. 🧪 Test Runner        │
         │     cargo test --workspace│
         └───────────┬───────────────┘
                     ▼
            ┌─── 通过？───┐
            │             │
          是│             │否
            ▼             ▼
   ┌────────────┐  ┌────────────┐
   │ 4. ✅ Git  │  │ 4. 🔧 Fix │
   │    Commit  │  │    重试    │
   │    Push    │  │    (×3)    │
   └────────────┘  └──────┬─────┘
                          │
                    还是失败？── 报告
```

使用方式：
```bash
# 一键触发自演化闭环
cargo run --release -p omega-agi -- run "执行一次完整的自动演化"
```

---

## 🔧 工具系统

Agent 拥有 12 种工具，可完整访问所有六层能力：

| 工具 | 功能 |
|------|------|
| `health` | 整体系统健康检查 |
| `diagnose` | 全系统诊断报告 |
| `heal` | 自动修复异常子系统 |
| `codegen` | 代码生成（Rust/Python） |
| `evolve` | 演化引擎单步运行 |
| `evolve_full` | **全自动演化闭环（里程碑）** |
| `read` | 读取文件 |
| `write` | 写入文件 |
| `search` | 源码搜索（grep） |
| `ls` | 目录列表 |
| `bash` | 执行 Shell 命令 |
| `think` | 内部推理（无外部调用） |

---

## 📊 演化引擎

遗传算法驱动的超参数优化器，支持：

- **Genome**: 19 个可优化超参数（学习率、批大小、层数、注意力头数…）
- **变异**: 自适应变异率，基于种群多样性动态调整
- **交叉**: 确定性线性同余随机数生成器，保证可复现
- **谱系追踪**: 完整记录每一代的家谱
- **回滚**: 连续失败时自动回滚
- **A/B 测试**: 并行方案对比

---

## 🐳 Docker 部署

```bash
docker-compose up --build
```

服务将在 `http://localhost:8080` 启动。

---

## 🛠️ 开发指南

### 工作区结构

```
omega-agi/
├── Cargo.toml              # 工作区根
├── src/
│   ├── main.rs             # CLI 入口点
│   └── lib.rs              # OmegaAGI 顶层集成
├── agent/                  # Layer 5 — LLM Agent
│   ├── src/
│   │   ├── lib.rs
│   │   ├── llm.rs          # LLM 客户端
│   │   ├── react.rs        # ReAct 循环引擎
│   │   └── tools.rs        # 工具定义 + 系统接口
├── hypercore/              # Layer 0 — 超内核
├── runtime/                # Layer 1 — 运行时
├── engineering/            # Layer 2 — 工程
│   ├── src/
│   │   ├── lib.rs
│   │   ├── code_generator.rs
│   │   ├── test_runner.rs
│   │   ├── pr_manager.rs
│   │   └── quality_gate.rs
├── evolution/              # Layer 3 — 自演化
│   ├── src/
│   │   ├── lib.rs
│   │   ├── self_evolve.rs   # 遗传算法核心
│   │   └── auto_evolve.rs   # 🆕 自动进化闭环
├── adapters/               # Layer 4 — 适配器
└── config/                 # 自动生成的进化配置
```

### 测试

```bash
# 运行所有测试
cargo test --workspace

# 运行单个 crate 测试
cargo test -p omega-evolution
cargo test -p omega-agent

# 运行特定测试
cargo test -p omega-evolution -- auto_evolve
```

---

## 🔐 安全

- 所有 LLM API Key 通过环境变量注入，不硬编码
- WASM 沙箱隔离不可信代码
- 工具调用有完整的错误处理链
- 演化引擎使用确定性随机数，确保可复现

---

## 📝 License

本项目采用 MIT 或 Apache-2.0 双许可。

---

## 🌟 路线图

- [x] 六层架构基础
- [x] ReAct Agent 推理循环
- [x] 演化引擎 + 遗传算法
- [x] **自动进化闭环（evolve → code → test → fix → commit）**
- [ ] 真实 LLM 接入（已准备，需配置 OMEGA_API_KEY）
- [ ] Web Dashboard 监控面板
- [ ] GitHub Action 自动 PR
- [ ] 多 Agent 协作
- [ ] 跨项目演化学习

---

*Built with ❤️ by OMEGA AGI*
