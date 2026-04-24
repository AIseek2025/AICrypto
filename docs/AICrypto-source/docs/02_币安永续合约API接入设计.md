# 币安永续合约 API 接入设计

## 1. 接入范围

首版聚焦 Binance USDⓈ-M Futures，也就是 USDT/UUSD 等稳定币计价的永续合约。原因如下：

- 标的数量多，流动性最好。
- 主流和山寨合约覆盖齐全，最适合“主升浪/主跌浪”研究。
- API 文档、测试网、市场数据流和用户数据流相对成熟。
- 资金结算统一，便于组合风险核算。

## 2. 官方 API 关键结论

### 2.1 基础端点

根据币安官方衍生品文档：

- 生产 REST 基础地址：`https://fapi.binance.com`
- 测试网 REST 基础地址：`https://demo-fapi.binance.com`
- WebSocket API 基础地址：`wss://ws-fapi.binance.com/ws-fapi/v1`
- 测试网 WebSocket API：`wss://testnet.binancefuture.com/ws-fapi/v1`
- 市场流 WebSocket 使用 Futures Stream 域名。

### 2.2 支持的能力类型

1. 公共市场数据 REST
   - `exchangeInfo`
   - `klines`
   - `continuousKlines`
   - `markPrice`
   - `premiumIndex`
   - `fundingRate`
   - `openInterest`
   - `ticker/24hr`
   - `depth`
   - `aggTrades`
2. WebSocket 市场流
   - K 线流
   - Mark Price 流
   - 深度流
   - 成交流
   - 综合 ticker 流
3. 私有 REST / WebSocket Trading API
   - 下单、改单、撤单、批量下单、查询订单、调整杠杆、调整保证金模式。
4. User Data Stream
   - 订单更新
   - 账户更新
   - 仓位更新
   - 余额更新

### 2.3 鉴权方式

- `TRADE` 和 `USER_DATA` 接口需要 API Key + 签名。
- REST 签名采用 HMAC SHA256，签名串包括请求参数与 `timestamp`。
- WebSocket Trading API 也支持签名请求。
- 所有签名请求建议使用较小 `recvWindow`，配合本地 NTP 校时。

### 2.4 限流与异常处理

币安官方文档明确说明：

- REST 和 WebSocket API 共享部分限流配额。
- 超过限流会触发 `429`，持续违规可能触发 `418` IP ban。
- 某些 `503` 并不代表明确失败，而是“执行状态未知”，必须通过订单查询或用户流确认，避免重复下单。
- 系统级保护 `-1008` 发生时，应优先减并发，且 reduce-only/close-position 订单通常优先保障。

这意味着 AICrypto 的执行器必须实现：

- 限流器
- 幂等下单键
- 订单状态确认补偿
- 执行状态机
- 强制减仓优先通道

## 3. 推荐接入策略

## 3.1 接入原则

- 行情与执行分离。
- REST 主要用于补数、快照、对账和兜底。
- WebSocket 主要用于实时市场流、用户流和低延迟下单回报。
- 所有订单最终状态以用户流 + 查询对账双确认。

## 3.2 首版优先接入的接口清单

### A. 元数据与交易规则

- `GET /fapi/v1/exchangeInfo`
  - 获取合约列表、交易规则、价格/数量精度、过滤器、状态。

### B. 历史数据

- `GET /fapi/v1/klines`
- `GET /fapi/v1/continuousKlines`
- `GET /fapi/v1/fundingRate`
- `GET /fapi/v1/openInterest`
- `GET /fapi/v1/ticker/24hr`
- `GET /fapi/v1/depth`
- `GET /fapi/v1/aggTrades`

### C. 实时数据流

- `symbol@kline_<interval>`
- `symbol@markPrice` 或 `symbol@markPrice@1s`
- `symbol@depth`
- `symbol@aggTrade`
- `!ticker@arr`

### D. 账户与下单

- 下单
- 撤单
- 查询当前订单/历史订单
- 查询仓位与余额
- 调整杠杆
- 调整持仓模式
- 调整保证金模式

### E. 用户流

- listen key 或对应用户数据流连接
- 监听订单、仓位、余额变更

## 4. 生产级接入架构

### 4.1 连接器分层

1. `binance-metadata-sync`
   - 周期拉取 `exchangeInfo`
   - 同步 symbol 生命周期和交易规则
2. `binance-market-rest-loader`
   - 回补 K 线、Funding、OI、Ticker 等历史数据
3. `binance-market-stream`
   - 长连订阅实时市场流
4. `binance-user-stream`
   - 订阅账户和订单事件
5. `binance-trading-gateway`
   - 对内提供统一下单接口
   - 对外调用 REST 或 WebSocket Trading API
6. `binance-reconciliation`
   - 对账、补状态、补漏单

### 4.2 订单执行状态机

建议订单状态机包括：

- `CREATED`
- `RISK_APPROVED`
- `SENT`
- `ACKED`
- `PARTIALLY_FILLED`
- `FILLED`
- `CANCEL_PENDING`
- `CANCELED`
- `REJECTED`
- `EXPIRED`
- `UNKNOWN`
- `RECONCILED`

当收到币安 503 且执行状态未知时：

1. 标记订单为 `UNKNOWN`
2. 等待用户流回报
3. 若超时未回报，则调用查询订单接口
4. 若仍无法确认，进入人工/自动风控异常队列

## 5. 数据字段设计建议

### 5.1 Symbol 主表

- `symbol`
- `contract_type`
- `underlying`
- `quote_asset`
- `margin_asset`
- `status`
- `onboard_date`
- `price_precision`
- `quantity_precision`
- `tick_size`
- `step_size`
- `min_qty`
- `min_notional`
- `max_leverage`
- `maint_margin_ratio`

### 5.2 行情与衍生特征表

- OHLCV
- Mark Price
- Index Price
- Funding Rate
- Open Interest
- Basis/Premium
- Volume / Trade Count
- Depth Imbalance
- Liquidation Proxy
- Volatility Regime

### 5.3 账户与订单表

- 账户余额快照
- 仓位快照
- 订单主表
- 成交明细
- 资金费历史
- 手续费历史
- 风险事件日志

## 6. 鉴权与密钥安全设计

- 生产密钥与测试网密钥分离。
- 研究环境不得直接访问生产实盘密钥。
- 密钥只保存在 KMS/Vault，不进入代码仓库与日志。
- 执行器只获得最小化权限的临时凭证或代理签名能力。
- 操作日志禁止打印签名串和完整请求参数。

## 7. 测试网接入建议

首版必须先跑通币安测试网：

1. 读取 `exchangeInfo`
2. 拉取至少 20 个标的多周期 K 线
3. 建立市场流和用户流连接
4. 下测试单
5. 监听订单更新
6. 查询订单与仓位对账
7. 模拟网络抖动、断线重连、限流和超时

## 8. 下单网关设计建议

对上层 Agent 暴露统一内部接口：

- `place_order(intent)`
- `cancel_order(order_ref)`
- `close_position(symbol, reason)`
- `set_protection(symbol, tp/sl plan)`
- `reduce_risk(scope, reason)`

内部由下单网关负责：

- 映射到币安参数
- 精度修正
- 检查最小下单量
- 自动补齐 clientOrderId
- 选择 REST 还是 WebSocket Trading API
- 记录审计日志
- 回写订单状态机

## 9. 为什么首版建议 REST + Stream 混合而不是只用一种

- 只用 REST：实时性不足，轮询浪费配额。
- 只用 Stream：补数、快照、状态确认不足。
- 混合模式最稳健：
  - Stream 负责实时性
  - REST 负责快照、历史、纠偏、对账

## 10. AICrypto 的最终接入建议

### 10.1 首版

- 行情：WebSocket Stream + REST 补数
- 交易：REST 下单为主，用户流确认；同时预留 WebSocket Trading API 实验通道
- 对账：定时 REST 查询
- 测试：先测试网，再仿真，再小仓位实盘

### 10.2 第二版

- 引入 WebSocket Trading API 降低交互延迟
- 引入多连接池与专用账户路由
- 引入订单执行质量评估与滑点归因

## 11. 关键风险提示

- 币安接口文档与字段可能迭代，必须通过适配层隔离外部变更。
- 山寨币永续合约流动性、深度、插针与资金费波动极大，不能把“可下单”误判为“适合自动化交易”。
- 交易所异常、网络抖动和用户流丢消息是常态，系统要按“异常必然发生”设计。

## 12. 官方文档参考

- Binance Derivatives USDⓈ-M Futures General Info
- Binance WebSocket API General Info
- Binance Market Data REST API
- Binance WebSocket Market Streams
- Binance User Data Streams

建议开发时固定建立 `binance adapter` 层，不让上层策略直接依赖任何外部字段原名。
