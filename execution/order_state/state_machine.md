# state_machine.md

## 订单状态机详细设计

### 状态迁移规则

| 当前状态 | 触发事件 | 目标状态 | 说明 |
|----------|----------|----------|------|
| CREATED | risk_approved | RISK_APPROVED | 风控通过 |
| CREATED | risk_denied | REJECTED | 风控拒绝 |
| RISK_APPROVED | send_success | SENT | 发送成功 |
| RISK_APPROVED | send_failed | REJECTED | 发送失败 |
| SENT | ack | ACKED | 交易所确认 |
| ACKED | partial_fill | PARTIALLY_FILLED | 部分成交 |
| ACKED | full_fill | FILLED | 完全成交 |
| ACKED | cancel_request | CANCEL_PENDING | 请求撤单 |
| PARTIALLY_FILLED | partial_fill | PARTIALLY_FILLED | 继续部分成交 |
| PARTIALLY_FILLED | full_fill | FILLED | 完全成交 |
| PARTIALLY_FILLED | cancel_request | CANCEL_PENDING | 请求撤单 |
| CANCEL_PENDING | cancel_confirmed | CANCELED | 撤单确认 |
| SENT | timeout/unknown | UNKNOWN | 状态未知 |
| UNKNOWN | reconcile_success | FILLED/CANCELED | 对账确认 |
| * | expire | EXPIRED | 过期 |

### 异常处理

当收到 503 且执行状态未知时：
1. 标记为 UNKNOWN
2. 等待用户流回报
3. 超时则查询订单接口
4. 仍无法确认则进入异常队列
