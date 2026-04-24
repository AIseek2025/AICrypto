#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# AICrypto ECS 运维脚本
# 用法: ./scripts/ecs/ops.sh <command>
# 命令: status | restart | logs | reload-nginx | cert-check | health
# ============================================================

REMOTE="${DEPLOY_REMOTE:-admin@localhost}"
DOMAIN="${DEPLOY_DOMAIN:-localhost}"
APP_PORT=3006
CODE_DIR="/var/www/aicrypto/current"
LOG_DIR="/var/log/aicrypto"

cmd="${1:-help}"

case "$cmd" in
  status)
    echo "=== Service ==="
    ssh "$REMOTE" "sudo systemctl status aicrypto-web --no-pager"
    echo ""
    echo "=== Port ==="
    ssh "$REMOTE" "sudo ss -ltnp | grep $APP_PORT"
    ;;

  restart)
    echo "Restarting aicrypto-web..."
    ssh "$REMOTE" "sudo systemctl restart aicrypto-web && sleep 2 && sudo systemctl status aicrypto-web --no-pager"
    ;;

  logs)
    n="${2:-100}"
    ssh "$REMOTE" "tail -n $n $LOG_DIR/web.log"
    ;;

  error-logs)
    n="${2:-100}"
    ssh "$REMOTE" "tail -n $n $LOG_DIR/web-error.log"
    ;;

  reload-nginx)
    ssh "$REMOTE" "sudo nginx -t && sudo systemctl reload nginx && echo 'Nginx reloaded OK'"
    ;;

  cert-check)
    ssh "$REMOTE" "sudo certbot certificates 2>/dev/null | grep -A3 '$DOMAIN'"
    ;;

  cert-renew)
    ssh "$REMOTE" "sudo certbot renew --dry-run"
    ;;

  health)
    echo "=== Local loopback ==="
    ssh "$REMOTE" "curl -sI http://127.0.0.1:$APP_PORT/ | head -5"
    echo ""
    echo "=== Public HTTPS ==="
    curl -sI "https://$DOMAIN" | head -5
    echo ""
    echo "=== Certificate ==="
    ssh "$REMOTE" "sudo certbot certificates 2>/dev/null | grep -A3 '$DOMAIN'"
    echo ""
    echo "=== Service ==="
    ssh "$REMOTE" "systemctl is-active aicrypto-web"
    ;;

  help|*)
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  status         Show service and port status"
    echo "  restart        Restart aicrypto-web"
    echo "  logs [N]       Show last N log lines (default 100)"
    echo "  error-logs [N] Show last N error log lines"
    echo "  reload-nginx   Test and reload Nginx config"
    echo "  cert-check     Check certificate expiry"
    echo "  cert-renew     Dry-run certificate renewal"
    echo "  health         Full health check"
    ;;
esac
