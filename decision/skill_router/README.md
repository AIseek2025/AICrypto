# skill_router

Skill 注册与路由模块。

## 职责

- Skill 注册与查找
- 根据市场状态和信号匹配适用 Skill
- Skill 版本管理

## 接口草案

```
register(skill_spec) -> skill_id
get(skill_id) -> SkillSpec
find(criteria) -> list[SkillSpec]
resolve(signal, market_state) -> list[SkillSpec]
```
