# SignalEvent

研究或模型产生的候选信号。

## 用途

表示研究层或智能层产生的交易信号。SignalEvent 不是订单，必须继续流向 Decision 与 Risk 层处理。

## 字段定义 (v1)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `signal_id` | string | Y | 信号唯一 ID |
| `signal_type` | string | Y | 信号类型 (entry/exit/add/reduce/risk_alert) |
| `symbol` | string | Y | 标的符号 |
| `direction` | string | Y | 方向 (LONG/SHORT/NEUTRAL) |
| `confidence` | decimal | Y | 置信度 (0.0-1.0) |
| `horizon` | string | Y | 持仓周期 (scalp/intraday/swing/positional) |
| `reason_codes` | string[] | Y | 触发原因编码 |
| `evidence_refs` | string[] | N | 证据引用 |
| `ts_signal` | int64 | Y | 信号产生时间戳 |
