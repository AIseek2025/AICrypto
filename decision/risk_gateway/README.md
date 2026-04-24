# risk_gateway

统一风控闸门 — **首批核心模块**。

## 职责

风控是独立服务，位于信号之后、执行之前，拥有最终 veto 权。

## 风控检查层级

1. **账户级** — 最大总仓位、最大日亏损、最大回撤、最大杠杆
2. **标的级** — 单币最大暴露、最低流动性、最大滑点、禁开仓窗口
3. **策略级** — 最大连续亏损、最大并发信号、策略冷却时间
4. **组合级** — 高相关币种聚集暴露、同板块集中暴露、多空失衡
5. **系统级** — 接口异常率、用户流断开、行情延迟、对账失败

## 熔断策略

- 只禁止开新仓
- 自动降低仓位
- 仅允许 reduce-only
- 全局紧急平仓
- 切换到人工审批模式

## 接口草案

```
pre_trade(intent, context) -> RiskDecision
in_trade(position_state, context) -> RiskDecision
post_trade(execution_report, context) -> RiskDecision
explain(decision_id) -> explanation
```
