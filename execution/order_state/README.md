# order_state

订单状态机模块。

## 订单生命周期

```
CREATED -> RISK_APPROVED -> SENT -> ACKED -> PARTIALLY_FILLED -> FILLED
                    |          |          |
                    v          v          v
               REJECTED  CANCEL_PENDING  UNKNOWN
                              |
                              v
                           CANCELED
```

## 状态定义

| 状态 | 说明 |
|------|------|
| `CREATED` | 意图创建 |
| `RISK_APPROVED` | 风控通过 |
| `SENT` | 已发送到交易所 |
| `ACKED` | 交易所确认接收 |
| `PARTIALLY_FILLED` | 部分成交 |
| `FILLED` | 完全成交 |
| `CANCEL_PENDING` | 撤单中 |
| `CANCELED` | 已撤单 |
| `REJECTED` | 被拒绝 |
| `EXPIRED` | 已过期 |
| `UNKNOWN` | 状态未知（需对账确认） |
| `RECONCILED` | 已对账确认 |

## 接口草案

```
init(intent) -> order_state
transition(current_state, event) -> new_state
reconcile(order_ref, exchange_data) -> reconcile_result
```
