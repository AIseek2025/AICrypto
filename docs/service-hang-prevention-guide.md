# AICrypto 服务进程卡死问题 — 根因分析与永久解决方案

> **文档版本**: v2.0
> **日期**: 2026-04-24
> **适用范围**: AICrypto 项目所有 Rust 服务型 binary
> **状态**: ✅ 已实施并验证

---

## 一、问题总结

### 1.1 现象

在 AI 辅助开发工具（Crush / Claude Code）的 bash 终端中运行 Rust 服务 binary 时，命令长期无响应，表现为"卡住"或"死循环"。开发者被迫中断会话、重启对话，严重影响开发效率。

### 1.2 历史发生记录

| 日期 | Binary | 现象 | 当次解决方式 |
|------|--------|------|-------------|
| 2026-04-24 | signal-runtime | sleep+kill 无效，进程被自动移到后台 | 改用输出重定向到文件 |
| 2026-04-24 | portfolio-engine | 同上 | 同上 |
| 2026-04-24 | risk-engine | 输出重定向到文件 | ✅ 成功 |
| 2026-04-24 | gateway-trading | 输出重定向到文件 | ✅ 成功 |
| 2026-04-24 | gateway-market | ctrl_c().await 阻塞 | 手动 job_kill |
| 2026-04-24 | api-gateway | axum::serve() 永久阻塞 | 手动 job_kill |

### 1.3 影响范围

**9 个 binary 中有 7 个受影响**：

| Binary | 阻塞模式 | 严重度 |
|--------|---------|--------|
| signal-runtime | `loop { interval.tick().await }` | 🔴 高 |
| portfolio-engine | `loop { interval.tick().await }` | 🔴 高 |
| risk-engine | `loop { interval.tick().await }` | 🔴 高 |
| gateway-trading | `loop { interval.tick().await }` | 🔴 高 |
| gateway-market | `tokio::signal::ctrl_c().await` | 🔴 高 |
| api-gateway | `axum::serve(listener, app).await` | 🔴 高 |
| feature-engine | `tokio::signal::ctrl_c().await` + spawned `loop {}` | 🔴 高 |
| pipeline-integration | 无（正常退出） | ✅ 无 |
| backtest-engine | 无（正常退出） | ✅ 无 |

---

## 二、根因分析

### 2.1 架构层面：服务型 binary 设计为永不退出

所有服务 binary 在 demo 逻辑执行完后，会进入无限等待状态：

**模式 A — Heartbeat 循环**（4 个 binary）:
```rust
let mut interval = tokio::time::interval(Duration::from_secs(60));
loop {
    interval.tick().await;
    tracing::debug!("heartbeat");
}
```

**模式 B — Ctrl+C 阻塞**（2 个 binary）:
```rust
tokio::signal::ctrl_c().await?;
```

**模式 C — HTTP Serve**（1 个 binary）:
```rust
axum::serve(listener, app).await?;
```

这些是**正确的生产环境设计**——服务应持续运行直到收到停止信号。但在开发调试场景下，这些 binary **永远不会主动退出**。

### 2.2 环境层面：macOS 缺少 `timeout` 命令

| 限制 | 说明 |
|------|------|
| `timeout` 命令不可用 | macOS 默认不安装 GNU coreutils，无法用 `timeout 10 cargo run ...` |
| `gtimeout` 需额外安装 | Homebrew 安装 `coreutils` 后可用，但非默认环境 |
| `sleep + kill` 不可靠 | 在 AI 工具的受限 shell 中 `$!` 可能无法正确捕获 PID |

### 2.3 工具层面：AI 终端的 bash 超时机制

AI 辅助开发工具（Crush / Claude Code）的 bash 工具有以下行为：

1. 命令执行超过 **60 秒**后自动移到后台
2. 返回一个 shell ID，需要用 `job_output` 读取
3. `job_output(wait=true)` 对仍在运行的进程**也会阻塞**
4. 必须先用 `job_kill` 终止，再读取输出

### 2.4 连锁效应

```
运行 demo binary
  → 进程永不退出
    → bash 工具 60s 超时
      → 自动移到后台
        → AI agent 尝试读取输出
          → job_output 阻塞（进程仍在运行）
            → agent 超时 / 卡死
              → 整个开发会话中断
```

---

## 三、永久解决方案：条件编译（已实施）

### 3.1 方案原理

在所有受影响的 binary 中引入 Cargo feature `server`，通过 `#[cfg(feature = "server")]` 控制是否进入无限等待。

```
默认模式（生产）: cargo run --bin signal-runtime
  → 含 server feature → 执行 demo + 进入 idle 循环

调试模式（开发）: cargo run --bin signal-runtime --no-default-features
  → 不含 server feature → 执行 demo 后立即退出
```

### 3.2 代码变更

#### 变更 A：Cargo.toml 添加 feature 声明

每个受影响的 crate 的 `Cargo.toml` 添加：

```toml
[features]
default = ["server"]
server = []
```

**涉及文件**（7 个）：
- `crates/signal-runtime/Cargo.toml`
- `crates/portfolio-engine/Cargo.toml`
- `crates/risk-engine/Cargo.toml`
- `crates/gateway-trading/Cargo.toml`
- `crates/gateway-market/Cargo.toml`
- `crates/api-gateway/Cargo.toml`
- `crates/feature-engine/Cargo.toml`

#### 变更 B：main.rs 中用 cfg 控制 idle 逻辑

**模式 A 的 binary**（signal-runtime / portfolio-engine / risk-engine / gateway-trading）：

```rust
// 之前（会卡死）:
tracing::info!("demo complete — entering idle mode (ctrl+c to stop)");
let mut interval = tokio::time::interval(Duration::from_secs(60));
loop {
    interval.tick().await;
    tracing::debug!("heartbeat");
}

// 之后（开发模式自动退出）:
tracing::info!("demo complete");

#[cfg(feature = "server")]
{
    tracing::info!("entering idle mode (ctrl+c to stop)");
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        tracing::debug!("heartbeat");
    }
}

Ok(())
```

**模式 B 的 binary**（gateway-market）：

```rust
// 之前:
tracing::info!("fully operational — press Ctrl+C to stop");
tokio::signal::ctrl_c().await?;
// cleanup...

// 之后:
#[cfg(feature = "server")]
{
    tracing::info!("fully operational — press Ctrl+C to stop");
    tokio::signal::ctrl_c().await?;
}

#[cfg(not(feature = "server"))]
tracing::info!("demo complete — shutting down");

// cleanup（两种模式都执行）...
```

**模式 C 的 binary**（api-gateway）：

```rust
// 之前:
let listener = tokio::net::TcpListener::bind(...).await?;
tracing::info!("listening on ...");
axum::serve(listener, app).await?;

// 之后:
#[cfg(feature = "server")]
let listener = tokio::net::TcpListener::bind(...).await?;

#[cfg(feature = "server")]
{
    tracing::info!("listening on ...");
    axum::serve(listener, app).await?;
}

#[cfg(not(feature = "server"))]
tracing::info!("initialized (demo mode — not serving)");
```

**feature-engine**（spawned loop + ctrl_c）：

```rust
// spawn 和 ctrl_c 都用 cfg 包裹
#[cfg(feature = "server")]
let db_bg = db.clone();
#[cfg(feature = "server")]
let symbols_bg = symbols.clone();

#[cfg(feature = "server")]
tokio::spawn(async move { /* periodic loop */ });

#[cfg(feature = "server")]
{
    tracing::info!("fully operational — press Ctrl+C to stop");
    tokio::signal::ctrl_c().await?;
}

#[cfg(not(feature = "server"))]
tracing::info!("demo complete");
```

### 3.3 便捷脚本

已创建 `scripts/run-demo.sh`，一键运行 demo 模式：

```bash
# 运行单个 binary（自动退出）
./scripts/run-demo.sh signal-runtime

# 运行所有 demo
./scripts/run-demo.sh signal-runtime
./scripts/run-demo.sh portfolio-engine
./scripts/run-demo.sh risk-engine
./scripts/run-demo.sh gateway-trading
```

---

## 四、操作指引

### 4.1 开发调试时的标准操作

#### 方式一：使用便捷脚本（推荐）

```bash
cd /Users/surferboy/AICrypto
./scripts/run-demo.sh signal-runtime
```

脚本会自动：
1. 以 `--no-default-features` 编译运行
2. 捕获所有输出到 `/tmp/aicrypto-<name>-demo.log`
3. 显示 INFO/WARN/ERROR 级别日志
4. 进程正常退出，不会卡住

#### 方式二：直接 cargo 命令

```bash
# 开发调试（demo 模式，执行完自动退出）
cargo run --bin signal-runtime --no-default-features

# 生产运行（服务模式，持续运行直到 ctrl+c）
cargo run --bin signal-runtime
```

#### 方式三：在 AI 终端中运行

在 Crush / Claude Code 等工具中，使用以下命令：

```
# 不会再卡住
cargo run --bin signal-runtime --no-default-features
```

如果必须测试服务模式（含 idle 循环），使用后台运行：

```bash
cargo run --bin signal-runtime > /tmp/sr.log 2>&1 &
PID=$!
sleep 5
kill $PID 2>/dev/null
cat /tmp/sr.log | grep -E "INFO|WARN|ERROR"
```

### 4.2 新增 binary 时的规范

**规则**：任何包含 `loop {}`、`ctrl_c().await`、`serve().await` 的 binary **必须**使用条件编译。

步骤：

1. 在 crate 的 `Cargo.toml` 添加：
   ```toml
   [features]
   default = ["server"]
   server = []
   ```

2. 在 `main.rs` 中将阻塞逻辑用 `#[cfg(feature = "server")]` 包裹

3. 确保 `--no-default-features` 下 demo 执行完能正常退出

### 4.3 如果还是卡住了（急救流程）

1. **确认 shell ID**（从错误信息或 AI 工具提示中获取）
2. **终止进程**：`job_kill <shell_id>` 或 `kill <PID>`
3. **清理所有残留进程**：
   ```bash
   pkill -f "target/debug/signal-runtime" 2>/dev/null
   pkill -f "target/debug/portfolio-engine" 2>/dev/null
   pkill -f "target/debug/risk-engine" 2>/dev/null
   pkill -f "target/debug/gateway-trading" 2>/dev/null
   pkill -f "target/debug/gateway-market" 2>/dev/null
   pkill -f "target/debug/api-gateway" 2>/dev/null
   pkill -f "target/debug/feature-engine" 2>/dev/null
   lsof -ti:8080 -ti:8090 | xargs kill -9 2>/dev/null
   ```
4. **用 `--no-default-features` 重新运行**

### 4.4 CI/CD 中的使用

```yaml
# CI 中统一使用 demo 模式（不会卡住）
- name: Run demos
  run: |
    cargo run --bin signal-runtime --no-default-features
    cargo run --bin portfolio-engine --no-default-features
    cargo run --bin risk-engine --no-default-features
```

---

## 五、验证清单

以下检查项确认方案已正确实施：

- [x] 7 个受影响 crate 的 `Cargo.toml` 已添加 `[features]`
- [x] 7 个 `main.rs` 已添加 `#[cfg(feature = "server")]` 条件编译
- [x] `cargo check`（默认 server feature）编译通过
- [x] `cargo check -p <crate> --no-default-features` 编译通过（全部 7 个）
- [x] `scripts/run-demo.sh` 已创建并设为可执行
- [x] 2 个无需修改的 binary（pipeline-integration、backtest-engine）未受影响

---

## 六、变更文件清单

| 文件 | 变更类型 | 说明 |
|------|---------|------|
| `crates/signal-runtime/Cargo.toml` | 修改 | 添加 `[features]` |
| `crates/signal-runtime/src/main.rs` | 修改 | cfg 控制 idle 循环 |
| `crates/portfolio-engine/Cargo.toml` | 修改 | 添加 `[features]` |
| `crates/portfolio-engine/src/main.rs` | 修改 | cfg 控制 idle 循环 |
| `crates/risk-engine/Cargo.toml` | 修改 | 添加 `[features]` |
| `crates/risk-engine/src/main.rs` | 修改 | cfg 控制 idle 循环 |
| `crates/gateway-trading/Cargo.toml` | 修改 | 添加 `[features]` |
| `crates/gateway-trading/src/main.rs` | 修改 | cfg 控制 idle 循环 |
| `crates/gateway-market/Cargo.toml` | 修改 | 添加 `[features]` |
| `crates/gateway-market/src/main.rs` | 修改 | cfg 控制 ctrl_c 阻塞 |
| `crates/api-gateway/Cargo.toml` | 修改 | 添加 `[features]` |
| `crates/api-gateway/src/main.rs` | 修改 | cfg 控制 axum serve |
| `crates/feature-engine/Cargo.toml` | 修改 | 添加 `[features]` |
| `crates/feature-engine/src/main.rs` | 修改 | cfg 控制 spawn + ctrl_c |
| `scripts/run-demo.sh` | 新增 | 便捷 demo 运行脚本 |
| `docs/troubleshooting/demo-binary-hang-solution.md` | 已有 | v1 文档（保留参考） |
| `docs/service-hang-prevention-guide.md` | 新增 | 本文档 |

---

## 七、速查表

```
┌─────────────────────────────────────────────────────────┐
│                  AICrypto Demo 速查表                     │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  开发调试（自动退出）:                                    │
│    cargo run --bin <name> --no-default-features         │
│                                                         │
│  便捷脚本:                                               │
│    ./scripts/run-demo.sh <name>                         │
│                                                         │
│  生产运行（持续服务）:                                    │
│    cargo run --bin <name>                               │
│                                                         │
│  卡住急救:                                               │
│    job_kill <id> 或 pkill -f "target/debug/<name>"      │
│                                                         │
│  新 binary 规范:                                         │
│    Cargo.toml 加 [features] default=["server"] server=[]│
│    main.rs 加 #[cfg(feature = "server")] 包裹阻塞逻辑    │
│                                                         │
└─────────────────────────────────────────────────────────┘
```
