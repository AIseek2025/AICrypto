# execution/

交易执行层，包含交易网关、订单状态机、仓位管理、对账和应急处理。

## 目录

| 目录 | 职责 |
|------|------|
| `gateway/` | 交易网关 — 接收 OrderIntent，对接交易所 |
| `order_state/` | 订单状态机 — 维护订单生命周期 |
| `positions/` | 仓位管理 — 实时仓位追踪 |
| `reconciliation/` | 对账引擎 — 本地与交易所状态比对 |
| `emergency/` | 应急处理 — 紧急平仓、reduce-only 通道 |

## 数据流

```
OrderIntent -> Gateway -> Exchange -> User Stream -> Order State -> Positions
```
