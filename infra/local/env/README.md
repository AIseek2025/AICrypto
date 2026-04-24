# AICrypto 环境配置

## 开发环境 (.env.dev)
```bash
RUST_LOG=debug
APP_ENV=dev
BINANCE_REST_URL=https://testnet.binancefuture.com
BINANCE_WS_URL=wss://stream.binancefuture.com/ws
BINANCE_TESTNET=true
DATABASE_URL=postgres://aicrypto:aicrypto@localhost:5432/aicrypto_dev
REDIS_URL=redis://localhost:6379
API_PORT=8080
```

## 仿真环境 (.env.paper)
```bash
RUST_LOG=info
APP_ENV=paper
BINANCE_REST_URL=https://testnet.binancefuture.com
BINANCE_WS_URL=wss://stream.binancefuture.com/ws
BINANCE_TESTNET=true
DATABASE_URL=postgres://aicrypto:aicrypto@localhost:5432/aicrypto_paper
REDIS_URL=redis://localhost:6379
API_PORT=8080
```

## 实盘环境 (.env.prod)
```bash
RUST_LOG=warn
APP_ENV=prod
BINANCE_REST_URL=https://fapi.binance.com
BINANCE_WS_URL=wss://fstream.binance.com/ws
BINANCE_TESTNET=false
DATABASE_URL=postgres://aicrypto:aicrypto@localhost:5432/aicrypto_prod
REDIS_URL=redis://localhost:6379
API_PORT=8080
```

## 前端环境

```bash
# .env.local (console-web)
NEXT_PUBLIC_API_URL=http://localhost:8080
```

## 切换环境

```bash
# 复制目标环境配置
cp infra/local/env/.env.dev .env

# 或使用 ops tool
python3 apps/ops-tools/aicrypto-ops.py health
```
