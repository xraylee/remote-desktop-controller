#!/bin/bash
# RDCS 后端服务快速部署脚本

set -e

echo "=== RDCS 后端服务部署 ==="
echo ""

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
DOCKER_DIR="$PROJECT_ROOT/deploy/docker"

cd "$DOCKER_DIR"

# 步骤 1: 检查 Docker
echo -e "${YELLOW}[1/6] 检查 Docker...${NC}"
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ 未安装 Docker${NC}"
    echo "请访问 https://www.docker.com/get-started 安装 Docker Desktop"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo -e "${RED}❌ Docker 未启动${NC}"
    echo "请启动 Docker Desktop"
    exit 1
fi

echo -e "${GREEN}✅ Docker 已就绪${NC}"
echo ""

# 步骤 2: 创建配置文件
echo -e "${YELLOW}[2/6] 配置环境变量...${NC}"
if [ ! -f .env ]; then
    echo -e "${BLUE}创建 .env 配置文件...${NC}"
    cp .env.example .env
    echo -e "${GREEN}✅ .env 已创建（使用默认配置）${NC}"
else
    echo -e "${GREEN}✅ .env 已存在${NC}"
fi
echo ""

# 步骤 3: 停止旧容器（如果存在）
echo -e "${YELLOW}[3/6] 清理旧容器...${NC}"
docker compose -f docker-compose.yml -f docker-compose.dev.yml down 2>/dev/null || true
echo -e "${GREEN}✅ 清理完成${NC}"
echo ""

# 步骤 4: 构建镜像
echo -e "${YELLOW}[4/6] 构建 Docker 镜像（可能需要几分钟）...${NC}"
docker compose -f docker-compose.yml -f docker-compose.dev.yml build

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ 镜像构建成功${NC}"
else
    echo -e "${RED}❌ 镜像构建失败${NC}"
    exit 1
fi
echo ""

# 步骤 5: 启动服务
echo -e "${YELLOW}[5/6] 启动所有服务...${NC}"
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ 服务已启动${NC}"
else
    echo -e "${RED}❌ 服务启动失败${NC}"
    exit 1
fi
echo ""

# 步骤 6: 等待服务就绪
echo -e "${YELLOW}[6/6] 等待服务就绪...${NC}"
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

echo -e "${BLUE}等待 API 服务...${NC}"
for i in {1..30}; do
    if curl -sf http://localhost:8080/healthz &>/dev/null; then
        echo -e "${GREEN}✅ API 服务已就绪${NC}"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo -e "${RED}❌ API 服务启动超时${NC}"
        docker logs rdcs-api --tail 50
        exit 1
    fi
done

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         ✅ RDCS 后端服务部署成功！                 ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}📊 服务状态:${NC}"
echo ""
docker compose -f docker-compose.yml -f docker-compose.dev.yml ps
echo ""
echo -e "${BLUE}🌐 服务地址:${NC}"
echo -e "  • API 服务:       ${GREEN}http://localhost:8080${NC}"
echo -e "  • 健康检查:       ${GREEN}http://localhost:8080/healthz${NC}"
echo -e "  • Web 管理台:     ${GREEN}http://localhost:3000${NC}"
echo -e "  • MinIO 控制台:   ${GREEN}http://localhost:9001${NC}"
echo -e "  • PostgreSQL:     ${GREEN}localhost:5432${NC}"
echo -e "  • Redis:          ${GREEN}localhost:6379${NC}"
echo ""
echo -e "${BLUE}📝 常用命令:${NC}"
echo -e "  • 查看日志:   ${YELLOW}docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f${NC}"
echo -e "  • 查看 API:   ${YELLOW}docker logs -f rdcs-api${NC}"
echo -e "  • 停止服务:   ${YELLOW}docker compose -f docker-compose.yml -f docker-compose.dev.yml down${NC}"
echo -e "  • 重启服务:   ${YELLOW}docker compose -f docker-compose.yml -f docker-compose.dev.yml restart${NC}"
echo ""
echo -e "${BLUE}🔧 数据库连接信息:${NC}"
echo -e "  • Host:     localhost"
echo -e "  • Port:     5432"
echo -e "  • Database: rdcs"
echo -e "  • User:     rdcs"
echo -e "  • Password: rdcs_dev"
echo ""
echo -e "${BLUE}📱 下一步:${NC}"
echo -e "  1. 配置 Flutter 客户端连接到 ${GREEN}http://localhost:8080${NC}"
echo -e "  2. 在两台 Mac 上运行客户端"
echo -e "  3. 测试远程桌面连接"
echo ""
