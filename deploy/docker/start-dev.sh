#!/usr/bin/env bash
# RDCS 开发环境一键启动脚本
# Usage: ./start-dev.sh [mode]
# mode: minimal (默认) | full | down | status | logs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()    { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $*" >&2; }
log_section() { echo -e "\n${BLUE}=== $* ===${NC}"; }

MODE="${1:-minimal}"

cd "$SCRIPT_DIR"

# -----------------------------------------------
# 检查前置条件
# -----------------------------------------------
check_prerequisites() {
    log_section "检查前置条件"

    if ! command -v docker &> /dev/null; then
        log_error "Docker 未安装或未启动，请启动 Docker Desktop"
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Docker Desktop 未运行，请先启动 Docker Desktop"
        exit 1
    fi
    log_info "Docker Desktop 运行正常 ✓"

    if ! command -v docker compose &> /dev/null; then
        log_error "docker compose 未找到"
        exit 1
    fi
    log_info "docker compose 可用 ✓"
}

# -----------------------------------------------
# 设置环境变量
# -----------------------------------------------
setup_env() {
    log_section "配置环境变量"

    if [ ! -f ".env" ]; then
        log_warn ".env 文件不存在，从模板创建..."
        cp .env.example .env
        log_info ".env 文件已创建，请根据需要编辑"
    else
        log_info ".env 文件存在 ✓"
    fi

    # 检查 JWT 密钥
    if grep -q "^JWT_PRIVATE_KEY=$" .env || ! grep -q "JWT_PRIVATE_KEY" .env; then
        log_warn "JWT 密钥未配置，开发环境不需要（可选）"
    else
        log_info "JWT 密钥已配置 ✓"
    fi
}

# -----------------------------------------------
# 启动最小化模式（仅数据库）
# -----------------------------------------------
start_minimal() {
    log_section "启动最小化模式（PostgreSQL + Redis）"

    docker compose -f docker-compose.minimal.yml up -d

    log_section "等待服务就绪"
    echo -n "等待 PostgreSQL..."
    for i in {1..30}; do
        if docker exec rdcs-postgres pg_isready -U rdcs -q 2>/dev/null; then
            echo " ✓"
            break
        fi
        echo -n "."
        sleep 2
        if [ $i -eq 30 ]; then
            echo " ✗"
            log_error "PostgreSQL 启动超时，请查看日志: docker compose -f docker-compose.minimal.yml logs postgres"
            exit 1
        fi
    done

    echo -n "等待 Redis..."
    for i in {1..15}; do
        if docker exec rdcs-redis redis-cli ping &>/dev/null; then
            echo " ✓"
            break
        fi
        echo -n "."
        sleep 1
        if [ $i -eq 15 ]; then
            echo " ✗"
            log_error "Redis 启动超时，请查看日志: docker compose -f docker-compose.minimal.yml logs redis"
            exit 1
        fi
    done

    log_section "服务信息"
    echo ""
    echo "  数据库连接:"
    echo "    DATABASE_URL=postgres://rdcs:rdcs_dev@localhost:5432/rdcs?sslmode=disable"
    echo "    redis-cli -h localhost -p 6379"
    echo ""
    echo "  psql 快速连接:"
    echo "    docker exec -it rdcs-postgres psql -U rdcs -d rdcs"
    echo ""
    echo "  在主机上运行服务:"
    echo "    # Terminal 1 - API"
    echo "    cd $PROJECT_ROOT/services/api && go run ./cmd/api"
    echo ""
    echo "    # Terminal 2 - Signaling"
    echo "    cd $PROJECT_ROOT && cargo run --bin rdcs-signaling"
    echo ""
    echo "    # Terminal 3 - Relay"
    echo "    cd $PROJECT_ROOT && cargo run --bin rdcs-relay -- --hmac-secret dev_secret"
    echo ""
}

# -----------------------------------------------
# 启动完整开发模式（所有服务 in Docker）
# -----------------------------------------------
start_full() {
    log_section "启动完整开发环境（所有服务 in Docker）"
    log_warn "注意: Rust 服务首次构建需要 5-15 分钟"

    docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d

    log_section "等待服务就绪"
    echo -n "等待 PostgreSQL..."
    for i in {1..30}; do
        if docker exec rdcs-postgres pg_isready -U rdcs -q 2>/dev/null; then
            echo " ✓"; break
        fi
        echo -n "."; sleep 2
    done

    echo -n "等待 API..."
    for i in {1..30}; do
        if curl -sf http://localhost:8080/healthz &>/dev/null; then
            echo " ✓"; break
        fi
        echo -n "."; sleep 3
    done

    log_section "服务端点"
    echo ""
    echo "  API:       http://localhost:8080"
    echo "  Signaling: ws://localhost:8443"
    echo "  Relay:     udp://localhost:3478"
    echo "  Web:       http://localhost:3000"
    echo ""
    echo "  测试 API:"
    echo "    curl http://localhost:8080/healthz"
    echo ""
}

# -----------------------------------------------
# 停止服务
# -----------------------------------------------
stop_services() {
    log_section "停止服务"

    if docker compose -f docker-compose.minimal.yml ps -q 2>/dev/null | grep -q .; then
        docker compose -f docker-compose.minimal.yml down
        log_info "最小化服务已停止"
    fi

    if docker compose ps -q 2>/dev/null | grep -q .; then
        docker compose down
        log_info "完整服务已停止"
    fi

    log_info "所有 RDCS 服务已停止 ✓"
}

# -----------------------------------------------
# 查看状态
# -----------------------------------------------
show_status() {
    log_section "RDCS 服务状态"

    echo ""
    docker compose -f docker-compose.minimal.yml ps 2>/dev/null || true
    docker compose ps 2>/dev/null || true

    echo ""
    log_section "容器资源使用"
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}" \
        $(docker ps --filter "name=rdcs" --format "{{.Names}}") 2>/dev/null || log_warn "没有运行中的 RDCS 容器"

    echo ""
    log_section "服务健康检查"

    services=(
        "PostgreSQL:docker exec rdcs-postgres pg_isready -U rdcs -q"
        "Redis:docker exec rdcs-redis redis-cli ping > /dev/null"
        "API:curl -sf http://localhost:8080/healthz > /dev/null"
        "Relay Health:curl -sf http://localhost:9091/health > /dev/null"
    )

    for service_check in "${services[@]}"; do
        IFS=':' read -r name check <<< "$service_check"
        if eval "$check" 2>/dev/null; then
            echo "  ✓ $name"
        else
            echo "  ✗ $name (未运行)"
        fi
    done
    echo ""
}

# -----------------------------------------------
# 查看日志
# -----------------------------------------------
show_logs() {
    SERVICE="${2:-}"
    if [ -n "$SERVICE" ]; then
        docker compose logs -f "$SERVICE"
    else
        docker compose logs -f
    fi
}

# -----------------------------------------------
# 主逻辑
# -----------------------------------------------
check_prerequisites

case "$MODE" in
    minimal)
        setup_env
        start_minimal
        ;;
    full)
        setup_env
        start_full
        ;;
    down|stop)
        stop_services
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs "$@"
        ;;
    *)
        echo "Usage: $0 [minimal|full|down|status|logs]"
        echo ""
        echo "  minimal  启动仅数据库服务（默认，推荐开发使用）"
        echo "  full     启动所有服务（API/Signaling/Relay 在 Docker 中）"
        echo "  down     停止所有服务"
        echo "  status   查看服务状态和健康检查"
        echo "  logs     查看实时日志"
        exit 1
        ;;
esac
