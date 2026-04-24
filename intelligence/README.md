# intelligence/

智能分析层，包含特征计算、状态判断、相关性分析、回测、知识图谱和 Agent。

## 目录

| 目录 | 职责 | 语言 |
|------|------|------|
| `features/` | 特征引擎 | Rust |
| `states/` | 市场状态引擎 | Rust |
| `correlations/` | 相关性引擎 | Rust + Python |
| `backtest/` | 回测引擎 | Rust |
| `knowledge/` | 知识服务 | Python |
| `graph/` | 相关性图谱 | Python |
| `agents/` | Agent 体系 | Python |

## 输入

- CanonicalEvent
- MarketSnapshot

## 输出

- FeatureVector
- SignalEvent
- ReviewReport
