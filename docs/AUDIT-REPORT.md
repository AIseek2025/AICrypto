# AICrypto 项目审计报告

**审计日期**: 2026-04-24  
**审计范围**: 全项目（Rust 后端 + Next.js 前端 + 基础设施）  
**当前版本**: M7-Phase 8 完成后

---

## 1. 项目概况

| 指标 | 数值 |
|------|------|
| Rust 源文件 | 68 |
| Rust 代码行数 | 8,115 |
| TypeScript/TSX 文件 | 19 |
| Rust Crates | 11（含 7 个可执行 binary） |
| 前端页面路由 | 13 |
| API 端点 | 11 |
| 技能 (Skills) | 12 |
| 风控规则 | 7 |
| 协议结构体 | 10+ |

---

## 2. 全链路测试结果

### 2.1 Rust Binary 测试

| Binary | 状态 | 关键指标 |
|--------|------|---------|
| `backtest-engine` | ✅ PASS | 32 笔交易，胜率 96.9%，利润因子 1319 |
| `signal-runtime` | ✅ PASS | 12 个 skill 加载，4 场景评估完成 |
| `risk-engine` | ✅ PASS | reduce-only 自动通过，大仓位触发 Shrink |
| `portfolio-engine` | ✅ PASS | 5 个信号处理，仓位追踪正常 |
| `gateway-trading` | ✅ PASS | 4 笔订单，状态机完整流转（Draft→Filled） |
| `pipeline-integration` | ✅ PASS | 5 阶段 pipeline 全链路通过 |
| `api-gateway` | ✅ PASS | 8080 端口监听，11 个端点响应 |

### 2.2 API 端点测试

| 端点 | 状态 | 备注 |
|------|------|------|
| `GET /health` | ✅ 200 | 返回 `{"status":"ok","version":"0.1.0"}` |
| `GET /api/skills` | ✅ 200 | 返回 12 个 skill |
| `GET /api/signals` | ✅ 200 | 返回空数组（正常，需先运行 pipeline） |
| `GET /api/portfolio` | ✅ 200 | 返回权益/敞口/持仓数据 |
| `GET /api/portfolio/positions` | ⚠️ 未独立测试 | |
| `GET /api/risk/events` | ⚠️ 未独立测试 | |
| `GET /api/risk/rules` | ⚠️ 未独立测试 | |
| `GET /api/executions` | ⚠️ 未独立测试 | |
| `POST /api/run-pipeline` | ⚠️ 未独立测试 | |
| `POST /api/market/{symbol}/evaluate` | ⚠️ 未独立测试 | |

### 2.3 前端构建

| 指标 | 状态 |
|------|------|
| `next build` | ✅ 零错误 |
| 页面路由 | ✅ 13/13 全部生成 |
| TypeScript | ✅ 类型检查通过 |

### 2.4 编译

| 指标 | 状态 |
|------|------|
| `cargo check` | ✅ 零 warning |
| `cargo build` | ✅ 零 error |

---

## 3. 安全审计

### 🔴 HIGH — 默认数据库凭证硬编码

**位置**: `crates/foundation/src/config.rs:57`

```rust
url: "postgres://aicrypto:aicrypto@localhost:5432/aicrypto".to_string()
```

**风险**: 源码中包含用户名密码，若部署时未设置环境变量则使用此默认值。  
**建议**: 改为启动时强制要求设置 `DATABASE_URL`，不提供默认值。

### 🟡 MEDIUM — API 密钥未脱敏

**位置**: `crates/foundation/src/config.rs:41-42`

`BinanceConfig` 的 `api_key` 和 `api_secret` 是普通 `String`，实现了 `Debug` 和 `Serialize`。若 config 被日志打印或序列化，密钥会泄露。

**建议**: 创建 `SecretString` 包装类型，实现 `Debug` 时显示 `***`。

### 🟡 MEDIUM — listenKey 明文日志

**位置**: `crates/gateway-market/src/binance/user_stream.rs:50`

**建议**: 移除 listenKey 的日志输出，或截断显示。

### 🟢 OK — 无硬编码 API 密钥

所有密钥均从环境变量读取（`BINANCE_API_KEY`、`BINANCE_API_SECRET`）。✅

---

## 4. 代码质量审计

### 🔴 BUG — 仓位追踪器不同步

**位置**: `crates/portfolio-engine/src/portfolio.rs`

`handle_exit()`、`handle_reduce()`、`handle_risk_alert()` 从未调用 `tracker.reduce_position()`。追踪器状态与实际持仓不一致，导致：
- 后续信号基于错误的仓位信息做决策
- 风险敞口计算错误

**影响**: 严重 — 实盘使用会导致仓位管理混乱。

### 🔴 BUG — 风控状态不更新

**位置**: `crates/pipeline-integration/src/pipeline.rs`

pipeline 执行后 `RiskState`（总敞口、持仓数、PnL）从不更新。风控引擎始终基于初始空状态评估，所有风控检查形同虚设。

**影响**: 严重 — 风控引擎完全失效。

### 🔴 BUG — NATS pipeline 价格为零

**位置**: `crates/pipeline-integration/src/pipeline.rs:238`

NATS 路径 `current_price` 硬编码为 `0.0`，导致所有仓位追踪和 PnL 计算无意义。

### 🟡 STUB — 交易所执行器未接入真实 API

**位置**: `crates/gateway-trading/src/executor.rs:56-61`

`dry_run = false` 时仍调用 `simulate_execution()`，无真实 HTTP 请求。live 路径是空壳。

### 🟡 STUB — NATS 消息总线未接入

`BusClient` 已定义但无任何 gateway 使用。事件创建后立即丢弃（`let _event = ...`）。

### 🟡 GAP — 市价单绕过风控检查

**位置**: `crates/risk-engine/src/rules.rs:87-88`

Market order 的 `price_limit` 默认为 `0.0`，导致 `notional = 0`，名义价值和敞口检查被绕过。

### 🟡 GAP — 技能状态未过滤

**位置**: `crates/signal-runtime/src/signal_engine.rs:36`

draft/disabled 状态的 skill 也参与评估，应在生产环境只评估 `Live` 状态的 skill。

### 🟡 GAP — Shrink 判定未处理

pipeline 收到 `RiskVerdict::Shrink` 后直接拒绝订单，无缩减仓位逻辑。`Review` 判定同样无人工队列。

### 🟡 UNUSED — 止损/止盈订单从未触发

**位置**: `crates/portfolio-engine/src/order_builder.rs:76-136`

`build_stop_loss()` 和 `build_take_profit()` 已实现但从未被调用。

### 🟡 UNUSED — 相关性风控配置未执行

**位置**: `crates/risk-engine/src/rules.rs:13`

`max_correlated_exposure_pct` 字段存在但无检查逻辑。

### 🟢 MINOR — `SkillSpec.output_contract` 始终为 Null

**位置**: `crates/signal-runtime/src/skill_registry.rs:134`

### 🟢 MINOR — K线批量插入非真正批量

**位置**: `crates/gateway-market/src/persistence/db.rs:154-171`

函数名含 batch 但逐行 INSERT，大数据量时性能差。

---

## 5. 前端审计

### 5.1 页面完整性

| 页面 | 路由 | 数据源 | 状态 |
|------|------|--------|------|
| Dashboard | `/` | ✅ Live API + Fallback | 完整 |
| Skills | `/skills` | ✅ Live API + Fallback | 完整 |
| Skill Detail | `/skills/[id]` | 静态数据 | 完整 |
| Risk | `/risk` | ✅ Live API + Fallback | 完整 |
| Market | `/market` | 静态数据 | 完整 |
| Symbol Detail | `/market/[symbol]` | 静态数据 | 完整 |
| Agents | `/agents` | 静态数据 | 完整 |
| Live Trading | `/live` | 静态数据 | 完整 |
| Paper Trading | `/paper-trading` | 静态数据 | 完整 |
| Backtests | `/backtests` | 静态数据 | 完整 |
| Audit Log | `/audit` | 静态数据 | 完整 |
| Settings | `/settings` | 硬编码配置 | 完整 |
| Market States | `/states` | 静态数据 | 完整 |

### 5.2 API 对接情况

- 11 个前端 API 方法与后端 11 个端点 **1:1 完全匹配**，无路由不一致
- 仅 4 个页面使用 `useApi` 尝试实时获取数据，其余 9 个使用静态 fallback
- **6 个 API 方法已定义但未被任何页面调用**：`skill(id)`、`riskEvents()`、`executions()`、`positions()`、`runPipeline()`、`evaluateMarket()`

### 5.3 后端已知问题影响前端

- `health` 端点的 `uptime_secs` 始终为 0
- `risk/rules` 返回硬编码数据，非实时
- `evaluate_symbol` 不保存风控/执行结果，相关页面数据不完整

---

## 6. 架构评估

### ✅ 优点

1. **Monorepo 结构清晰** — 11 个 crate 职责分明，协议层冻结
2. **10 大协议全部定义** — CanonicalEvent、SignalEvent、RiskDecision 等
3. **5 阶段 pipeline 完整串联** — 特征→信号→组合→风控→执行
4. **7 条风控规则** — 覆盖杠杆/仓位/敞口/日损/冷却/订单数/单笔风险
5. **前端 13 页全部有实质内容** — 无空壳页面
6. **零编译 warning** — 代码质量基线高
7. **YAML 驱动的 Skill 系统** — 12 个策略可热加载

### ⚠️ 待改进

1. **执行层未接入真实交易所** — 最大缺口，当前全是模拟
2. **消息总线未连通** — NATS 基础设施就绪但未接入
3. **仓位追踪器与风控状态不同步** — 多模块状态不一致
4. **前端 9/13 页面依赖静态数据** — 需逐步接入实时 API

---

## 7. 问题优先级排序

| 优先级 | 编号 | 问题 | 类型 | 状态 |
|--------|------|------|------|------|
| P0 | #1 | 仓位追踪器不同步（exit/reduce/risk_alert） | Bug | ✅ 已修复 |
| P0 | #2 | 风控状态不更新 | Bug | ✅ 已修复 |
| P0 | #3 | NATS pipeline 价格为 0 | Bug | ✅ 已修复 |
| P1 | #4 | 默认数据库凭证硬编码 | 安全 | ✅ 已修复 |
| P1 | #5 | API 密钥未脱敏 | 安全 | ✅ 已修复 |
| P1 | #6 | 市价单绕过风控检查 | 逻辑缺陷 | ✅ 已修复 |
| P1 | #7 | Shrink/Review 判定未处理 | 功能缺失 | ✅ 已修复 |
| P2 | #8 | 交易所执行器未接入 | Stub |
| P2 | #9 | NATS 消息总线未接入 | Stub |
| P2 | #10 | 技能状态未过滤 | 逻辑缺陷 |
| P2 | #11 | 止损/止盈订单未触发 | 未使用代码 |
| P2 | #12 | 相关性风控未执行 | 未使用代码 |
| P3 | #13 | 前端 9 页面使用静态数据 | UX |
| P3 | #14 | 6 个 API 方法未使用 | 未使用代码 |
| P3 | #15 | K线批量插入性能 | 性能 |
| P3 | #16 | output_contract 始终 Null | Minor |

---

## 8. 结论

**项目完成度**: 约 **75%**

- **协议与架构层**: 95%（协议冻结、模块划分、数据流设计）
- **核心逻辑层**: 80%（信号引擎、风控规则、回测引擎均已实现）
- **状态同步层**: 40%（仓位追踪、风控状态、pipeline 间通信存在 bug）
- **执行层**: 20%（全部模拟，无真实交易所接入）
- **前端层**: 70%（页面完整，但多数使用静态数据）
- **安全层**: 60%（无硬编码密钥，但存在凭证泄露风险）

**建议下一里程碑 (M8) 聚焦**:
1. 修复 P0 级 3 个状态同步 bug
2. 修复 P1 级安全问题和逻辑缺陷
3. 接入币安 Testnet 真实交易 API
4. 前端逐步接入实时 API
