#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# AICrypto ECS 一键部署脚本
# 用法: ./scripts/ecs/deploy.sh [skip-build]
# ============================================================

REMOTE="${DEPLOY_REMOTE:-admin@localhost}"
DOMAIN="${DEPLOY_DOMAIN:-localhost}"
APP_PORT=3006
SIM_PORT=8090
CODE_DIR="/var/www/aicrypto/current"
SHARED_DIR="/var/www/aicrypto/shared"
LOG_DIR="/var/log/aicrypto"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
FRONTEND_DIR="$PROJECT_ROOT/apps/console-web"

echo "=========================================="
echo "  AICrypto ECS Deploy"
echo "  Domain:  $DOMAIN"
echo "  App Port: $APP_PORT"
echo "  Server:  $REMOTE"
echo "=========================================="

# ---- Step 1: Local build ----
echo ""
echo ">>> [1/8] Building frontend locally..."
cd "$FRONTEND_DIR"
npm run build

echo "Build complete. Checking standalone output..."
if [ ! -d ".next/standalone" ]; then
  echo "ERROR: standalone output not found. Ensure next.config.ts has output: 'standalone'"
  exit 1
fi
echo "OK: standalone output found"

# ---- Step 2: Create remote directories ----
echo ""
echo ">>> [2/8] Creating remote directories..."
ssh "$REMOTE" "sudo mkdir -p $CODE_DIR $SHARED_DIR $LOG_DIR && sudo chown -R admin:admin /var/www/aicrypto /var/log/aicrypto"

# ---- Step 3: Sync frontend code ----
echo ""
echo ">>> [3/8] Syncing frontend to ECS..."
rsync -avz --delete \
  --exclude 'node_modules/.cache' \
  --exclude '.git' \
  "$FRONTEND_DIR/" "$REMOTE:$CODE_DIR/"

# ---- Step 4: Install deps on server ----
echo ""
echo ">>> [4/8] Installing dependencies on ECS..."
ssh "$REMOTE" "cd $CODE_DIR && npm install --omit=dev"

# ---- Step 5: Write env file ----
echo ""
echo ">>> [5/8] Writing environment file..."
ssh "$REMOTE" "cat > $SHARED_DIR/aicrypto.env << 'ENVEOF'
NEXT_PUBLIC_SITE_URL=https://$DOMAIN
NEXT_PUBLIC_API_URL=http://127.0.0.1:8080
NEXT_PUBLIC_SIM_URL=http://127.0.0.1:$SIM_PORT
NODE_ENV=production
PORT=$APP_PORT
HOSTNAME=127.0.0.1
ENVEOF"

# ---- Step 6: Install systemd service ----
echo ""
echo ">>> [6/8] Installing systemd service..."
ssh "$REMOTE" "sudo tee /etc/systemd/system/aicrypto-web.service > /dev/null << 'SVCEOF'
[Unit]
Description=AICrypto Web Console
After=network.target

[Service]
Type=simple
User=admin
WorkingDirectory=$CODE_DIR
EnvironmentFile=$SHARED_DIR/aicrypto.env
ExecStart=$CODE_DIR/node_modules/.bin/next start --hostname 127.0.0.1 --port $APP_PORT
Restart=always
RestartSec=5
StandardOutput=append:$LOG_DIR/web.log
StandardError=append:$LOG_DIR/web-error.log

[Install]
WantedBy=multi-user.target
SVCEOF"

# ---- Step 7: Start the service ----
echo ""
echo ">>> [7/8] Starting aicrypto-web service..."
ssh "$REMOTE" "sudo systemctl daemon-reload && sudo systemctl enable aicrypto-web && sudo systemctl restart aicrypto-web"
sleep 3
ssh "$REMOTE" "sudo systemctl status aicrypto-web --no-pager"

# ---- Step 8: Install Nginx config (HTTP only first for certbot) ----
echo ""
echo ">>> [8/8] Installing Nginx HTTP config (for certbot)..."
ssh "$REMOTE" "sudo tee /etc/nginx/conf.d/$DOMAIN.conf > /dev/null << 'NGXEOF'
# AICrypto — HTTP only (run certbot after this)
server {
    listen 80;
    server_name $DOMAIN www.$DOMAIN;

    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        return 301 https://\$host\$request_uri;
    }
}
NGXEOF"

ssh "$REMOTE" "sudo nginx -t && sudo systemctl reload nginx"
echo ""
echo "=========================================="
echo "  Frontend deployed!"
echo "  Next: run ./scripts/ecs/setup-ssl.sh"
echo "=========================================="
