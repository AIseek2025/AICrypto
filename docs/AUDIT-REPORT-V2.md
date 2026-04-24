# AICrypto 项目全面自查审计报告 V2

**审计日期**: 2026-04-24  
**审计范围**: 全项目代码库（Rust 后端 11 crates + Next.js 前端 + Python 研究层 + 基础设施）  
**基准版本**: M7-Phase 8 完成后  
**审计方法**: 逐文件源码审查，覆盖安全、正确性、架构完整性、生产就绪度  

---

## 一、项目概况

| 指标 | 数值 |
|------|------|
| Rust 源文件 | ~68 |
| Rust 代码行数 | ~8,100 |
| TypeScript/TSX 文件 | ~19 |
| Rust Crates | 11（含 7 个可执行 binary） |
| 前端页面路由 | 16 |
| API 端点 | 11 |
| YAML 技能 (Skills) | 10 |
| 风控规则 | 7 |
| 协议结构体 | 10 |
| Python 研究模块 | 4（已实现） |
| Adapters (Python 层) | 0（全部为 README 设计稿） |
| Decision (Python 层) | 1/5 已实现 |
| Execution (Python 层) | 0/5 已实现 |
| Intelligence (Python 层) | 4/13 已实现 |

---

## 二、安全审计

### 🔴 CRITICAL — S1: 生产服务器 IP 硬编码于源码

**位置**: `scripts/ecs/deploy.sh:9`, `scripts/ecs/ops.sh:10`, `scripts/ecs/setup-ssl.sh:10`

```bash
REMOTE="admin@8.218.209.218"
DOMAIN="aicrypto.cool"
```

**风险**: 任何获取仓库访问权限的人都能获知生产服务器 IP、SSH 用户名和域名。应使用环境变量或 gitignore 的配置文件。

### 🔴 CRITICAL — S2: CORS 完全开放

**位置**: `crates/api-gateway/src/main.rs:28`

```rust
let app = handlers::router(state.clone()).layer(CorsLayer::permissive());
```

**风险**: 允许任何来源、任何方法、任何头部。生产部署时任何网站均可跨域调用 API。

### 🔴 CRITICAL — S3: 全系统无认证机制

**位置**: 全项目范围

- API Gateway: 无 auth middleware
- 前端: 无 JWT / session / API key
- Ops 工具: 无认证
- 任何人可调用 `POST /api/run-pipeline` 触发完整交易流程

### 🔴 CRITICAL — S4: BinanceConfig 的 Debug trait 泄露 API 密钥

**位置**: `crates/foundation/src/config.rs:37`

`BinanceConfig` derive `Debug`，`api_key`/`api_secret` 为普通 `String`。`SecretString` 类型已定义但**从未使用**。任何 `println!("{:?}", config)` 或日志输出都会泄露密钥。

### 🟡 HIGH — S5: listenKey 明文日志输出

**位置**: `crates/gateway-market/src/binance/user_stream.rs:50`

```rust
tracing::info!(listen_key = %listen_key, "listenKey created");
```

listenKey 是会话令牌，获取日志访问权限即可劫持用户私有数据流。

### 🟡 HIGH — S6: 数据库/Redis 连接无 TLS

**位置**: `crates/foundation/src/storage.rs:15`

`PgPool::connect()` 未配置 TLS。`Redis` 使用 `redis://` 而非 `rediss://`。非本地部署时凭证明文传输。

### 🟡 HIGH — S7: Docker Compose 硬编码数据库凭证

**位置**: `infra/docker/docker-compose.yml:8-9`

```yaml
POSTGRES_USER: aicrypto
POSTGRES_PASSWORD: aicrypto
```

生产环境模板（`local/env/README.md`）同样使用弱密码。

### 🟡 HIGH — S8: 前端 Settings 页面暴露内部架构

**位置**: `apps/console-web/src/app/settings/page.tsx:5-30`

客户端 bundle 中暴露 Binance testnet URL、数据库连接模式、NATS 地址、全部风控阈值。

### 🟡 MEDIUM — S9: 前端默认使用 HTTP 明文

**位置**: `apps/console-web/src/lib/api.ts:1-2`

```ts
const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";
```

若环境变量未设置，所有 API 流量（含仓位、PnL）以明文传输。

### 🟢 OK — 无硬编码 API 密钥

所有密钥均从环境变量读取。`.env.example` 中 API key 字段为空。`.gitignore` 正确排除 `.env`、`*.pem`、`*.key`。

---

## 三、正确性审计（Bug 清单）

### 🔴 CRITICAL — B1: 回测引擎双重佣金扣除

**位置**: `crates/backtest-engine/src/engine.rs:94` + `crates/backtest-engine/src/metrics.rs:84`

`pos.close()` 已扣除佣金返回 PnL，但 `BacktestMetrics::compute` 在构建权益曲线时再次扣除 `t.commission`。**所有回测的净 PnL、权益曲线、最大回撤均不正确。**

### 🔴 CRITICAL — B2: 回测 SMA 实现为累积平均（非滚动窗口）

**位置**: `crates/backtest-engine/src/strategy.rs:146-150`

`sma_sum` 只累加不移除旧值。超过 SMA 周期后，"SMA" 变为全部历史的累积平均，信号越来越迟钝。

### 🔴 CRITICAL — B3: 回测平仓滑点方向反转

**位置**: `crates/backtest-engine/src/engine.rs:92`

平多仓使用买入方向滑点（价格上滑），应为卖出方向（价格下滑）。平空仓同理反转。**所有回测的模拟成交价不正确。**

### 🔴 CRITICAL — B4: 风控 reduce_only 绕过全部 7 条规则

**位置**: `crates/risk-engine/src/evaluator.rs:37-50`

任何 `reduce_only = true` 的订单自动通过 `RiskVerdict::Allow`，不验证是否持有仓位或订单数量是否超出仓位。**恶意订单可绕过全部风控。**

### 🔴 CRITICAL — B5: 风控 leverage_hint 缺失时绕过杠杆检查

**位置**: `crates/risk-engine/src/rules.rs:68`

```rust
if let Some(lev) = intent.leverage_hint { ... }
```

`None` 时杠杆检查被完全跳过。

### 🔴 CRITICAL — B6: 市价单名义价值使用任意价格 `equity * 0.01`

**位置**: `crates/risk-engine/src/rules.rs:87-88, 112-113, 185-186`

市价单 `price_limit` 为 `None` 时，名义价值 = `qty * equity * 0.01`，完全偏离实际市场价格。**10 BTC 的名义价值被计算为 $10,000 而非 ~$650,000。**

### 🔴 CRITICAL — B7: 负数数量/价格绕过风控检查

**位置**: `crates/risk-engine/src/rules.rs:83`

`quantity.parse::<f64>()` 接受负值，使名义价值为负，绕过 R002/R003/R007。

### 🔴 CRITICAL — B8: Short 仓位退出生成错误方向

**位置**: `crates/portfolio-engine/src/order_builder.rs:32-38`

平空仓的退出信号映射为 `Side::SELL`，应为 `Side::BUY`。平仓订单将执行为开空而非平空。

### 🔴 CRITICAL — B9: 追踪器在订单成交前更新状态

**位置**: `crates/portfolio-engine/src/portfolio.rs:111, 125, 139, 152, 164`

所有 handler 立即更新 tracker 后返回 OrderIntent。若交易所拒绝，tracker 状态永久不一致。

### 🔴 CRITICAL — B10: 订单状态机不支持连续部分成交

**位置**: `crates/gateway-trading/src/order_state_machine.rs:82`

`PartiallyFilled → PartiallyFilled` 不是合法转换。第二次部分成交将导致错误。

### 🔴 CRITICAL — B11: Draft/Disabled 技能参与信号评估

**位置**: `crates/signal-runtime/src/skill_registry.rs:160-167`

`find_by_market_state()` 和评估管线无状态过滤。未测试或已禁用的技能可发出信号。

### 🔴 CRITICAL — B12: NATS 管线忽略风控裁决

**位置**: `crates/pipeline-integration/src/pipeline.rs:293`

NATS 路径对 Deny/Shrink/Review 裁决均执行 `executor.submit()`，风控形同虚设。

### 🟡 HIGH — B13: 交易所执行器未接入真实 API

**位置**: `crates/gateway-trading/src/executor.rs:56-61`

`dry_run = false` 路径仍调用 `simulate_execution()`。无任何 HTTP 请求。Live 模式是空壳。

### 🟡 HIGH — B14: handle_add 绕过敞口限制

**位置**: `crates/portfolio-engine/src/portfolio.rs:129-142`

`handle_entry` 检查 `total_exposure() >= max_notional`，但 `handle_add` 不检查。加仓可突破总敞口限制。

### 🟡 HIGH — B15: mark_price 从未更新

**位置**: `crates/portfolio-engine/src/position_tracker.rs:52-59`

`update_mark_price()` 存在但从未被调用。所有敞口、未实现 PnL、保证金计算使用入场价，越来越不准确。

### 🟡 HIGH — B16: "eq"/"ne" 比较运算符使用 f64::EPSILON 容差

**位置**: `crates/signal-runtime/src/skill_registry.rs:225-226`

`f64::EPSILON ≈ 2.2e-16`，实际浮点数据几乎不可能匹配 "eq"。`"ne"` 几乎永远为 true。

### 🟡 HIGH — B17: listenKey 无自动续期

**位置**: `crates/gateway-market/src/binance/user_stream.rs:55-77`

Binance listenKey 60 分钟过期。`keepalive()` 方法存在但无定时调用。用户数据流会静默中断。

### 🟡 HIGH — B18: 对账从未执行

**位置**: `crates/gateway-trading/src/order_state_machine.rs:103`

`ReconcileState::Pending` 在成交时设置，但 `Reconciled` 事件从未触发。对账流程形同虚设。

### 🟡 HIGH — B19: 无止损/止盈订单生成

**位置**: `crates/portfolio-engine/src/order_builder.rs:76-136`

`build_stop_loss()` 和 `build_take_profit()` 已实现但从未被调用。尽管仓位计算依赖止损距离，但无保护性止损单。

### 🟡 MEDIUM — B20: 回测 Sharpe 比率使用错误周期

**位置**: `crates/backtest-engine/src/metrics.rs:93, 106`

使用每笔交易收益作为"日收益"，以 `√252` 年化。应为日收益或调整年化因子。

### 🟡 MEDIUM — B21: profit_factor 返回 INFINITY

**位置**: `crates/backtest-engine/src/metrics.rs:75`

`gross_loss = 0` 时返回 `f64::INFINITY`，JSON 序列化为 `null`，`{:.2}` 格式化为 "inf"。

### 🟡 MEDIUM — B22: API 历史记录无界增长导致 OOM

**位置**: `crates/api-gateway/src/handlers.rs:21-23`

`signal_history`、`risk_history`、`execution_history` 为无界 Vec。重复调用 pipeline 导致内存无限增长。

### 🟡 MEDIUM — B23: 硬编码杠杆和缩减比例

**位置**: `crates/portfolio-engine/src/portfolio.rs:96, 150, 162`

`handle_entry` 硬编码杠杆 3（忽略配置），`handle_reduce`/`handle_risk_alert` 硬编码 50% 缩减（忽略 confidence）。

### 🟡 MEDIUM — B24: 协议层零验证

**位置**: `crates/protocols/src/` 全部 10 个文件

所有协议结构体无任何字段验证：`quantity` 为 String 接受任意值、`confidence` 无 `[0,1]` 范围检查、`leverage_hint` 无上界、无 `deny_unknown_fields`。

### 🟡 MEDIUM — B25: 相关性风控未实现

**位置**: `crates/risk-engine/src/rules.rs:13`

`max_correlated_exposure_pct` 配置字段存在但无检查逻辑。

---

## 四、架构完整性评估

### 4.1 Rust 核心层

| 层级 | 完成度 | 说明 |
|------|--------|------|
| 协议层 (`protocols`) | 🟢 95% | 10 个核心协议全部定义，但缺少 PositionState、AccountSnapshot、TradeFill |
| 基础层 (`foundation`) | 🟡 75% | 配置/日志/存储可用，SecretString 未使用，DB 连接池配置未生效 |
| 信号引擎 (`signal-runtime`) | 🟡 70% | YAML 加载+评估完整，但状态过滤和比较运算符有严重 bug |
| 特征引擎 (`feature-engine`) | 🟢 90% | 11 个指标正确实现，compute O(n²) 性能待优化 |
| 风控引擎 (`risk-engine`) | 🔴 40% | 7 条规则定义完整，但状态不更新、reduce_only 绕过、市价单绕过 |
| 组合引擎 (`portfolio-engine`) | 🟡 55% | 仓位追踪框架在，但 short 方向错误、先改状态后下单、SL/TP 未接入 |
| 回测引擎 (`backtest-engine`) | 🔴 45% | 框架完整但双重佣金、SMA 错误、滑点反转导致结果不可信 |
| 交易网关 (`gateway-trading`) | 🔴 25% | 状态机定义完整但执行器全模拟、部分成交不支持、对账未执行 |
| 行情网关 (`gateway-market`) | 🟡 65% | REST/WS 可用，但批量插入伪实现、listenKey 无续期、部分数据硬编码 |
| API 网关 (`api-gateway`) | 🟡 60% | 11 端点可用，但 CORS 开放、无认证、无输入验证、历史无界增长 |
| Pipeline (`pipeline-integration`) | 🟡 65% | 进程内管线正确，但 NATS 路径忽略风控、RiskState 部分字段不更新 |
| 模拟交易 (`sim-trading`) | 🟡 55% | 自动交易循环可用，但无资金检查、Mutex unwrap 风险、Shrink 空操作 |

### 4.2 Python 层

| 层级 | 完成度 | 说明 |
|------|--------|------|
| Adapters | 🔴 0% | 全部 9 个子目录为 README 设计稿，零实现 |
| Decision | 🔴 20% | 仅 allocator 已实现；risk_gateway（最关键）为空 |
| Execution | 🔴 0% | 全部 5 个子目录为 README 设计稿 |
| Intelligence | 🟡 31% | agents/framework、evaluator、correlations、kg 已实现 |
| Research | 🟡 30% | labeling 可用，features/experiments/notebooks 为空 |

### 4.3 前端

| 指标 | 状态 | 说明 |
|------|------|------|
| 页面完整性 | 🟢 | 16 页全部有实质内容 |
| API 对接 | 🟡 | 仅 4 页使用实时 API，12 页依赖静态数据 |
| 类型安全 | 🔴 | 40+ 处 `any` 类型，绕过全部 TypeScript 检查 |
| 认证 | 🔴 | 零认证机制 |
| 错误处理 | 🔴 | 无 Error Boundary，错误静默吞没 |

---

## 五、代码质量

### 5.1 编译状态

| 指标 | 状态 |
|------|------|
| `cargo check` | ✅ 零 warning |
| `cargo build` | ✅ 零 error |
| `next build` | ✅ 零错误 |
| TypeScript 检查 | ✅ 通过（但大量 any） |

### 5.2 unwrap/panic 使用

| 位置 | 模式 | 风险 |
|------|------|------|
| sim-trading main.rs (10 处) | `Mutex::lock().unwrap()` | 🔴 锁中毒时 panic，单线程失败级联到整个系统 |
| gateway-trading executor.rs:63 | `.unwrap()` | 🟠 订单创建与获取间可能不一致 |
| gateway-market rest.rs:18, user_stream.rs:19 | `.expect()` | 🟢 启动时 panic，可接受 |
| 其余生产代码 | `unwrap_or`/`unwrap_or_default` | 🟢 安全降级 |

### 5.3 代码规范

- ✅ 零 TODO/FIXME/HACK 注释
- ✅ 统一使用 `anyhow::Result` 错误处理
- ✅ 一致的 serde 序列化
- ⚠️ 4 个 crate 的 `main.rs` 使用 `#![allow(unreachable_code)]` 隐藏警告

---

## 六、生产就绪度评估

### 6.1 阻断项（必须修复才能上线）

| 编号 | 阻断项 | 影响 |
|------|--------|------|
| B4 | reduce_only 绕过全部风控 | 恶意订单无阻碍通过 |
| B5 | leverage_hint 缺失绕过杠杆检查 | 无限制杠杆 |
| B7 | 负数数量绕过风控 | 风控计算完全失效 |
| B8 | Short 退出方向错误 | 平仓订单执行为开仓 |
| B11 | Draft 技能发出信号 | 未测试策略产生实盘交易 |
| B12 | NATS 管线忽略风控 | Deny 订单仍被执行 |
| S1 | 生产 IP 暴露于源码 | 安全攻击面 |
| S2 | CORS 完全开放 | 跨站攻击 |
| S3 | 全系统无认证 | 任何人可触发交易 |
| S4 | API 密钥可泄露 | Debug 输出暴露密钥 |

### 6.2 完成度评分

| 维度 | 上次审计 | 本次审计 | 变化 |
|------|----------|----------|------|
| 协议与架构层 | 95% | 95% | → |
| 核心逻辑层 | 80% | 70% | ↓ (发现更多 bug) |
| 风控正确性 | 40% | 25% | ↓ (发现绕过漏洞) |
| 回测正确性 | N/A | 35% | 新评 |
| 执行层 | 20% | 20% | → |
| 状态同步 | 40% | 50% | ↑ (进程内管线已修复) |
| 前端层 | 70% | 65% | ↓ (发现安全问题) |
| 安全层 | 60% | 35% | ↓ (发现更多泄露) |
| Python 层 | N/A | 12% | 新评 |

**综合完成度**: ~45%（上次评估 75%，本次发现大量隐藏 bug 后下调）

---

## 七、问题优先级排序

### P0 — 立即修复（阻断上线）

| # | 问题 | 类型 | 位置 |
|---|------|------|------|
| 1 | 回测双重佣金扣除 | Bug | `backtest-engine/engine.rs:94` + `metrics.rs:84` |
| 2 | 回测 SMA 累积平均 | Bug | `backtest-engine/strategy.rs:146-150` |
| 3 | 回测滑点方向反转 | Bug | `backtest-engine/engine.rs:92` |
| 4 | reduce_only 绕过全部风控 | 安全 | `risk-engine/evaluator.rs:37-50` |
| 5 | 市价单名义价值使用 equity*0.01 | Bug | `risk-engine/rules.rs:87-88` |
| 6 | 负数数量绕过风控 | 安全 | `risk-engine/rules.rs:83` |
| 7 | Short 退出方向错误 | Bug | `portfolio-engine/order_builder.rs:32-38` |
| 8 | Draft 技能参与评估 | Bug | `signal-runtime/skill_registry.rs:160-167` |
| 9 | NATS 管线忽略风控裁决 | Bug | `pipeline-integration/pipeline.rs:293` |
| 10 | 追踪器先改状态后下单 | Bug | `portfolio-engine/portfolio.rs:111` |
| 11 | 订单状态机不支持连续部分成交 | Bug | `gateway-trading/order_state_machine.rs:82` |

### P1 — 高优先级（下一迭代修复）

| # | 问题 | 类型 | 位置 |
|---|------|------|------|
| 12 | BinanceConfig Debug 泄露密钥 | 安全 | `foundation/config.rs:37` |
| 13 | CORS 完全开放 | 安全 | `api-gateway/main.rs:28` |
| 14 | 全系统无认证 | 安全 | 全局 |
| 15 | 生产 IP 硬编码于脚本 | 安全 | `scripts/ecs/deploy.sh:9` |
| 16 | listenKey 明文日志 | 安全 | `gateway-market/user_stream.rs:50` |
| 17 | leverage_hint 缺失绕过检查 | Bug | `risk-engine/rules.rs:68` |
| 18 | handle_add 绕过敞口限制 | Bug | `portfolio-engine/portfolio.rs:129` |
| 19 | mark_price 从未更新 | Bug | `portfolio-engine/position_tracker.rs:52` |
| 20 | "eq"/"ne" 运算符 f64::EPSILON | Bug | `signal-runtime/skill_registry.rs:225` |
| 21 | listenKey 无自动续期 | Bug | `gateway-market/user_stream.rs:55-77` |
| 22 | 无止损/止盈订单 | 功能缺失 | `portfolio-engine/order_builder.rs:76` |
| 23 | 对账从未执行 | Bug | `gateway-trading/order_state_machine.rs:103` |
| 24 | 前端无认证 | 安全 | `console-web/src/lib/api.ts` |
| 25 | Settings 页面暴露架构 | 安全 | `console-web/src/app/settings/page.tsx` |

### P2 — 中优先级

| # | 问题 | 类型 | 位置 |
|---|------|------|------|
| 26 | 交易所执行器全模拟 | Stub | `gateway-trading/executor.rs:56` |
| 27 | NATS 消息总线未连通 | Stub | `pipeline-integration/pipeline.rs` |
| 28 | DB 批量插入伪实现 | 性能 | `gateway-market/persistence/db.rs:154` |
| 29 | API 历史无界增长 OOM | Bug | `api-gateway/handlers.rs:21-23` |
| 30 | 协议层零验证 | 设计 | `protocols/src/` 全部 |
| 31 | 相关性风控未实现 | 功能缺失 | `risk-engine/rules.rs:13` |
| 32 | 回测 Sharpe 周期错误 | Bug | `backtest-engine/metrics.rs:93` |
| 33 | profit_factor INFINITY | Bug | `backtest-engine/metrics.rs:75` |
| 34 | 硬编码杠杆和缩减比例 | Bug | `portfolio-engine/portfolio.rs:96,150` |
| 35 | DB 连接池配置未生效 | Bug | `foundation/storage.rs:14` |
| 36 | 无请求体大小限制 | 安全 | `api-gateway/main.rs` |
| 37 | sim-trading Mutex unwrap | Bug | `sim-trading/main.rs` (10处) |
| 38 | RiskState daily_pnl 等字段从不更新 | Bug | `pipeline-integration/pipeline.rs:204` |
| 39 | Docker Compose 弱密码 | 安全 | `infra/docker/docker-compose.yml:8` |
| 40 | DB/Redis 无 TLS | 安全 | `foundation/storage.rs:15` |

### P3 — 低优先级

| # | 问题 | 类型 | 位置 |
|---|------|------|------|
| 41 | 前端 12/16 页使用静态数据 | UX | `console-web/src/` |
| 42 | 40+ 处 `any` 类型 | 代码质量 | `console-web/src/` |
| 43 | 无 Error Boundary | 代码质量 | `console-web/src/app/layout.tsx` |
| 44 | K线 taker_buy_volume 硬编码 "0" | 数据质量 | `gateway-market/main.rs:101` |
| 45 | Mark price 时间戳用本地时钟 | 数据质量 | `gateway-market/main.rs:109` |
| 46 | 6 个 API 方法未使用 | 未使用代码 | `console-web/src/lib/api.ts` |
| 47 | output_contract 始终 Null | Minor | `signal-runtime/skill_registry.rs:134` |
| 48 | 回测无清算检查 | 逻辑缺失 | `backtest-engine/engine.rs` |
| 49 | 回测 look-ahead bias | 逻辑 | `backtest-engine/engine.rs:62-68` |
| 50 | compute_features_for_candles O(n²) | 性能 | `feature-engine/compute.rs:128` |
| 51 | Python 层全部 adapter/execution 为空 | 完成度 | `adapters/`, `execution/` |

---

## 八、与上次审计对比

| 维度 | 上次评估 | 本次发现 | 说明 |
|------|----------|----------|------|
| P0 Bug | 3 个（已标记修复） | **11 个** | 上次修复了状态同步，但深层 bug 本次大量暴露 |
| 安全问题 | 3 个 | **10 个** | 新发现生产 IP 泄露、CORS、无认证等 |
| 风控有效性 | "7 条规则覆盖" | **25% 有效** | reduce_only 绕过、市价单绕过、负值绕过 |
| 回测可信度 | "胜率 96.9%" | **不可信** | 双重佣金、SMA 错误、滑点反转 |
| 执行层 | "20% 全模拟" | **20% 不变** | 仍无真实交易所连接 |
| Python 层 | 未评估 | **12%** | 大部分为设计稿 |
| 综合完成度 | 75% | **45%** | 深入审查发现更多问题 |

### 上次 P0 修复验证

| 问题 | 上次状态 | 本次验证 |
|------|----------|----------|
| 仓位追踪器不同步 | ✅ 已修复 | ✅ exit/reduce/risk_alert 均调用 reduce_position |
| 风控状态不更新 | ✅ 已修复 | ✅ 进程内管线已同步（NATS 路径仍未同步） |
| NATS pipeline 价格为 0 | ✅ 已修复 | ✅ 进程内路径从参数获取，NATS 有 fallback+guard |

---

## 九、结论与建议

### 9.1 项目定位

AICrypto 当前是一个**架构清晰、模块完整的原型/演示系统**。协议设计严谨，分层合理，YAML 驱动的 Skill 系统灵活。但**核心逻辑层存在多个严重 bug，安全层存在重大缺陷，不可用于任何真实资金环境。**

### 9.2 建议下一里程碑 (M8) 聚焦

**第一阶段：修复核心正确性（2-3 周）**
1. 修复回测引擎 3 个 P0 bug（双重佣金、SMA、滑点方向）
2. 修复风控引擎 4 个绕过漏洞（reduce_only、leverage_hint、负值、市价单）
3. 修复 Short 退出方向和 Draft 技能过滤
4. 修复订单状态机部分成交

**第二阶段：安全加固（1-2 周）**
5. 实现 API 认证（JWT 或 API key）
6. 收紧 CORS、移除生产 IP 硬编码
7. 将 BinanceConfig 的 api_key/api_secret 改为 SecretString
8. 移除 listenKey 日志输出

**第三阶段：执行层接入（3-4 周）**
9. 接入币安 Testnet 真实交易 API
10. 实现止损/止盈订单生成
11. 实现对账循环
12. 实现 NATS 管线风控分支

### 9.3 关键教训

1. **"编译通过 ≠ 正确"** — 零 warning 的代码仍可能包含严重逻辑错误
2. **风控必须纵深防御** — 单层检查易被绕过，需协议层验证 + 引擎层检查 + 执行层兜底
3. **回测结果需独立验证** — 本次发现的 3 个回测 bug 使此前所有回测结论不可信
4. **安全不是附加层** — CORS、认证、密钥管理应从第一天纳入，而非事后修补
