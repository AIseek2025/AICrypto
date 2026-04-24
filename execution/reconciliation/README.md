# reconciliation

对账引擎模块。

## 职责

- 定时比对本地订单状态与交易所状态
- 发现差异并生成修复计划
- 自动或半自动补偿

## 接口草案

```
compare(local, remote) -> diff
repair(diff) -> repair_plan
confirm(repair_plan) -> final_status
```
