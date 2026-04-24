# adapters/

外部数据源接入层。

## 职责

对接交易所、社交平台、新闻源、链上数据等外部系统，将原始数据标准化为平台 CanonicalEvent。

## 目录

| 目录 | 职责 |
|------|------|
| `exchanges/binance/` | 币安永续合约适配器（首批核心） |
| `exchanges/binance/market_data/` | 市场数据接入 |
| `exchanges/binance/trading/` | 交易执行接入 |
| `exchanges/binance/user_stream/` | 用户流接入 |
| `exchanges/binance/mappings/` | Raw -> Canonical 映射说明 |
| `market_data/` | 通用市场数据适配 |
| `social/` | 社交平台适配 |
| `news/` | 新闻源适配 |
| `onchain/` | 链上数据适配 |
| `knowledge/` | 外部知识源适配 |

## 边界

- 允许：接入与映射、协议转换
- 禁止：策略判断、风控裁决、正式下单（trading 子目录除外）
