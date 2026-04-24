# states

市场状态引擎模块。

## 职责

判断市场当前所处状态 (regime)，供 Skill Router 筛选适用 Skill。

## 市场状态定义

| 状态 | 说明 |
|------|------|
| `BULL_TREND` | 牛市趋势 |
| `BULL_EUPHORIA` | 牛市狂热 |
| `RANGE_NEUTRAL` | 震荡中性 |
| `RISK_OFF` | 风险规避 |
| `PANIC_SELL` | 恐慌抛售 |
| `SHORT_SQUEEZE` | 空头挤压 |
| `EVENT_DRIVEN` | 事件驱动 |

## 接口草案

```
evaluate(symbol, features, context) -> state_result
classify_regime(state_result) -> regime_label
emit_signal(state_result) -> SignalEvent | None
```
