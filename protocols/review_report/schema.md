# ReviewReport

研究复盘或 Agent 审查的标准结果对象。

## 用途

用于研究闭环和知识沉淀，不直接进入执行层。

## 字段定义 (v1)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `review_id` | string | Y | 复盘唯一 ID |
| `review_type` | string | Y | 复盘类型 (strategy/skill/daily/weekly/event) |
| `target_ref` | string | Y | 复盘目标引用 |
| `summary` | string | Y | 摘要 |
| `findings` | object[] | Y | 发现 |
| `recommendations` | string[] | Y | 建议 |
| `evidence_refs` | string[] | N | 证据引用 |
| `reviewer` | string | Y | 审查者 (agent_id 或 human) |
| `ts_review` | int64 | Y | 复盘时间戳 |
