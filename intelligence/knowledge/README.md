# knowledge

知识服务模块。

## 子模块

| 目录 | 职责 |
|------|------|
| `skill_memory/` | Skill 运行记忆与经验沉淀 |
| `research_rag/` | 研究文档检索增强生成 |
| `review_archive/` | 复盘归档与知识回流 |

## 接口草案

```
index(document) -> doc_ref
search(query, scope) -> search_results
fetch(ref) -> knowledge_object
link_skill(skill_id, refs) -> ack
```
