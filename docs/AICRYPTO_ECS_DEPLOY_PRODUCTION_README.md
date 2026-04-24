# AICrypto ECS 正式部署手册

## 1. 文档目的

本文档记录 `AICrypto` 在阿里云 ECS 上的真实生产部署流程，作为后续团队统一复用的上线、回滚与巡检手册。

适用范围：

- 域名：`aicrypto.cool`
- 别名：`www.aicrypto.cool`
- 部署形态：`Next.js (standalone) + systemd + Nginx + Certbot`
- 发布策略：单站点独立目录、独立本地端口、独立 systemd 服务、独立 Nginx server block

本文重点强调：

- 同机有十几个项目，**禁止**影响其他站点
- 只允许操作 AICrypto 自己的目录、服务、Nginx 配置与证书
- 一切生产变更应遵循"先本机回环验证，再接公网"的顺序

配套资料：

- 一键部署命令：`./scripts/ecs/deploy.sh`
- 一键证书命令：`./scripts/ecs/setup-ssl.sh`
- 运维脚本：`./scripts/ecs/ops.sh`

---

## 2. 当前生产环境

### 2.1 核心信息

| 项          | 值                                        |
| ---------- | ---------------------------------------- |
| ECS 公网 IP  | `8.218.209.218`                          |
| SSH 用户     | `admin`                                  |
| 正式域名       | `https://aicrypto.cool`                  |
| 别名域名       | `https://www.aicrypto.cool`              |
| 代码目录       | `/var/www/aicrypto/current`              |
| 共享环境文件     | `/var/www/aicrypto/shared/aicrypto.env`  |
| systemd 服务 | `aicrypto-web`                           |
| 本地监听地址     | `127.0.0.1:3006`                         |
| Nginx 配置   | `/etc/nginx/conf.d/aicrypto.cool.conf`   |
| 证书目录       | `/etc/letsencrypt/live/aicrypto.cool/`   |
| 日志目录       | `/var/log/aicrypto/`                     |

### 2.2 架构说明

AICrypto 采用前后端分离架构：

- **前端**：Next.js 16 standalone 模式，systemd 常驻，Nginx 反代到 `127.0.0.1:3006`
- **后端 API Gateway**（端口 8080）：Rust 编写，pipeline 集成服务
- **Sim Trading Engine**（端口 8090）：Rust 编写，模拟交易引擎 + API

当前部署仅包含前端。后端服务需要单独部署（见第 13 节）。

### 2.3 与同机其他项目的隔离原则

服务器上已有 `AINews`、`AIEmail`、`AIInvestor`、`GiftHub`、`CloudCode` 等多个项目，AICrypto 必须保持如下隔离：

- 不复用其他项目目录
- 不复用其他项目端口
- 不修改其他项目 Nginx conf
- 不重启其他项目的 systemd / pm2 / Docker 服务
- 不执行全局清理命令，如批量删容器、批量 reload 无关服务、批量删日志

---

## 3. 当前生产目录与对象

### 3.1 目录

- 代码目录：`/var/www/aicrypto/current`
- 共享配置：`/var/www/aicrypto/shared/aicrypto.env`
- 运行日志：`/var/log/aicrypto/web.log`
- 错误日志：`/var/log/aicrypto/web-error.log`

### 3.2 服务

- `aicrypto-web`

查看状态：

```bash
systemctl status aicrypto-web
systemctl is-active aicrypto-web
systemctl is-enabled aicrypto-web
```

### 3.3 端口

- 对外：`80 / 443` 由 `nginx` 占用
- AICrypto 本地应用：`127.0.0.1:3006`

检查端口：

```bash
sudo ss -ltnp | grep 3006
sudo ss -ltnp | grep -E ':(80|443)\b'
```

---

## 4. 生产环境变量

当前核心环境变量写在：

- `/var/www/aicrypto/shared/aicrypto.env`

推荐至少包含：

```bash
NEXT_PUBLIC_SITE_URL=https://aicrypto.cool
NEXT_PUBLIC_API_URL=http://127.0.0.1:8080
NEXT_PUBLIC_SIM_URL=http://127.0.0.1:8090
NODE_ENV=production
PORT=3006
HOSTNAME=127.0.0.1
```

说明：

- `NEXT_PUBLIC_SITE_URL`：生产必须，决定 canonical、OG
- `NEXT_PUBLIC_API_URL`：后端 API Gateway 地址
- `NEXT_PUBLIC_SIM_URL`：模拟交易引擎地址

---

## 5. systemd 配置

当前服务文件：

- `/etc/systemd/system/aicrypto-web.service`

核心配置逻辑：

```ini
[Unit]
Description=AICrypto Web Console
After=network.target

[Service]
Type=simple
User=admin
WorkingDirectory=/var/www/aicrypto/current
EnvironmentFile=/var/www/aicrypto/shared/aicrypto.env
ExecStart=/var/www/aicrypto/current/node_modules/.bin/next start --hostname 127.0.0.1 --port 3006
Restart=always
RestartSec=5
StandardOutput=append:/var/log/aicrypto/web.log
StandardError=append:/var/log/aicrypto/web-error.log

[Install]
WantedBy=multi-user.target
```

常用命令：

```bash
sudo systemctl daemon-reload
sudo systemctl enable aicrypto-web
sudo systemctl restart aicrypto-web
sudo systemctl status aicrypto-web
journalctl -u aicrypto-web -n 200 --no-pager
```

---

## 6. Nginx 配置

当前生产配置文件：

- `/etc/nginx/conf.d/aicrypto.cool.conf`

目标规则：

1. `http://aicrypto.cool` -> `301` 到 `https://aicrypto.cool`
2. `http://www.aicrypto.cool` -> `301` 到 `https://aicrypto.cool`
3. `https://www.aicrypto.cool` -> `301` 到 `https://aicrypto.cool`
4. `https://aicrypto.cool` -> 反代 `127.0.0.1:3006`
5. `/.well-known/acme-challenge/` 指向 `/var/www/certbot`

完整配置：

```nginx
# === AICrypto aicrypto.cool ===

# HTTP -> HTTPS redirect
server {
    listen 80;
    server_name aicrypto.cool www.aicrypto.cool;

    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        return 301 https://aicrypto.cool$request_uri;
    }
}

# www -> non-www redirect
server {
    listen 443 ssl;
    server_name www.aicrypto.cool;

    ssl_certificate /etc/letsencrypt/live/aicrypto.cool/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/aicrypto.cool/privkey.pem;

    return 301 https://aicrypto.cool$request_uri;
}

# Main HTTPS server
server {
    listen 443 ssl;
    server_name aicrypto.cool;

    ssl_certificate /etc/letsencrypt/live/aicrypto.cool/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/aicrypto.cool/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;

    location / {
        proxy_pass http://127.0.0.1:3006;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    access_log /var/log/nginx/aicrypto_access.log;
    error_log  /var/log/nginx/aicrypto_error.log;
}
```

每次修改后必须执行：

```bash
sudo nginx -t
sudo systemctl reload nginx
```

严禁：

- 修改其他站点 conf
- 覆盖其他域名证书路径
- 删除 `conf.d` 内其他项目配置

---

## 7. HTTPS 证书

### 7.1 签发方式

采用 `certbot webroot`：

```bash
sudo certbot certonly \
  --webroot -w /var/www/certbot \
  -d aicrypto.cool \
  -d www.aicrypto.cool \
  --non-interactive \
  --agree-tos \
  -m hello@aicrypto.cool
```

### 7.2 当前证书目录

- `/etc/letsencrypt/live/aicrypto.cool/fullchain.pem`
- `/etc/letsencrypt/live/aicrypto.cool/privkey.pem`

### 7.3 到期检查

```bash
sudo certbot certificates
```

### 7.4 续签

`certbot` 已自动注册续签任务，手工测试可执行：

```bash
sudo certbot renew --dry-run
```

---

## 8. 标准发布流程

### 8.1 本地准备

在本地仓库执行：

```bash
cd /Users/surferboy/AICrypto/apps/console-web
npm run build
```

确认 `next.config.ts` 包含 `output: "standalone"`。

### 8.2 提交并推送

```bash
cd /Users/surferboy/AICrypto
git add -A
git commit -m "release: 更新说明"
git push origin main
```

### 8.3 一键部署

```bash
cd /Users/surferboy/AICrypto
./scripts/ecs/deploy.sh
```

说明：

- 自动在本地 build
- rsync 同步到 ECS `/var/www/aicrypto/current`
- 自动写入 systemd 服务和环境变量
- 自动启动服务
- 不覆盖 `/var/www/aicrypto/shared/aicrypto.env`（如已存在手动修改的版本）

### 8.4 安装 HTTPS 证书

首次部署后执行：

```bash
./scripts/ecs/setup-ssl.sh
```

该命令会：

1. 向 Let's Encrypt 申请证书
2. 写入完整 Nginx HTTPS 配置（含 www 跳转、HSTS）
3. Reload Nginx

### 8.5 本机回环验证

```bash
ssh admin@8.218.209.218 "curl -I http://127.0.0.1:3006/"
```

预期：

- 返回 `200`
- 可看到 Next.js 响应头

### 8.6 公网验证

```bash
curl -I http://aicrypto.cool
curl -I https://aicrypto.cool
curl -I https://www.aicrypto.cool
```

预期：

- `http://aicrypto.cool` -> `301`
- `https://aicrypto.cool` -> `200`
- `https://www.aicrypto.cool` -> `301` 到主域

---

## 9. 回滚方案

### 9.1 Git 版本回滚

```bash
cd /Users/surferboy/AICrypto
# 查看可回滚的版本
git log --oneline -10
# 回到指定版本并重新部署
git checkout <commit-sha>
./scripts/ecs/deploy.sh
```

### 9.2 配置级回滚

如仅 Nginx 出错：

```bash
ssh admin@8.218.209.218
sudo cp /etc/nginx/conf.d/aicrypto.cool.conf /etc/nginx/conf.d/aicrypto.cool.conf.bak.$(date +%Y%m%d-%H%M%S)
# 恢复或修改配置
sudo nginx -t
sudo systemctl reload nginx
```

### 9.3 服务级回滚

```bash
ssh admin@8.218.209.218
sudo systemctl stop aicrypto-web
# 修复后
sudo systemctl start aicrypto-web
```

---

## 10. 常用巡检命令

### 10.1 快捷运维

```bash
cd /Users/surferboy/AICrypto

# 服务状态
./scripts/ecs/ops.sh status

# 重启服务
./scripts/ecs/ops.sh restart

# 查看日志
./scripts/ecs/ops.sh logs 200

# 查看错误日志
./scripts/ecs/ops.sh error-logs 50

# Reload Nginx
./scripts/ecs/ops.sh reload-nginx

# 证书检查
./scripts/ecs/ops.sh cert-check

# 完整健康检查
./scripts/ecs/ops.sh health
```

### 10.2 手动巡检

```bash
# 服务
systemctl status aicrypto-web
journalctl -u aicrypto-web -n 200 --no-pager
tail -n 200 /var/log/aicrypto/web.log

# Nginx
sudo nginx -t
sudo tail -n 200 /var/log/nginx/aicrypto_access.log
sudo tail -n 200 /var/log/nginx/aicrypto_error.log

# 域名与证书
curl -I https://aicrypto.cool
sudo certbot certificates
sudo certbot renew --dry-run
```

---

## 11. 风险与注意事项

### 11.1 不要影响其他项目

严禁以下操作：

- `systemctl restart` 其他项目服务
- 改动 `/etc/nginx/conf.d/` 中非 AICrypto 配置
- 删除 `/var/www`、`/opt` 中其他目录
- 使用全局清理命令

### 11.2 不要覆盖共享环境变量

部署代码时不要把生产环境变量直接写进仓库，不要覆盖：

- `/var/www/aicrypto/shared/aicrypto.env`

### 11.3 发布顺序不能颠倒

必须坚持：

1. 先本地 `npm run build`
2. 再 `deploy.sh` 同步到服务器
3. 先本机 `127.0.0.1:3006` 验证
4. 最后再验证公网

### 11.4 DNS 检查

部署前确认 DNS 解析：

```bash
dig aicrypto.cool +short
# 应返回 8.218.209.218
```

---

## 12. 端口分配记录

| 项目          | 端口    | 状态 |
| ----------- | ----- | -- |
| CloudCode   | 3003  | 已占用 |
| AICrypto    | 3006  | 已分配 |
| API Gateway | 8080  | 待部署 |
| Sim Trading | 8090  | 待部署 |

部署前务必确认端口未被占用：

```bash
sudo ss -ltnp | grep 3006
```

---

## 13. 后端服务部署（待执行）

AICrypto 后端为 Rust 工作区，包含 12 个 crate，8 个二进制目标。部署步骤：

### 13.1 方案 A：服务器本地编译

```bash
# 在 ECS 上安装 Rust（如未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 克隆代码并编译
cd /var/www/aicrypto
git clone https://github.com/AIseek2025/AICrypto.git repo
cd repo
cargo build --release

# 运行 sim-trading
./target/release/sim-trading
```

### 13.2 方案 B：本地交叉编译

```bash
# macOS 交叉编译 Linux x86_64
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
# 上传二进制
scp target/x86_64-unknown-linux-gnu/release/sim-trading admin@8.218.209.218:/var/www/aicrypto/current/
```

### 13.3 systemd 服务配置（示例）

```ini
[Unit]
Description=AICrypto Sim Trading Engine
After=network.target

[Service]
Type=simple
User=admin
WorkingDirectory=/var/www/aicrypto/current
ExecStart=/var/www/aicrypto/current/sim-trading
Restart=always
RestartSec=5
StandardOutput=append:/var/log/aicrypto/sim-trading.log
StandardError=append:/var/log/aicrypto/sim-trading-error.log

[Install]
WantedBy=multi-user.target
```

---

## 14. 本次真实上线记录

_待部署完成后填写。_
