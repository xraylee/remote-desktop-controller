#!/bin/bash
# 查看 RDCS 后端服务日志

cd "$(dirname "$0")/deploy/docker"

if [ "$1" == "api" ]; then
    echo "📝 API 服务日志 (Ctrl+C 退出):"
    docker logs -f rdcs-api
elif [ "$1" == "signaling" ]; then
    echo "📝 信令服务日志 (Ctrl+C 退出):"
    docker logs -f rdcs-signaling
elif [ "$1" == "relay" ]; then
    echo "📝 中继服务日志 (Ctrl+C 退出):"
    docker logs -f rdcs-relay
elif [ "$1" == "postgres" ]; then
    echo "📝 PostgreSQL 日志 (Ctrl+C 退出):"
    docker logs -f rdcs-postgres
else
    echo "📝 所有服务日志 (Ctrl+C 退出):"
    docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f
fi
