# ExecutionReport

执行层反馈报告。

## 用途

所有下单结果、撤单结果、部分成交和补偿结果，都必须最终映射到 ExecutionReport。

## 字段定义 (v1)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `report_id` | string | Y | 报告唯一 ID |
| `intent_id` | string | Y | 关联意图 ID |
| `exchange` | string | Y | 交易所 |
| `symbol` | string | Y | 标的符号 |
| `order_status` | string | Y | 订单状态 |
| `filled_qty` | decimal | N | 成交数量 |
| `avg_fill_price` | decimal | N | 成交均价 |
| `fees` | object | N | 手续费 |
| `exchange_order_id` | string | N | 交易所订单 ID |
| `raw_status` | string | N | 原始状态 |
| `reconcile_state` | string | N | 对账状态 |
| `ts_report` | int64 | Y | 报告时间戳 |
