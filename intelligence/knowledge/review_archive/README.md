# review_archive

复盘归档模块。

## 职责

- 交易复盘报告归档
- 策略表现归档
- 知识回流与引用

## 接口草案

```
archive(review_report) -> archive_ref
query(filters) -> archive_results
link_execution(review_ref, execution_refs) -> ack
```
