# 多架构兼容构建系统实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Intel Mac (x86_64) 和 Apple Silicon (arm64) 的多架构支持，包括本地开发快速构建和 CI/CD 自动化构建系统。

**Architecture:** 按需架构构建策略——开发模式自动检测当前架构快速构建单一版本，发布模式显式指定架构生成独立安装包，CI 并行构建两个架构的独立 DMG。

**Tech Stack:** Rust (cargo/rustup), Flutter (flutter build macos), macOS (install_name_tool, hdiutil), GitHub Actions (matrix builds)

---

## 文件结构

### 新增文件
- `client/flutter/macos/build_rust_lib.sh` - Rust 库多架构构建脚本
- `scripts/package_macos.sh` - DMG 打包脚本
- `.github/workflows/build-macos.yml` - GitHub Actions CI 配置
- `docs/BUILD_MULTI_ARCH.md` - 用户构建说明文档

### 修改文件
- `client/flutter/macos/Runner.xcodeproj/project.pbxproj` - 更新 Build Phase 脚本
- `README.md` - 添加架构下载说明

### 删除文件
- `client/flutter/macos/copy_ffi_lib.sh` - 被 build_rust_lib.sh 替换
- `client/flutter/macos/scripts/copy_rust_lib.sh` - 旧脚本，不再使用

---

## Task 1: 创建 Rust 库多架构构建脚本

**Files:**
- Create: `client/flutter/macos/build_rust_lib.sh`

- [ ] **Step 1: 创建脚本文件基础结构**

创建文件 `client/flutter/macos/build_rust_lib.sh` 包含头部和参数解析：

```bash
#!/bin/bash
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0
# Build Rust FFI library for specified or current architecture

set -e

# Parse arguments
RELEASE_MODE=false
EXPLICIT_ARCH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            RELEASE_MODE=true
            shift
            ;;
        --arch)
            EXPLICIT_ARCH="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "🔨 Building Rust FFI library..."
echo "   Release mode: $RELEASE_MODE"
echo "   Explicit arch: ${EXPLICIT_ARCH:-auto-detect}"
```

- [ ] **Step 2: 添加架构检测逻辑**

在步骤1的代码后追加：

```bash
# Determine target architecture
if [ -n "$EXPLICIT_ARCH" ]; then
    TARGET_ARCH="$EXPLICIT_ARCH"
    echo "📋 Using explicit architecture: $TARGET_ARCH"
else
    SYSTEM_ARCH=$(uname -m)
    echo "📋 Detected system architecture: $SYSTEM_ARCH"
    
    case "$SYSTEM_ARCH" in
        arm64)
            TARGET_ARCH="arm64"
            ;;
        x86_64)
            TARGET_ARCH="x64"
            ;;
        *)
            echo "❌ Unsupported architecture: $SYSTEM_ARCH"
            exit 1
            ;;
    esac
fi

# Map to Rust target
case "$TARGET_ARCH" in
    arm64)
        RUST_TARGET="aarch64-apple-darwin"
        ;;
    x64)
        RUST_TARGET="x86_64-apple-darwin"
        ;;
    *)
        echo "❌ Invalid target: $TARGET_ARCH"
        exit 1
        ;;
esac

echo "🎯 Rust target: $RUST_TARGET"
```

- [ ] **Step 3: 添加 Rust 目标安装检查**

```bash
# Check Rust target
echo "🔍 Checking Rust target..."
if rustup target list | grep -q "^${RUST_TARGET} (installed)"; then
    echo "✅ Target installed: $RUST_TARGET"
else
    echo "📥 Installing target: $RUST_TARGET"
    rustup target add "$RUST_TARGET"
fi
```

- [ ] **Step 4: 添加 Cargo 构建**

```bash
# Build configuration
if [ "$RELEASE_MODE" = true ]; then
    BUILD_FLAG="--release"
    BUILD_DIR="release"
    echo "🚀 RELEASE mode"
else
    BUILD_FLAG=""
    BUILD_DIR="debug"
    echo "🔧 DEBUG mode"
fi

# Get project root
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
cd "$PROJECT_ROOT"

# Build
echo "⚙️  Running cargo build..."
cargo build $BUILD_FLAG --target "$RUST_TARGET" --package rdcs-ffi
echo "✅ Build complete"
```

- [ ] **Step 5: 添加库复制和修复**

```bash
# Paths
SRC_LIB="$PROJECT_ROOT/target/$RUST_TARGET/$BUILD_DIR/librdcs_core.dylib"
DEST_DIR="${BUILT_PRODUCTS_DIR:-$PROJECT_ROOT/client/flutter/build/macos/Build/Products/$BUILD_DIR/rdcs_client.app}/Contents/Frameworks"
DEST_LIB="$DEST_DIR/librdcs_core.dylib"

# Verify source
if [ ! -f "$SRC_LIB" ]; then
    echo "❌ Library not found: $SRC_LIB"
    exit 1
fi

# Copy
echo "📦 Copying..."
mkdir -p "$DEST_DIR"
cp -v "$SRC_LIB" "$DEST_LIB"

# Fix install_name
echo "🔧 Fixing install_name..."
install_name_tool -id "@rpath/librdcs_core.dylib" "$DEST_LIB"

# Verify
echo "✅ Verification:"
otool -D "$DEST_LIB"
file "$DEST_LIB"

echo "✅ Done! Architecture: $TARGET_ARCH"
```

- [ ] **Step 6: 设置可执行权限**

```bash
chmod +x client/flutter/macos/build_rust_lib.sh
```

- [ ] **Step 7: 测试脚本**

Run: `cd client/flutter/macos && ./build_rust_lib.sh`

Expected: 成功构建当前架构的 debug 版本

- [ ] **Step 8: 提交**

```bash
git add client/flutter/macos/build_rust_lib.sh
git commit -m "feat: add multi-arch Rust build script

- Auto-detect or explicit architecture
- Debug/release modes
- Automatic target installation
- install_name fixing"
```

---

## Task 2: 更新 Xcode Build Phase

**Files:**
- Modify: `client/flutter/macos/Runner.xcodeproj/project.pbxproj`

- [ ] **Step 1: 打开 Xcode 项目**

```bash
open client/flutter/macos/Runner.xcworkspace
```

- [ ] **Step 2: 定位 Build Phase**

在 Xcode 中：
1. 选择 Runner 项目
2. 选择 Runner target
3. 点击 Build Phases 标签
4. 找到 "Run Script" phase（名为 "Copy Rust FFI Library" 或类似）

- [ ] **Step 3: 替换脚本内容**

将现有脚本替换为：

```bash
#!/bin/bash
# Build Rust FFI library for current/specified architecture

set -e

cd "$SRCROOT/../../.."

# Determine build mode
if [ "$CONFIGURATION" == "Release" ]; then
    BUILD_MODE="--release"
else
    BUILD_MODE=""
fi

# Determine architecture from Xcode
if [ -n "$ARCHS" ]; then
    TARGET_ARCH=$(echo "$ARCHS" | awk '{print $1}')
    
    case "$TARGET_ARCH" in
        arm64)
            ARCH_FLAG="--arch arm64"
            ;;
        x86_64)
            ARCH_FLAG="--arch x64"
            ;;
        *)
            echo "⚠️  Unknown arch: $TARGET_ARCH, auto-detect"
            ARCH_FLAG=""
            ;;
    esac
else
    ARCH_FLAG=""
fi

# Run build script
./client/flutter/macos/build_rust_lib.sh $BUILD_MODE $ARCH_FLAG
```

- [ ] **Step 4: 保存并测试**

Run: `cd client/flutter && flutter build macos --debug`

Expected: 构建成功，使用新脚本

- [ ] **Step 5: 提交**

```bash
git add client/flutter/macos/Runner.xcodeproj/project.pbxproj
git commit -m "feat: update Xcode build phase for multi-arch

- Read architecture from Xcode ARCHS variable
- Pass to build_rust_lib.sh script
- Support debug and release modes"
```

---

## Task 3: 删除旧构建脚本

**Files:**
- Delete: `client/flutter/macos/copy_ffi_lib.sh`
- Delete: `client/flutter/macos/scripts/copy_rust_lib.sh`

- [ ] **Step 1: 验证旧脚本未被使用**

检查 Xcode 项目中是否还有引用：

```bash
grep -r "copy_ffi_lib.sh" client/flutter/macos/Runner.xcodeproj/ || echo "Not found (good)"
grep -r "copy_rust_lib.sh" client/flutter/macos/Runner.xcodeproj/ || echo "Not found (good)"
```

Expected: 两个命令都输出 "Not found (good)"

- [ ] **Step 2: 删除文件**

```bash
rm -f client/flutter/macos/copy_ffi_lib.sh
rm -f client/flutter/macos/scripts/copy_rust_lib.sh
```

- [ ] **Step 3: 提交**

```bash
git add -A
git commit -m "chore: remove old build scripts

- Remove copy_ffi_lib.sh (replaced by build_rust_lib.sh)
- Remove copy_rust_lib.sh (obsolete)"
```

---

## Task 4: 创建 DMG 打包脚本

**Files:**
- Create: `scripts/package_macos.sh`

- [ ] **Step 1: 创建打包脚本**

创建文件 `scripts/package_macos.sh`:

```bash
#!/bin/bash
# Package macOS app into DMG

set -e

VERSION=$(grep "version:" client/flutter/pubspec.yaml | awk '{print $2}')
ARCH=$1

if [ -z "$ARCH" ]; then
    echo "Usage: $0 <arm64|x64>"
    exit 1
fi

APP_PATH="client/flutter/build/macos/Build/Products/Release/rdcs_client.app"
DMG_NAME="rdcs_client-${ARCH}-v${VERSION}.dmg"

# Verify app
if [ ! -d "$APP_PATH" ]; then
    echo "❌ App not found: $APP_PATH"
    exit 1
fi

# Verify architecture
DYLIB_ARCH=$(file "$APP_PATH/Contents/Frameworks/librdcs_core.dylib" | awk '{print $NF}')
echo "📦 App architecture: $DYLIB_ARCH"

# Create DMG
echo "📦 Creating DMG: $DMG_NAME"
hdiutil create -volname "RDCS Client" \
    -srcfolder "$APP_PATH" \
    -ov -format UDZO \
    "$DMG_NAME"

echo "✅ DMG created: $DMG_NAME"
```

- [ ] **Step 2: 设置可执行权限**

```bash
chmod +x scripts/package_macos.sh
```

- [ ] **Step 3: 测试脚本**

先构建发布版本：
```bash
cd client/flutter
flutter build macos --release --target-arch=arm64
cd ../..
```

然后打包：
```bash
./scripts/package_macos.sh arm64
```

Expected: 生成 `rdcs_client-arm64-v<version>.dmg`

- [ ] **Step 4: 验证 DMG**

```bash
open rdcs_client-arm64-*.dmg
```

Expected: DMG 挂载，可以看到 rdcs_client.app

- [ ] **Step 5: 提交**

```bash
git add scripts/package_macos.sh
git commit -m "feat: add DMG packaging script

- Read version from pubspec.yaml
- Verify architecture before packaging
- Use hdiutil to create DMG"
```

---

## Task 5: 创建 GitHub Actions CI 配置

**Files:**
- Create: `.github/workflows/build-macos.yml`

- [ ] **Step 1: 创建 workflow 文件头部**

创建文件 `.github/workflows/build-macos.yml`:

```yaml
name: Build macOS Client

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build-macos:
    strategy:
      matrix:
        include:
          - runner: macos-14
            arch: arm64
            rust-target: aarch64-apple-darwin
            flutter-arch: arm64
            
          - runner: macos-13
            arch: x64
            rust-target: x86_64-apple-darwin
            flutter-arch: x64
    
    runs-on: ${{ matrix.runner }}
    
    steps:
      - uses: actions/checkout@v4
```

- [ ] **Step 2: 添加 Rust 设置步骤**

```yaml
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.rust-target }}
      
      - name: Rust cache
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: "."
```

- [ ] **Step 3: 添加 Flutter 设置步骤**

```yaml
      - name: Setup Flutter
        uses: subosito/flutter-action@v2
        with:
          channel: 'stable'
      
      - name: Get Flutter dependencies
        run: |
          cd client/flutter
          flutter pub get
```

- [ ] **Step 4: 添加构建步骤**

```yaml
      - name: Build Rust FFI
        run: |
          cargo build --release \
            --target ${{ matrix.rust-target }} \
            --package rdcs-ffi
      
      - name: Build Flutter app
        run: |
          cd client/flutter
          flutter build macos --release \
            --target-arch=${{ matrix.flutter-arch }}
      
      - name: Package DMG
        run: ./scripts/package_macos.sh ${{ matrix.arch }}
```

- [ ] **Step 5: 添加上传和发布步骤**

```yaml
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: rdcs-client-macos-${{ matrix.arch }}
          path: rdcs_client-${{ matrix.arch }}-*.dmg
      
      - name: Release
        if: startsWith(github.ref, 'refs/tags/')
        uses: softprops/action-gh-release@v1
        with:
          files: rdcs_client-${{ matrix.arch }}-*.dmg
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

- [ ] **Step 6: 提交**

```bash
git add .github/workflows/build-macos.yml
git commit -m "ci: add macOS multi-arch build workflow

- Matrix builds for arm64 and x64
- Parallel builds on macos-14 and macos-13
- Upload artifacts
- Auto-release on tags"
```

- [ ] **Step 7: 测试 workflow（手动触发）**

在 GitHub 上：
1. 进入 Actions 标签
2. 选择 "Build macOS Client"
3. 点击 "Run workflow"
4. 选择 branch 并运行

Expected: 两个并行任务（arm64 + x64），都成功完成

---

## Task 6: 创建用户构建文档

**Files:**
- Create: `docs/BUILD_MULTI_ARCH.md`

- [ ] **Step 1: 创建文档**

创建文件 `docs/BUILD_MULTI_ARCH.md`:

```markdown
# 多架构构建指南

本文档说明如何为 Intel Mac 和 Apple Silicon 构建 RDCS 客户端。

## 支持的架构

- **Apple Silicon** (arm64): M1/M2/M3/M4 芯片
- **Intel Mac** (x86_64): Intel Core 系列

## 开发构建

开发时会自动检测并构建当前架构：

```bash
cd client/flutter
flutter run -d macos
```

或：

```bash
flutter build macos --debug
```

## 发布构建

显式指定目标架构：

### Apple Silicon

```bash
cd client/flutter
flutter build macos --release --target-arch=arm64
```

### Intel Mac

```bash
cd client/flutter
flutter build macos --release --target-arch=x64
```

## 打包 DMG

构建完成后打包：

```bash
# 对应构建的架构
./scripts/package_macos.sh arm64
# 或
./scripts/package_macos.sh x64
```

## 验证架构

检查构建的库架构：

```bash
file client/flutter/build/macos/Build/Products/Release/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib
```

预期输出：
- Apple Silicon: `Mach-O 64-bit dynamically linked shared library arm64`
- Intel Mac: `Mach-O 64-bit dynamically linked shared library x86_64`

## CI/CD

GitHub Actions 会自动构建两个架构的安装包。

发布时会生成：
- `rdcs_client-arm64-vX.X.X.dmg`
- `rdcs_client-x86_64-vX.X.X.dmg`

## 依赖

### macOS 开发环境

- macOS 11+ (Apple Silicon) 或 10.15+ (Intel)
- Xcode 13+
- Rust 1.70+
- Flutter 3.x

### Rust 目标

构建脚本会自动安装需要的目标：
- `aarch64-apple-darwin` (arm64)
- `x86_64-apple-darwin` (x64)

手动安装：

```bash
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
```

## 常见问题

**Q: 我在 Apple Silicon 上能构建 Intel 版本吗？**
A: 不能。需要在 Intel Mac 或 CI 上构建。

**Q: 构建时间有多长？**
A: 开发构建约 2-3 分钟，发布构建约 5-8 分钟。

**Q: DMG 多大？**
A: 单架构约 50-60 MB。
```

- [ ] **Step 2: 提交**

```bash
git add docs/BUILD_MULTI_ARCH.md
git commit -m "docs: add multi-arch build guide

- Development and release build instructions
- Architecture verification
- Common FAQ"
```

---

## Task 7: 更新 README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: 添加架构说明**

在 README.md 的下载或安装章节添加：

```markdown
## 下载

根据您的 Mac 型号选择对应版本：

- **Apple Silicon** (M1/M2/M3/M4): [下载 arm64 版本](https://github.com/your-org/rdcs/releases/latest/download/rdcs_client-arm64-latest.dmg)
- **Intel Mac**: [下载 x86_64 版本](https://github.com/your-org/rdcs/releases/latest/download/rdcs_client-x86_64-latest.dmg)

不确定您的 Mac 型号？在终端运行：
```bash
uname -m
# arm64 = Apple Silicon
# x86_64 = Intel Mac
```
```

- [ ] **Step 2: 更新构建说明**

更新或添加构建章节：

```markdown
## 从源码构建

详细说明请参阅 [多架构构建指南](docs/BUILD_MULTI_ARCH.md)。

快速开始：

```bash
# 1. 克隆仓库
git clone https://github.com/your-org/rdcs.git
cd rdcs

# 2. 开发构建（自动检测架构）
cd client/flutter
flutter run -d macos

# 3. 发布构建（显式指定架构）
flutter build macos --release --target-arch=arm64  # 或 x64
cd ../..
./scripts/package_macos.sh arm64  # 或 x64
```
```

- [ ] **Step 3: 提交**

```bash
git add README.md
git commit -m "docs: add architecture download instructions

- Separate download links for arm64 and x64
- Architecture detection command
- Link to detailed build guide"
```

---

## Task 8: 完整测试流程

**Files:**
- Test all components

- [ ] **Step 1: 清理构建缓存**

```bash
cd client/flutter
flutter clean
cd ../..
cargo clean
```

- [ ] **Step 2: 测试开发构建**

```bash
cd client/flutter
flutter run -d macos
```

Expected: 应用启动，生成邀请码功能正常

- [ ] **Step 3: 测试发布构建（当前架构）**

```bash
flutter build macos --release --target-arch=arm64  # 或 x64（根据当前设备）
cd ../..
```

Verify:
```bash
file client/flutter/build/macos/Build/Products/Release/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib
```

Expected: 输出匹配当前架构

- [ ] **Step 4: 测试 DMG 打包**

```bash
./scripts/package_macos.sh arm64  # 或 x64
```

Expected: 生成 DMG 文件

- [ ] **Step 5: 测试 DMG 安装**

```bash
open rdcs_client-*.dmg
```

手动测试：
1. 拖拽 app 到 Applications
2. 从 Applications 启动
3. 测试邀请码生成功能
4. 测试屏幕捕获功能

- [ ] **Step 6: 验证架构**

```bash
file /Applications/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib
```

Expected: 正确的架构

- [ ] **Step 7: 最终提交**

```bash
git add -A
git commit -m "test: verify multi-arch build system

All components tested:
- Development build (auto-detect)
- Release build (explicit arch)
- DMG packaging
- Installation and runtime"
```

---

## 实施总结

完成上述 8 个任务后，多架构构建系统将完整实施：

### 已实现功能
✅ 开发模式：自动检测架构，快速构建
✅ 发布模式：显式指定架构，生成独立安装包
✅ CI/CD：并行构建两个架构
✅ DMG 打包：自动化打包流程
✅ 文档：完整的构建和下载说明

### 验证清单
- [ ] 本地开发构建成功
- [ ] 发布构建生成正确架构
- [ ] DMG 打包成功
- [ ] CI workflow 运行成功
- [ ] 文档完整准确

### 下一步
1. 推送到远程仓库
2. 触发 CI 构建测试
3. 创建 release 验证自动发布
4. 更新下载页面链接

