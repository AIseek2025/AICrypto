# market_data

币安市场数据接入模块。

## 职责

- exchangeInfo 元数据同步
- K 线历史数据回补
- Funding / OI / Mark Price 采集
- WebSocket 市场流订阅与重连
- 原始数据标准化为 CanonicalEvent

## 首批接口

- `GET /fapi/v1/exchangeInfo` — 合约元数据
- `GET /fapi/v1/klines` — K 线历史
- `GET /fapi/v1/fundingRate` — 资金费率
- `GET /fapi/v1/openInterest` — 未平仓量
- `symbol@kline_<interval>` — K 线流
- `symbol@markPrice` — 标记价格流
- `!ticker@arr` — 全市场 Ticker 流
