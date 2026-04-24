#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# AICrypto HTTPS 证书申请 + Nginx HTTPS 配置
# 前置: deploy.sh 已完成, DNS 已指向 ECS
# 用法: ./scripts/ecs/setup-ssl.sh
# ============================================================

REMOTE="admin@8.218.209.218"
DOMAIN="aicrypto.cool"
APP_PORT=3006

echo "=========================================="
echo "  AICrypto SSL Setup"
echo "  Domain: $DOMAIN"
echo "=========================================="

# ---- Step 1: Request certificate ----
echo ""
echo ">>> [1/3] Requesting Let's Encrypt certificate..."
ssh "$REMOTE" "sudo certbot certonly \
  --webroot -w /var/www/certbot \
  -d $DOMAIN \
  -d www.$DOMAIN \
  --non-interactive \
  --agree-tos \
  -m hello@$DOMAIN"

# ---- Step 2: Install full Nginx config with HTTPS ----
echo ""
echo ">>> [2/3] Installing Nginx HTTPS config..."
ssh "$REMOTE" "sudo tee /etc/nginx/conf.d/$DOMAIN.conf > /dev/null << 'NGXEOF'
# === AICrypto aicrypto.cool ===

# HTTP -> HTTPS redirect
server {
    listen 80;
    server_name $DOMAIN www.$DOMAIN;

    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        return 301 https://$DOMAIN\$request_uri;
    }
}

# www -> non-www redirect
server {
    listen 443 ssl;
    server_name www.$DOMAIN;

    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;

    return 301 https://$DOMAIN\$request_uri;
}

# Main HTTPS server
server {
    listen 443 ssl;
    server_name $DOMAIN;

    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    add_header Strict-Transport-Security \"max-age=31536000; includeSubDomains\" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;

    location / {
        proxy_pass http://127.0.0.1:$APP_PORT;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection \"upgrade\";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_cache_bypass \$http_upgrade;
    }

    access_log /var/log/nginx/aicrypto_access.log;
    error_log  /var/log/nginx/aicrypto_error.log;
}
NGXEOF"

# ---- Step 3: Reload Nginx ----
echo ""
echo ">>> [3/3] Reloading Nginx..."
ssh "$REMOTE" "sudo nginx -t && sudo systemctl reload nginx"

echo ""
echo "=========================================="
echo "  SSL setup complete!"
echo "  Verify: https://$DOMAIN"
echo "=========================================="
