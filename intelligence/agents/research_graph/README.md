# research_graph

研究 Agent 图。

## 职责

组织研究型 Agent 流程：
- 扫描历史主升浪/主跌浪样本
- 提炼特征组合
- 发现可编码的规律
- 输出 Skill Proposal

## 接口草案

```
start(task, context) -> run_id
step(run_id, input) -> state
finalize(run_id) -> ReviewReport
```
