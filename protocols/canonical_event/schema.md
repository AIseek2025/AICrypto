# CanonicalEvent

平台内部统一事件总线对象。

## 用途

所有外部输入（行情、用户流、社交、新闻、链上、系统状态）必须先标准化为 CanonicalEvent。

## 字段定义 (v1)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `schema_name` | string | Y | 固定值 `canonical_event` |
| `schema_version` | string | Y | 协议版本，首版 `v1` |
| `event_id` | string | Y | 全局唯一事件 ID |
| `trace_id` | string | Y | 链路追踪 ID |
| `source_type` | string | Y | 来源类型 (exchange/social/news/onchain/system) |
| `source_name` | string | Y | 来源名称 (binance/twitter/...) |
| `event_type` | string | Y | 事件类型 (kline/trade/depth/mark_price/funding_rate/...) |
| `symbol` | string | N | 关联标的 |
| `ts_event` | int64 | Y | 事件发生时间 (毫秒时间戳) |
| `ts_ingested` | int64 | Y | 平台接收时间 (毫秒时间戳) |
| `payload` | object | Y | 事件负载，允许保留 source-specific 字段 |
| `quality_flags` | string[] | N | 数据质量标记 |
| `tags` | string[] | N | 自定义标签 |

## 约束

- 所有外部输入必须先标准化成 CanonicalEvent
- payload 内允许保留 source-specific 字段，但顶层结构必须统一
- event_id 必须全局唯一，建议 `{source_type}:{source_name}:{event_type}:{序列}`
