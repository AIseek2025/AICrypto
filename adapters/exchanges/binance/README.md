# Binance USDⓈ-M Futures Adapter

币安永续合约适配器，AICrypto 首批核心模块。

## 子模块

| 模块 | 职责 |
|------|------|
| `market_data/` | 市场数据接入 (REST + WebSocket) |
| `trading/` | 交易执行 (下单/撤单/查单) |
| `user_stream/` | 用户流订阅 (订单/仓位/余额更新) |
| `mappings/` | 币安原始数据 -> CanonicalEvent 映射 |

## 接入范围

- 生产 REST: `https://fapi.binance.com`
- 测试网 REST: `https://testnet.binancefuture.com`
- 市场 WebSocket: `wss://fstream.binance.com/ws`
- 用户流 WebSocket: `wss://fstream.binance.com/ws/<listenKey>`

## 接口草案

```
connect_streams(config) -> stream_handle
backfill(range, symbols) -> raw_records
normalize(raw_event) -> CanonicalEvent
build_snapshot(events) -> MarketSnapshot
health() -> adapter_status
```
