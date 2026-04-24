# AICrypto Project Rules

## ⛔ 熔断规则（最高优先级）

遇到以下任一错误时，**立即停止所有操作**，向用户报告，不重试：
- `Internal Server Error` / `操作失败` — 服务端故障
- `速率限制` / `rate limit` / `too many requests` — 已触发限流
- `i/o timeout` / `dial tcp` / `lookup` — 网络故障
- `connection refused` — 服务不可达

连续 2 次 API 错误 = 停止工作。不要自动重试。

## 工具调用限制

- 单条消息最多 3 个并行工具调用
- 修改→编译→测试 最多 3 轮，不过就停下来报告
- 不要 `cargo run`，先 build 再执行二进制
- 运行服务必须后台 + sleep + kill：`./target/debug/xxx > /tmp/xxx.log 2>&1 & PID=$!; sleep 5; kill $PID 2>/dev/null`
- 结果从日志文件读取：`cat /tmp/xxx.log | tail -20`

## 完整规范

详见 `.planning/RULES.md`
