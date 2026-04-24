# user_stream

币安用户流订阅模块。

## 职责

- ListenKey 创建与续期
- 订单更新事件监听
- 仓位更新事件监听
- 余额更新事件监听
- 断线自动重连与保活

## 输出

- 订单更新 -> CanonicalEvent
- 仓位更新 -> CanonicalEvent
- 账户更新 -> CanonicalEvent
