# StrategySpec

策略定义对象。

## 用途

描述策略对象本身，与 SkillSpec 明确区分。Strategy 是策略框架，Skill 是可被调用的交易动作单元。

## 字段定义 (v1)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `strategy_id` | string | Y | 策略唯一 ID |
| `strategy_name` | string | Y | 策略名称 |
| `strategy_type` | string | Y | 策略类型 (trend/mean_reversion/event_driven/correlation) |
| `owner` | string | Y | 负责人 |
| `input_requirements` | object | Y | 输入要求 |
| `signal_model` | object | Y | 信号模型描述 |
| `risk_assumptions` | object | Y | 风险假设 |
| `execution_constraints` | object | Y | 执行约束 |
| `status` | string | Y | 状态 (draft/testing/paper/live/disabled) |
| `version` | string | Y | 版本号 |
