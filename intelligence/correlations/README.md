# correlations

相关性引擎模块。

## 职责

处理 BTC、板块、人物、事件、情绪对币种的相关性影响。

## 相关性类型

1. 大盘主导相关 (BTC -> 全市场)
2. 板块轮动相关 (龙头 -> 同赛道)
3. 叙事相关 (AI/L2/DeFi/Meme)
4. 人物影响相关 (Musk/CZ/Vitalik)
5. 宏观事件相关
6. 链上/项目事件相关

## 接口草案

```
ingest_event(event) -> ack
score_relations(symbol, context) -> correlation_score
generate_signal(score, context) -> SignalEvent | None
```
