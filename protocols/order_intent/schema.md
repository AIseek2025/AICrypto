# OrderIntent

通过决策层和风控前置后的候选交易意图。

## 用途

这是进入执行层的标准入口。执行层不得直接消费原始 SignalEvent。

## 字段定义 (v1)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `intent_id` | string | Y | 意图唯一 ID |
| `account_scope` | string | Y | 账户范围 |
| `symbol` | string | Y | 标的符号 |
| `side` | string | Y | 买卖方向 (BUY/SELL) |
| `position_side` | string | Y | 持仓方向 (LONG/SHORT) |
| `order_type` | string | Y | 订单类型 (LIMIT/MARKET/STOP) |
| `quantity` | decimal | Y | 数量 |
| `price_limit` | decimal | N | 限价 |
| `reduce_only` | boolean | N | 是否仅减仓 |
| `leverage_hint` | int | N | 杠杆建议 |
| `take_profit_hint` | object | N | 止盈建议 |
| `stop_loss_hint` | object | N | 止损建议 |
| `time_in_force` | string | N | 有效方式 (GTC/IOC/FOK) |
| `origin_ref` | string | Y | 来源引用 (signal_id) |
| `ts_intent` | int64 | Y | 意图产生时间戳 |
