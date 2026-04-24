# trading

币安交易执行接入模块。

## 职责

- 下单、撤单、改单、批量下单
- 查询订单与仓位
- 调整杠杆、保证金模式
- 限流与幂等控制
- 审计日志记录

## 接口草案

```
place_order(intent) -> ExecutionReport
cancel_order(order_ref) -> ExecutionReport
query_order(order_ref) -> ExecutionReport
close_position(symbol, reason) -> ExecutionReport
set_protection(symbol, tp_sl_plan) -> ack
reduce_risk(scope, reason) -> ack
```
