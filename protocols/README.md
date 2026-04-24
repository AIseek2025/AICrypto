# protocols/

核心协议定义目录。

这是 AICrypto **最优先**建立的目录。所有模块间交互必须基于本目录定义的协议。

## 冻结协议清单

| 协议 | 用途 | 状态 |
|------|------|------|
| `canonical_event/` | 统一事件总线对象 | v1 冻结 |
| `market_snapshot/` | 市场快照 | v1 冻结 |
| `feature_vector/` | 特征向量 | v1 冻结 |
| `strategy_spec/` | 策略定义 | v1 冻结 |
| `skill_spec/` | Skill 定义 | v1 冻结 |
| `signal_event/` | 信号事件 | v1 冻结 |
| `order_intent/` | 订单意图 | v1 冻结 |
| `risk_decision/` | 风控决策 | v1 冻结 |
| `execution_report/` | 执行报告 | v1 冻结 |
| `review_report/` | 复盘报告 | v1 冻结 |

## 纪律

- 新模块必须先声明使用哪些协议
- 新功能先判断是否要扩展现有协议，而不是随意新建对象
- 修改协议必须走版本升级和兼容评审
