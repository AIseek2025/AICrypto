# SkillSpec

可被 Agent 或 Strategy 调用的标准交易动作单元。

## 用途

描述平台 Skill 的完整定义。

## 字段定义 (v1)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `skill_id` | string | Y | Skill 唯一 ID |
| `skill_name` | string | Y | Skill 名称 |
| `skill_family` | string | Y | 所属家族 (trend/short/correlation/risk) |
| `entry_conditions` | object | Y | 入场条件 |
| `position_rules` | object | Y | 仓位规则 |
| `add_rules` | object | N | 加仓规则 |
| `reduce_rules` | object | N | 减仓规则 |
| `exit_rules` | object | Y | 退出规则 (止盈/止损/时间退出) |
| `risk_rules` | object | Y | 风险规则 |
| `applicable_market_states` | string[] | Y | 适用市场状态 |
| `input_contract` | object | Y | 输入契约 (所需特征和事件) |
| `output_contract` | object | Y | 输出契约 (产生的信号类型) |
| `status` | string | Y | 状态 (draft/backtest_passed/paper_approved/live/disabled) |
| `version` | string | Y | 版本号 |

## 约束

- 首版必须支持 trend、short、correlation、risk 四类 Skill
- 每个 Skill 必须绑定回测报告引用
