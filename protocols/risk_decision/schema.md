# RiskDecision

统一风险网关的裁决结果。

## 用途

风控闸门对每个 OrderIntent 做出的裁决。

## 字段定义 (v1)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `decision_id` | string | Y | 决策唯一 ID |
| `target_ref` | string | Y | 目标引用 (intent_id) |
| `decision` | string | Y | 决策结果 (allow/deny/review/shrink) |
| `severity` | string | Y | 严重程度 (info/warning/critical/emergency) |
| `rule_hits` | object[] | N | 触发的风控规则 |
| `exposure_snapshot` | object | N | 当前暴露快照 |
| `required_actions` | string[] | N | 要求的后续动作 |
| `review_required` | boolean | N | 是否需要人工审核 |
| `ts_decision` | int64 | Y | 决策时间戳 |
