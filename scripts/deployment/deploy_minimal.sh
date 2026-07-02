#!/bin/bash
# RDCS 最小化部署脚本 - 仅启动数据库，API 在本地运行

set -e

echo "=== RDCS 最小化部署（仅数据库）==="
echo ""

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DOCKER_DIR="$PROJECT_ROOT/deploy/docker"

cd "$DOCKER_DIR"

# 步骤 1: 检查 Docker
echo -e "${YELLOW}[1/4] 检查 Docker...${NC}"
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ 未安装 Docker${NC}"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo -e "${RED}❌ Docker 未启动${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Docker 已就绪${NC}"
echo ""

# 步骤 2: 创建配置文件
echo -e "${YELLOW}[2/4] 配置环境变量...${NC}"
if [ ! -f .env ]; then
    cp .env.example .env
    echo -e "${GREEN}✅ .env 已创建${NC}"
else
    echo -e "${GREEN}✅ .env 已存在${NC}"
fi
echo ""

# 步骤 3: 停止旧容器
echo -e "${YELLOW}[3/4] 清理旧容器...${NC}"
docker compose -f docker-compose.minimal.yml down 2>/dev/null || true
echo -e "${GREEN}✅ 清理完成${NC}"
echo ""

# 步骤 4: 启动数据库服务
echo -e "${YELLOW}[4/4] 启动数据库服务...${NC}"
docker compose -f docker-compose.minimal.yml up -d

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ 数据库服务已启动${NC}"
else
    echo -e "${RED}❌ 启动失败${NC}"
    exit 1
fi
echo ""

# 等待 PostgreSQL 就绪
echo -e "${BLUE}等待 PostgreSQL...${NC}"
for i in {1..30}; do
    if docker exec rdcs-postgres pg_isready -U rdcs -d rdcs &>/dev/null; then
        echo -e "${GREEN}✅ PostgreSQL 已就绪${NC}"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo -e "${RED}❌ PostgreSQL 启动超时${NC}"
        exit 1
    fi
done

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║       ✅ 数据库服务部署成功！                      ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}📊 运行中的服务:${NC}"
docker compose -f docker-compose.minimal.yml ps
echo ""
echo -e "${BLUE}🔧 数据库连接信息:${NC}"
echo -e "  • Host:     ${GREEN}localhost${NC}"
echo -e "  • Port:     ${GREEN}5432${NC}"
echo -e "  • Database: ${GREEN}rdcs${NC}"
echo -e "  • User:     ${GREEN}rdcs${NC}"
echo -e "  • Password: ${GREEN}rdcs_dev${NC}"
echo ""
echo -e "${BLUE}📝 下一步 - 在本地运行 API 服务:${NC}"
echo -e "  ${YELLOW}cd services/api${NC}"
echo -e "  ${YELLOW}go run cmd/api/main.go${NC}"
echo ""
echo -e "${BLUE}💡 停止服务:${NC}"
echo -e "  ${YELLOW}cd deploy/docker && docker compose -f docker-compose.minimal.yml down${NC}"
echo ""
