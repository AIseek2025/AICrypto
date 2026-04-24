# MarketSnapshot

某个时点某标的的市场快照。

## 用途

用于特征计算和状态判断，不直接替代原始逐笔流。

## 字段定义 (v1)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `schema_name` | string | Y | 固定值 `market_snapshot` |
| `schema_version` | string | Y | 协议版本 `v1` |
| `symbol` | string | Y | 标的符号 |
| `exchange` | string | Y | 交易所 |
| `market_type` | string | Y | 市场类型 (usds_m_futures) |
| `last_price` | decimal | Y | 最新价 |
| `mark_price` | decimal | N | 标记价格 |
| `index_price` | decimal | N | 指数价格 |
| `funding_rate` | decimal | N | 当前资金费率 |
| `open_interest` | decimal | N | 未平仓量 |
| `volume_24h` | decimal | N | 24h 成交量 |
| `ts_snapshot` | int64 | Y | 快照时间戳 |
