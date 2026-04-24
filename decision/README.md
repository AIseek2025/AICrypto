# decision/

决策层，包含 Skill 路由、组合管理、资金分配、执行规划和风控网关。

## 目录

| 目录 | 职责 |
|------|------|
| `skill_router/` | Skill 注册、查找与匹配 |
| `portfolio/` | 组合仓位与暴露控制 |
| `allocator/` | 动态资金分配 |
| `planner/` | 执行规划（Signal -> OrderIntent） |
| `risk_gateway/` | 统一风控闸门（**首批重中之重**） |

## 数据流

```
SignalEvent -> Skill Router -> Planner -> OrderIntent -> Risk Gateway -> Execution
```
