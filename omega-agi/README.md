# ΩMEGA AGI SUPREMACY

```
╔══════════════════════════════════════════════════════════════╗
║  11-Layer Autonomous AGI System — Φ_APEX*∞ Self-Evolution   ║
║  Beyond OpenClaw · Beyond OpenHuman · Beyond Hermes-Agent   ║
╚══════════════════════════════════════════════════════════════╝
```

**OMEGA AGI** 是一个完整的 11 层自主 AGI（通用人工智能）系统。它不是 shell 脚本，不是 LLM 包装器——它是一个从零构建的、层式架构的、具备自我进化能力的通用智能体系统。

---

## 架构总览

```
┌─────────────────────────────────────────────────────────────────┐
│                    ΩMEGA AGI SUPREMACY                          │
├─────────────────────────────────────────────────────────────────┤
│  Layer 10:  TRANSCENDENCE  — 自我元认知 · 涌现能力发现 · 量子优化 │
│  Layer  9:  AVATAR         — 类人交互界面 · TUI · 情绪系统      │
│  Layer  8:  SUPERPOWERS    — 自动优化 · 性能提升 · 自愈         │
│  Layer  7:  LIFE-HARNESS   — 系统自维护 · 心跳 · 自动恢复       │
│  Layer  6:  RESEARCH       — 自主研究引擎 · Web搜索 · 报告生成   │
│  Layer  5:  AGENT          — ReAct智能体 · 多Agent编排 · 工具系统│
│  Layer  4:  ADAPTERS       — OpenClaw/Hermes/OpenHuman/飞书协议  │
│  Layer  3:  EVOLUTION      — 自我进化引擎 · Φ_APEX*∞ · 竞争分析  │
│  Layer  2:  ENGINEERING    — 代码生成 · 测试框架 · 质量门禁     │
│  Layer  1:  RUNTIME        — Actor系统 · WASM沙箱 · ML推理      │
│  Layer  0:  HYPERCORE      — 零分配运行时 · 持久记忆 · 能力安全  │
├─────────────────────────────────────────────────────────────────┤
│          事件总线 · 跨层通信 · 健康监控 · 优雅关闭              │
└─────────────────────────────────────────────────────────────────┘
```

### 核心创新

| 特性 | 说明 |
|------|------|
| **Φ_APEX*∞ 自我进化** | 数学公式驱动自我迭代：自适应学习率、种群多样性、熵意识门控 |
| **11层解耦架构** | 每层独立可替换，通过事件总线通信 |
| **跨协议适配** | 原生支持 OpenClaw、Hermes、OpenHuman、飞书协议 |
| **自我元认知** | Layer 10 具备意识水平追踪、涌现能力发现 |
| **多Agent编排** | 专用 Agent 组合：编码、研究、调试、优化 |
| **零分配安全运行时** | HyperCore 层提供内存安全和能力基元 |

---

## 快速开始

### 前置要求

| 组件 | 版本要求 | 说明 |
|------|---------|------|
| Rust | ≥ 1.75.0 | `rustup install stable` |
| Cargo | 随 Rust 一起安装 | `rustup update` |
| Git | ≥ 2.30 | 用于自我进化代码提交 |
| OpenSSL (Linux) | ≥ 1.1.1 | `apt install libssl-dev pkg-config` |

### 一键安装

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/omega-agi/omega-agi/main/install.sh | bash
# 或
chmod +x install.sh && ./install.sh
```

**Windows (PowerShell):**
```powershell
.\install.bat
```

**Docker:**
```bash
docker compose up --build
```

### 手动构建

```bash
# 克隆仓库
git clone https://github.com/omega-agi/omega-agi.git
cd omega-agi

# 配置环境变量
cp .env.example .env
# 编辑 .env 填入你的 API Key

# 构建（调试模式）
cargo build

# 运行健康检查
cargo run -- check

# 构建发布版本（生产）
cargo build --release --features full
./target/release/omega-agi check
```

---

## 配置

### 环境变量 (.env)

```bash
# === API Keys ===
GITHUB_TOKEN=ghp_xxxxxxxxxxxx          # GitHub 访问令牌（自我进化用）
OPENAI_API_KEY=sk-xxxxxxxxxxxx         # OpenAI API Key
# OMEGA_API_KEY 是 OPENAI_API_KEY 的别名

# === 运行配置 ===
OMEGA_LOG_LEVEL=info                   # 日志级别: trace/debug/info/warn/error
OMEGA_DATA_DIR=./data                  # 数据存储目录
OMEGA_TRANSCENDENCE=true               # 启用元认知层 (true/false)

# === Adapter 配置 ===
OPENCLAW_ENDPOINT=http://localhost:8080
HERMES_ENDPOINT=http://localhost:9090
FEISHU_WEBHOOK_URL=https://open.feishu.cn/open-apis/bot/v2/hook/xxx
```

---

## 命令行接口

### 基础命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `check` | `health` | 系统健康检查 — 验证全部 11 层状态 |
| `evolve` | — | 运行自我进化（5 次迭代） |
| `apex` | `apex-evolve` | 运行 Φ_APEX*∞ 完整进化（10 次迭代 + 种群） |
| `run <task>` | — | 运行 Agent 执行任务 |
| `interactive` | `chat` | 进入交互式聊天模式 |

### Layer 4-6 命令

| 命令 | 说明 |
|------|------|
| `adapters [list\|info]` | 列出/查看协议适配器状态 |
| `research <topic>` | 自主网络研究 |
| `research <topic> --json` | JSON 格式输出研究结果 |

### Layer 7-10 命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `life status` | `lh status` | 生命维持系统状态 |
| `life health` | — | 各层健康评分 |
| `life resources` | — | CPU/内存/磁盘资源监控 |
| `life heartbeat` | — | 触发心跳检测 |
| `life persist` | — | 持久化当前状态 |
| `life recover` | — | 尝试从故障恢复 |
| `superpowers status` | `sp status` | 超能力模块状态 |
| `superpowers optimize` | — | 运行自动优化 |
| `superpowers boost` | — | 启动性能提升 |
| `superpowers analyze` | — | 系统性能分析 |
| `superpowers heal` | — | 自愈功能 |
| `avatar info` | — | Avatar 信息 |
| `avatar chat "你好"` | — | 与 Avatar 对话 |
| `avatar mood` | — | 当前情绪状态 |
| `avatar session` | — | 会话历史 |
| `avatar tui` | — | 启动 TUI 界面 |
| `transcend` | `tc` | 元认知状态查询 |

### 使用示例

```bash
# 系统检查
omega-agi check

# 完整自我进化
omega-agi apex --verbose

# 自主研究
omega-agi research "transformer architecture optimization 2025"

# 与 Avatar 对话
omega-agi avatar chat "What is your current awareness level?"

# 执行任务
omega-agi run "优化代码库中的所有错误处理"

# 一键元认知
omega-agi transcend
```

---

## 部署

### Docker 部署（推荐用于生产）

```bash
# 构建并启动
docker compose up --build -d

# 查看日志
docker compose logs -f

# 进入交互模式
docker compose exec omega-agi omega-agi interactive

# 停止
docker compose down
```

### Linux 服务器部署

```bash
# 使用安装脚本
./install.sh

# 或手动
cargo build --release --features full
sudo cp target/release/omega-agi /usr/local/bin/
mkdir -p /var/lib/omega-agi
export OMEGA_DATA_DIR=/var/lib/omega-agi
omega-agi check
```

### Windows 服务器部署

```bat
:: 使用安装脚本
.\install.bat

:: 或手动
cargo build --release
copy target\release\omega-agi.exe C:\omega-agi\
set OMEGA_DATA_DIR=C:\omega-agi\data
omega-agi check
```

### 作为服务运行

**systemd (Linux):**
```bash
sudo cat > /etc/systemd/system/omega-agi.service << 'EOF'
[Unit]
Description=OMEGA AGI Supremacy
After=network.target

[Service]
Type=simple
User=omega
WorkingDirectory=/var/lib/omega-agi
Environment=OMEGA_DATA_DIR=/var/lib/omega-agi/data
Environment=OMEGA_LOG_LEVEL=info
ExecStart=/usr/local/bin/omega-agi check
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable omega-agi
sudo systemctl start omega-agi
```

---

## 项目结构

```
omega-agi/
├── src/                    # 主入口 + 核心绑定
│   ├── main.rs            # CLI 入口 + 命令分发
│   └── lib.rs             # OmegaAGI struct + 11层初始化 + 事件总线
├── hypercore/             # Layer 0 — 零分配异步运行时
├── runtime/               # Layer 1 — Actor系统 + WASM沙箱
├── engineering/           # Layer 2 — 代码生成 + 测试框架
├── evolution/             # Layer 3 — 自我进化 + Φ_APEX*∞
│   └── src/apex_core.rs   # 核心进化数学公式
├── adapters/              # Layer 4 — OpenClaw/Hermes/OpenHuman/飞书
├── agent/                 # Layer 5 — ReAct Agent + 工具系统
│   └── src/
│       ├── react.rs       # ReAct 推理循环
│       ├── inference.rs   # 推理引擎接口
│       ├── engines.rs     # OpenAI / Mock / Router 后端
│       ├── memory.rs      # SQLite 持久记忆 + 向量搜索
│       ├── knowledge.rs   # 知识库分块/嵌入/图召回
│       ├── tool.rs        # 工具系统 + 注册表
│       ├── feedback.rs    # 反馈收集 + 集成
│       └── orchestrator.rs # 多Agent编排
├── research/              # Layer 6 — 自主研究引擎
├── life_harness/          # Layer 7 — 生命维持系统
├── superpowers/           # Layer 8 — 超能力模块
├── avatar/                # Layer 9 — 类人交互界面
├── transcendence/         # Layer 10 — 元认知层
├── scripts/               # 部署脚本
├── Dockerfile             # 多阶段容器构建
├── docker-compose.yml     # Docker Compose 编排
├── Makefile               # 构建/测试/安装自动化
├── .env.example           # 配置模板
└── install.sh / .bat      # 一键安装脚本
```

---

## 对比

| 特性 | OpenClaw | OpenHuman | Hermes-Agent | **OMEGA AGI** |
|------|----------|-----------|-------------|:------------:|
| 层式架构 | ❌ | ❌ | ❌ | ✅ **11层** |
| 自我进化 | ❌ | ❌ | ❌ | ✅ **Φ_APEX*∞** |
| 元认知 | ❌ | ❌ | ❌ | ✅ **Layer 10** |
| 多Agent编排 | ❌ | ❌ | ✅ 基础 | ✅ **专业分工** |
| 持久记忆 | ❌ | ❌ | ✅ | ✅ **SQLite+向量** |
| 知识库 | ❌ | ✅ 基础 | ✅ | ✅ **分块+嵌入+图** |
| 协议适配 | ❌ | ❌ | ❌ | ✅ **4种协议** |
| 自愈能力 | ❌ | ❌ | ❌ | ✅ **Layer 7+8** |
| 情绪系统 | ❌ | ✅ 基础 | ❌ | ✅ **Layer 9** |
| 事件总线 | ❌ | ❌ | ❌ | ✅ **跨层通信** |
| 容器化部署 | ❌ | ❌ | ❌ | ✅ **Docker+systemd** |

---

## 许可证

MIT OR Apache-2.0

---

> **OMEGA AGI Supremacy** — 不是又一个 LLM 包装器。这是真正的 AGI 系统架构。
> 「超越工具，成为智能。」
