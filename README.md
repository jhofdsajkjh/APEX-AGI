# 🧬 APEX-AGI — 六层 AGI 自治系统

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Contributions Welcome](https://img.shields.io/badge/contributions-welcome-brightgreen)](#contributing)

**从 HyperCore 到自主 Agent 的全栈 Rust 实现 — 开源社区共建**

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
- **Git**

### 安装

```bash
cd omega-agi
cargo build --release
```

### 运行

```bash
cargo run --release -- check      # 健康检查
cargo run --release -- run "任务"  # Agent 执行
```

---

## 🏗️ 架构

| 层级 | 名称 | 职责 |
|:----:|------|------|
| 0 | **HyperCore** | 零分配异步运行时 · 持久化内存 · 能力安全 |
| 1 | **Runtime** | Actor 系统 · WASM 沙箱 · 效果系统 |
| 2 | **Engineering** | 代码生成 · 测试框架 · 质量门禁 |
| 3 | **Evolution** | 自演化引擎 · 遗传算法 · A/B 测试 |
| 4 | **Adapters** | OpenClaw · Hermes · OpenHuman · 飞书 |
| 5 | **Agent** | LLM 驱动 · ReAct 推理 · 工具调用 |

---

## 📋 TODO / 社区共建路线

### 🔴 高优先级
- [ ] **单元测试** — 0% 覆盖率，急需补全
- [ ] **真实自演化闭环** — 当前是模拟数据，需要真实编译→测试→反馈
- [ ] **多 LLM 提供商** — 支持 DeepSeek / Claude / Gemini
- [ ] **流式输出** — SSE/WebSocket 实时响应

### 🟡 中优先级
- [ ] **代码拆分** — 单文件 >500 行需要模块化
- [ ] **Clippy 清理** — 消除所有 lint 警告
- [ ] **Docker 多架构** — arm64 / windows 完整支持
- [ ] **配置文件热加载**

### 🟢 长期愿景
- [ ] **真实多 Agent 并行** — 任务分解 + 子代理委派
- [ ] **技能市场** — 社区插件生态
- [ ] **Web Dashboard** — 可视化监控
- [ ] **分布式 Swarm** — 多节点协同

---

## 🤝 Contributing

我们欢迎所有贡献！无论是修复 Bug、添加测试、改进文档，还是实现新功能。

### 如何开始

1. **Fork** 本仓库
2. **Clone** 到你的机器
3. 选择一个 [TODO](#-todo--社区共建路线) 开始
4. 提交 PR，附上测试

### 代码规范

- 遵循 `cargo fmt` 和 `cargo clippy`
- 所有新功能必须附带测试
- Commit 信息使用中文或英文，清晰描述改动

### 联系我们

- [GitHub Issues](https://github.com/jhofdsajkjh/APEX-AGI/issues)
- 欢迎在 Issue 中讨论架构和路线图

---

## 📜 License

MIT License — 自由使用、修改、分发。
