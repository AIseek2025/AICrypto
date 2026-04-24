# foundation/

平台基础能力层。

## 职责

提供所有上层服务共享的基础能力：

| 目录 | 职责 |
|------|------|
| `config/` | 统一配置管理 |
| `security/` | 安全与密钥管理 |
| `observability/` | 日志、Metrics、Tracing |
| `storage/` | 数据库、缓存、对象存储抽象 |
| `bus/` | 消息总线抽象 (NATS JetStream) |

## 边界

- 允许：基础能力抽象、公共工具
- 禁止：业务逻辑
