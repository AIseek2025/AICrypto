# gateway

交易网关模块。

## 职责

- 接收 OrderIntent 并映射为交易所参数
- 精度修正与最小下单量检查
- 自动补齐 clientOrderId
- 选择 REST 或 WebSocket Trading API
- 记录审计日志

## 接口草案

```
submit(intent) -> ExecutionReport
cancel(order_ref) -> ExecutionReport
query(order_ref) -> ExecutionReport
```
