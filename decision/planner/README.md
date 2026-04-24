# planner

执行规划模块。

## 职责

把通过 Skill 匹配的信号细化为具体 OrderIntent。

## 接口草案

```
plan(signal, skill, context) -> OrderIntent
adjust(intent, risk_decision) -> OrderIntent
```
