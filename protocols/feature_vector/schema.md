# FeatureVector

统一特征输出对象。

## 用途

承载从市场数据和事件流中提取的标准化特征。

## 字段定义 (v1)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `schema_name` | string | Y | 固定值 `feature_vector` |
| `schema_version` | string | Y | 协议版本 `v1` |
| `feature_set` | string | Y | 特征集名称 |
| `feature_version` | string | Y | 特征集版本 |
| `symbol` | string | Y | 标的符号 |
| `window` | string | Y | 计算窗口 (如 `1h`, `4h`, `1d`) |
| `ts_feature` | int64 | Y | 特征计算时间戳 |
| `features` | object | Y | 特征键值对 |
| `source_refs` | string[] | N | 源数据引用 |

## 约束

- 首版不冻结所有特征字段名，但必须冻结特征集合的元数据结构
- features 内的键名建议遵循 `{category}_{metric}_{window}` 格式
