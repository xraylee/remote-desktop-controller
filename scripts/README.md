# 脚本目录

本目录包含项目的各类脚本工具。

## 目录结构

```
scripts/
├── README.md                        # 本文件
├── installation/                    # 安装脚本
│   ├── auto-install-all.sh         # 自动安装所有依赖
│   ├── check-and-install.sh        # 检查并安装缺失依赖
│   └── quick-install.sh            # 快速安装
├── testing/                         # 测试脚本（暂无）
├── validation/                      # 验证脚本（暂无）
├── diagnostics/                     # 诊断脚本
│   └── diagnose-videotoolbox-crash.sh  # VideoToolbox 崩溃诊断
├── check-compilation.sh             # 编译检查
├── run-unit-tests.sh               # 运行单元测试
├── run-local-roundtrip.sh          # 硬件加速回环测试
├── run-local-roundtrip-mock.sh     # Mock 回环测试
├── run-local-roundtrip-openh264.sh # OpenH264 回环测试
├── test-hardware-accel-gate.sh     # 硬件加速 feature gate 测试
├── health-check.sh                 # 健康检查
└── reset-dev.sh                    # 重置开发环境
```

## 使用指南

### 安装与配置

```bash
# 自动安装所有依赖
./scripts/installation/auto-install-all.sh

# 检查并安装缺失依赖
./scripts/installation/check-and-install.sh

# 快速安装（中国镜像优化）
./scripts/installation/quick-install.sh
```

### 开发测试流程

```bash
# 1. 编译检查
./scripts/check-compilation.sh

# 2. 运行单元测试
./scripts/run-unit-tests.sh

# 3. Mock 回环测试（推荐）
./scripts/run-local-roundtrip-mock.sh

# 4. OpenH264 软件编码测试
./scripts/run-local-roundtrip-openh264.sh

# 5. Feature gate 测试
./scripts/test-hardware-accel-gate.sh

# 6. 硬件加速测试（可选，当前有问题）
# ./scripts/run-local-roundtrip.sh
```

### 诊断工具

```bash
# 诊断 VideoToolbox 崩溃
./scripts/diagnostics/diagnose-videotoolbox-crash.sh

# 系统健康检查
./scripts/health-check.sh
```

### 开发环境管理

```bash
# 重置开发环境
./scripts/reset-dev.sh
```

## 脚本约定

1. **命名规范**
   - 使用 kebab-case（小写+连字符）
   - 描述性名称，体现脚本用途
   - 添加 `.sh` 扩展名

2. **可执行权限**
   ```bash
   chmod +x scripts/*.sh
   chmod +x scripts/**/*.sh
   ```

3. **错误处理**
   - 使用 `set -e` 在错误时退出
   - 使用 `set -u` 检测未定义变量
   - 提供清晰的错误信息

4. **文档要求**
   - 脚本开头包含用途说明
   - 复杂脚本提供使用示例
   - 注释关键步骤

## 添加新脚本

1. 根据用途放到相应子目录
2. 遵循命名和编码规范
3. 添加可执行权限
4. 更新本 README
5. 在相关文档中引用

## 相关文档

- [测试指南](../docs/testing/TESTING_GUIDELINES.md)
- [安装文档](../docs/installation/)
- [项目组织说明](../PROJECT_ORGANIZATION.md)
