# AICrypto

AI 驱动的币安永续合约自动化投研与交易平台。

## 项目定位

AICrypto 是一个面向币安永续合约市场的 AI 自动化投研及交易平台。核心目标是建立一套可持续进化的自动化研究、信号发现、策略封装、风险控制、仿真验证与实盘执行体系，重点捕捉主升浪、主跌浪和高波动结构性机会。

## 架构分层

| 层级 | 目录 | 语言 | 职责 |
|------|------|------|------|
| 产品层 | `apps/` | TypeScript | Web 控制台、管理 API、运维工具 |
| 决策层 | `decision/` | Rust | Skill 路由、组合优化、风控网关、执行规划 |
| 执行层 | `execution/` | Rust | 交易网关、订单状态机、对账、应急处理 |
| 智能层 | `intelligence/` | Rust + Python | 特征、状态、相关性、回测、知识图谱、Agent |
| 接入层 | `adapters/` | Rust | 币安及外部数据源适配 |
| 协议层 | `protocols/` | - | 冻结的核心协议定义 |
| 基础层 | `foundation/` | Rust | 配置、日志、监控、安全、存储抽象 |
| 研究层 | `research/` | Python | Notebooks、实验、数据集、报告 |
| Skill 层 | `skills/` | YAML/JSON | 平台正式 Skill 资产库 |
| 基础设施 | `infra/` | - | 本地开发环境、Docker、部署配置 |

## 核心协议

所有模块间交互基于冻结协议，详见 `protocols/` 目录：

- `CanonicalEvent` — 统一事件总线对象
- `MarketSnapshot` — 市场快照
- `FeatureVector` — 特征向量
- `StrategySpec` — 策略定义
- `SkillSpec` — Skill 定义
- `SignalEvent` — 信号事件
- `OrderIntent` — 订单意图
- `RiskDecision` — 风控决策
- `ExecutionReport` — 执行报告
- `ReviewReport` — 复盘报告

## 环境体系

| 环境 | 用途 |
|------|------|
| `dev` | 开发联调 |
| `backtest` | 历史回测 |
| `paper` | 实时仿真 |
| `prod` | 实盘 |

## 技术栈

- **核心热路径**: Rust + Tokio + Axum
- **研究计算**: Python + Polars + Jupyter
- **Web 控制台**: Next.js + React + TypeScript + Tailwind
- **数据库**: PostgreSQL + TimescaleDB
- **缓存**: Redis
- **消息总线**: NATS JetStream
- **可观测性**: Prometheus + Grafana + Loki + OpenTelemetry

## 快速开始

```bash
# 1. 启动本地基础设施
cd infra/docker && docker-compose up -d

# 2. 启动 Rust 核心服务
cargo build --workspace
cargo run --bin gateway-market

# 3. 启动 Web 控制台
cd apps/console-web && npm install && npm run dev
```

## 文档

详细规划文档位于 `docs/` 目录。
