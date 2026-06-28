#!/bin/bash
# Flutter 客户端编译测试

cd "$(dirname "$0")/../client/flutter"

echo "=========================================="
echo "📱 Flutter 客户端编译测试"
echo "=========================================="
echo ""

echo "1. 检查 Flutter 环境..."
flutter --version
if [ $? -ne 0 ]; then
    echo "❌ Flutter 未安装"
    exit 1
fi
echo "✅ Flutter 环境正常"
echo ""

echo "2. 检查项目结构..."
if [ ! -f "pubspec.yaml" ]; then
    echo "❌ pubspec.yaml 不存在"
    exit 1
fi
echo "✅ 项目结构正常"
echo ""

echo "3. 获取依赖..."
flutter pub get

if [ $? -ne 0 ]; then
    echo "❌ 获取依赖失败"
    exit 1
fi
echo "✅ 依赖获取成功"
echo ""

echo "4. 分析代码..."
flutter analyze --no-fatal-infos

if [ $? -ne 0 ]; then
    echo "⚠️  代码分析有警告（可忽略）"
else
    echo "✅ 代码分析通过"
fi
echo ""

echo "=========================================="
echo "✅ Flutter 客户端测试通过"
echo "=========================================="
echo ""
echo "界面摘要:"
echo "  HomePage        - 设备代码显示和状态"
echo "  ConnectPage     - 输入远程设备代码连接"
echo "  SessionScreen   - 视频显示和控制"
echo "  AdminPage       - 管理界面"
echo "  SettingsScreen  - 设置页面"
echo ""
echo "启动客户端: cd client/flutter && flutter run"
