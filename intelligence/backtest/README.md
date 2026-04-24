# backtest

回测引擎模块。

## 职责

为策略和 Skill 提供历史回测能力。

## 接口草案

```
run_backtest(strategy, dataset, config) -> backtest_result
evaluate(result) -> metrics
render_report(result) -> report_ref
promote_candidate(result) -> proposal
```

## 回测指标

- 胜率、盈亏比
- 最大回撤、收益波动比
- 不同市场状态表现
- 连续亏损分布
- 滑点与手续费影响
