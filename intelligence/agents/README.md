# agents

Agent 体系目录。

## Agent 类型

| 目录 | 职责 |
|------|------|
| `research_graph/` | 研究 Agent — 扫描历史样本，提炼特征，输出 Skill Proposal |
| `debate_graph/` | 辩论 Agent — 多 Agent 讨论信号有效性 |
| `review_graph/` | 复盘 Agent — 交易后复盘，评估 Skill 表现 |

## 纪律

- Agent 不能绕过风控直接交易
- 所有 Agent 输出必须可审计
