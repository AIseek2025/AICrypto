# Demo Binary 卡住问题分析与解决方案

> ⚠️ **本文档已归档**。永久解决方案已实施，详见 [service-hang-prevention-guide.md](../service-hang-prevention-guide.md)。
>
> 本文档保留作为历史参考，记录了 2026-04-24 的问题分析和临时解决方案。

## 问题现象

运行 Rust 服务型 demo binary 时（如 `cargo run --bin signal-runtime`），命令长期无响应，表现为"卡住"或"死循环"。

## 根因分析

### 1. 架构设计：服务型 binary 包含 idle 循环

所有 Rust 服务 binary（signal-runtime、portfolio-engine、risk-engine、gateway-trading）在 demo 执行完毕后，会进入一个无限等待循环：

```rust
let mut interval = tokio::time::interval(Duration::from_secs(60));
loop {
    interval.tick().await;
    tracing::debug!("heartbeat");
}
```

这是**设计意图**——生产环境中服务应持续运行直到收到 ctrl+c 信号。但在开发调试场景下，这会导致进程永不退出。

### 2. Shell 执行环境限制

当前执行环境中存在两个限制：

| 限制 | 说明 |
|------|------|
| `timeout` 命令不可用 | macOS 默认不安装 GNU coreutils 的 `timeout`，`timeout 10 cargo run ...` 会报 command not found |
| `sleep + kill` 不可靠 | 后台进程管理在当前 shell 环境中行为不稳定，`$!` 可能无法正确捕获 PID，导致 kill 无效 |

### 3. 工具层面

Bash 工具在命令执行超过 60 秒后，会自动将命令移到后台并返回一个 shell ID。此时：
- 无法再通过前台方式获取输出
- 必须使用 `job_output` 工具读取
- 进程仍在运行，需要 `job_kill` 手动终止

## 受影响的命令模式

以下三种模式在当前环境下**不可靠或不可用**：

```bash
# ❌ 模式 1：timeout 命令（macOS 无此命令）
timeout 10 cargo run --bin signal-runtime

# ❌ 模式 2：sleep + kill（PID 捕获不可靠）
cargo run --bin signal-runtime &
PID=$!
sleep 5
kill $PID

# ❌ 模式 3：直接前台运行（会卡死）
cargo run --bin signal-runtime
```

---

## 解决方案

### 方案 A：输出重定向到文件（推荐）

**适用场景**：运行 demo binary 并查看输出

```bash
# 步骤 1：后台运行 + 重定向到文件
cargo run --bin signal-runtime > /tmp/signal-runtime-demo.log 2>&1 &
PID=$!

# 步骤 2：等待足够时间让 demo 执行完
sleep 3

# 步骤 3：终止进程
kill $PID 2>/dev/null

# 步骤 4：从文件读取输出（过滤编译警告）
cat /tmp/signal-runtime-demo.log | grep "fields" | grep -E "INFO|WARN|ERROR"
```

**优点**：输出完整保留，可反复查看，不受后台进程管理限制。

**注意事项**：
- 日志文件路径用 `/tmp/` 确保可写
- `sleep` 时间根据 demo 复杂度调整（通常 3-5 秒足够）
- `kill` 后不需要 `wait`，避免再次卡住

### 方案 B：使用 job_output 读取后台输出

**适用场景**：命令已被自动移到后台（超过 60 秒超时）

当看到 `Command is taking longer than expected and has been moved to background` 提示时：

```
Background shell ID: 042
```

操作步骤：
1. 使用 `job_output` 读取输出（设置 wait=true 会一直等，设为 false 立即返回已有输出）
2. 使用 `job_kill` 终止进程

**注意**：`job_output` 设 `wait=true` 时，如果进程仍在运行，**本身也会卡住**。应先设 `wait=false` 查看状态，或先 `job_kill` 再读取。

### 方案 C：构建不含 idle 循环的测试专用 binary

在 `main.rs` 中用条件编译控制是否进入 idle 循环：

```rust
#[cfg(feature = "server")]
async fn idle_loop() {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop { interval.tick().await; }
}

#[cfg(not(feature = "server"))]
async fn idle_loop() {}

// 在 main 末尾
idle_loop().await;
```

`Cargo.toml` 中：
```toml
[features]
default = ["server"]
server = []
```

- 生产运行：`cargo run --bin signal-runtime`（默认含 idle 循环）
- 开发调试：`cargo run --bin signal-runtime --no-default-features`（执行完自动退出）

---

## 操作指引（快速参考）

### 运行任何 demo binary 的标准流程

```
1. 后台运行 + 重定向
   cargo run --bin <name> > /tmp/<name>-demo.log 2>&1 &
   PID=$!
   sleep 3
   kill $PID 2>/dev/null

2. 查看输出
   cat /tmp/<name>-demo.log | grep "fields" | grep -E "INFO|WARN|ERROR"

3. 如果卡住了
   job_kill <shell_id>
```

### 如果已经卡住（急救）

1. 确认 shell ID（从错误信息中获取）
2. 执行 `job_kill <shell_id>`
3. 用方案 A 重新运行

---

## 历史记录

| 日期 | Binary | 现象 | 解决方式 |
|------|--------|------|----------|
| 2026-04-24 | signal-runtime | sleep+kill 无效，进程被自动移到后台 | 改用输出重定向到文件 |
| 2026-04-24 | portfolio-engine | 同上 | 同上 |
| 2026-04-24 | risk-engine | 输出重定向到文件 | ✅ 一次成功 |
| 2026-04-24 | gateway-trading | 输出重定向到文件 | ✅ 一次成功 |

---

## 规范建议

1. **所有含无限循环的 binary**，开发调试时统一使用方案 A（输出重定向到文件）
2. **永远不要直接前台运行**含 `loop { }` 的服务型 binary
3. **遇到"卡住"时**，第一时间 `job_kill`，不要等待或重试
4. **新项目如需频繁调试**，建议采用方案 C（条件编译），一劳永逸
