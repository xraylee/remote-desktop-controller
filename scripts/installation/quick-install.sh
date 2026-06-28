#!/bin/bash
# RDCS 一键安装脚本 - 直接复制到 Mac 终端运行

echo "开始安装 RDCS 所有依赖..."
echo ""

# 进入项目目录
cd /Users/lc/Development/source/remote-desktop-controller

# 执行安装脚本
./install-china-mirror.sh

echo ""
echo "安装完成！运行以下命令验证："
echo "  rustc --version"
echo "  go version"
echo "  flutter --version"
