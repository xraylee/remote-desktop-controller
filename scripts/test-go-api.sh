#!/bin/bash
# Go API 服务启动测试

cd "$(dirname "$0")/../services/api"

echo "=========================================="
echo "🚀 Go API 服务启动测试"
echo "=========================================="
echo ""

echo "1. 检查 Go 环境..."
go version
if [ $? -ne 0 ]; then
    echo "❌ Go 未安装"
    exit 1
fi
echo "✅ Go 环境正常"
echo ""

echo "2. 检查项目结构..."
if [ ! -f "cmd/api/main.go" ]; then
    echo "❌ main.go 不存在"
    exit 1
fi
echo "✅ 项目结构正常"
echo ""

echo "3. 编译 API 服务..."
go build -o api-test ./cmd/api/main.go

if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi
echo "✅ 编译成功"
echo ""

echo "4. 清理..."
rm -f api-test

echo "=========================================="
echo "✅ Go API 服务测试通过"
echo "=========================================="
echo ""
echo "API 端点摘要:"
echo "  POST   /api/v1/teams/{teamID}/devices        - 注册设备"
echo "  GET    /api/v1/teams/{teamID}/devices        - 获取设备列表"
echo "  GET    /api/v1/teams/{teamID}/devices/{code} - 获取设备详情"
echo "  DELETE /api/v1/teams/{teamID}/devices/{code} - 删除设备"
echo "  GET    /api/v1/teams/{teamID}/sessions       - 获取会话列表"
echo "  GET    /api/v1/teams/{teamID}/audit          - 获取审计日志"
echo ""
echo "启动服务: cd services/api && go run cmd/api/main.go"
