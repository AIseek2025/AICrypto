# features

特征引擎模块。

## 职责

从市场数据和事件流中提取标准化特征。

## 接口草案

```
register_feature_set(feature_set) -> ack
compute(snapshot, context) -> FeatureVector
batch_compute(range, symbols) -> list[FeatureVector]
validate(feature_vector) -> validation_result
```

## 首批特征集

- 基础趋势特征 (价格动量、均线、RSI、ATR)
- 成交量特征 (量比、换手率、大单占比)
- 衍生品特征 (Funding、OI 变化、Basis)
- 波动率特征 (历史波动率、波动率偏斜)
