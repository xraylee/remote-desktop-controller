# 多架构兼容构建系统设计文档

**日期：** 2026-07-01  
**版本：** 1.0  
**作者：** Claude Code  
**状态：** 待审核

---

## 1. 概述

### 1.1 目标

为 RDCS 客户端实现 Intel Mac (x86_64) 和 Apple Silicon (arm64) 的多架构支持，满足以下需求：

- **开发环境：** 本地开发时快速构建，自动匹配当前架构
- **CI/CD：** 自动化构建分别生成两个架构的独立安装包
- **分发部署：** 用户根据设备架构下载对应的安装包

### 1.2 非目标

- **不构建 Universal Binary：** 不创建同时包含两个架构的单一应用包
- **不支持交叉编译：** arm64 机器上不编译 x86_64（反之亦然），各自在对应架构的 CI runner 上构建

---

## 2. 架构设计

### 2.1 构建模式

#### 开发模式（Development）

**触发条件：**
- `flutter run`
- `flutter build macos --debug`
- 本地手动构建

**行为：**
- 自动检测当前机器架构 (`uname -m`)
- 只编译匹配的单一架构：
  - Apple Silicon → `aarch64-apple-darwin`
  - Intel Mac → `x86_64-apple-darwin`
- 构建产物：单架构 dylib
- 构建时间：标准速度

**优点：** 快速迭代、节省时间、降低本地资源消耗

---

#### 发布模式（Release）

**触发条件：**
- `flutter build macos --release --target-arch=arm64`
- `flutter build macos --release --target-arch=x64`
- CI 环境构建

**行为：**
- 显式指定目标架构
- 只编译指定的单一架构
- 构建产物：
  - `rdcs_client-arm64.app` + `rdcs_client-arm64.dmg`
  - `rdcs_client-x86_64.app` + `rdcs_client-x86_64.dmg`
- 构建时间：标准速度（每个架构独立）

**优点：** 包体积小、下载快、CI 并行构建

---

### 2.2 构建流程

#### 开发构建流程

```
1. 检测当前架构
   ↓
2. 安装 Rust 目标（如未安装）
   - arm64 → rustup target add aarch64-apple-darwin
   - x86_64 → rustup target add x86_64-apple-darwin
   ↓
3. Cargo 构建
   - cargo build --target <架构> --package rdcs-ffi
   ↓
4. 复制 dylib 到 Frameworks
   - 源：target/<架构>/debug/librdcs_core.dylib
   - 目标：.app/Contents/Frameworks/librdcs_core.dylib
   ↓
5. 修复 install_name
   - install_name_tool -id "@rpath/librdcs_core.dylib"
   ↓
6. Flutter 打包
```

---

#### 发布构建流程

```
1. 接收目标架构参数
   - --target-arch=arm64 或 --target-arch=x64
   ↓
2. 验证 Rust 目标已安装
   ↓
3. Cargo 发布构建
   - cargo build --release --target <架构> --package rdcs-ffi
   ↓
4. 复制 dylib 到 Frameworks
   - 源：target/<架构>/release/librdcs_core.dylib
   - 目标：.app/Contents/Frameworks/librdcs_core.dylib
   ↓
5. 修复 install_name
   - install_name_tool -id "@rpath/librdcs_core.dylib"
   ↓
6. 代码签名（如配置）
   ↓
7. Flutter 打包
   ↓
8. 创建 DMG 安装包
   - rdcs_client-<架构>-<版本>.dmg
```

---

### 2.3 CI/CD 策略

#### GitHub Actions 矩阵构建

```yaml
strategy:
  matrix:
    include:
      - runner: macos-14          # Apple Silicon
        arch: arm64
        rust-target: aarch64-apple-darwin
        
      - runner: macos-13          # Intel
        arch: x64
        rust-target: x86_64-apple-darwin
```

**构建步骤：**
1. 并行启动两个 runner（arm64 + x86_64）
2. 每个 runner 独立执行：
   - 安装 Rust 工具链和目标
   - 构建 Rust FFI 库
   - 构建 Flutter 应用
   - 打包 DMG
3. 上传构建产物：
   - `rdcs_client-arm64-v1.0.0.dmg`
   - `rdcs_client-x86_64-v1.0.0.dmg`

**Release 资源：**
- 两个独立的 DMG 文件
- 下载页面说明：
  - Apple Silicon (M1/M2/M3/M4)：下载 arm64 版本
  - Intel Mac：下载 x86_64 版本

---

## 3. 组件设计

### 3.1 构建脚本

#### `client/flutter/macos/build_rust_lib.sh`

**新增脚本，替换现有的 `copy_ffi_lib.sh`**

**功能：**
- 检测构建模式（debug/release）
- 检测目标架构（自动检测或接收参数）
- 构建 Rust 库
- 复制到正确位置
- 修复 install_name

**接口：**
```bash
# 开发模式（自动检测架构）
./build_rust_lib.sh

# 发布模式（显式指定架构）
./build_rust_lib.sh --release --arch arm64
./build_rust_lib.sh --release --arch x64
```

**实现逻辑：**
```bash
1. 解析命令行参数
   - --release: 发布模式标志
   - --arch <arch>: 目标架构（arm64/x64）

2. 确定构建配置
   - 如果未指定 --arch，自动检测：uname -m
   - 将 arm64 → aarch64-apple-darwin
   - 将 x86_64 → x86_64-apple-darwin

3. 检查 Rust 目标
   - rustup target list | grep <target>
   - 如未安装：rustup target add <target>

4. 构建 Rust 库
   - cargo build [--release] --target <target> --package rdcs-ffi

5. 复制 dylib
   - 源路径：target/<target>/[debug|release]/librdcs_core.dylib
   - 目标路径：$BUILT_PRODUCTS_DIR/rdcs_client.app/Contents/Frameworks/

6. 修复 install_name
   - install_name_tool -id "@rpath/librdcs_core.dylib"

7. 验证
   - otool -D 确认 install_name 正确
```

---

### 3.2 Xcode 集成

#### 修改 `Runner.xcodeproj` Build Phases

**替换原有的 "Copy Rust FFI Library" 脚本：**

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

# Determine target architecture
# Flutter sets ARCHS during build, use it if available
if [ -n "$ARCHS" ]; then
    # Extract first architecture from ARCHS (e.g., "arm64" or "x86_64")
    TARGET_ARCH=$(echo "$ARCHS" | awk '{print $1}')
    
    # Normalize architecture names
    case "$TARGET_ARCH" in
        arm64)
            ARCH_FLAG="--arch arm64"
            ;;
        x86_64)
            ARCH_FLAG="--arch x64"
            ;;
        *)
            echo "⚠️  Unknown architecture: $TARGET_ARCH, using auto-detect"
            ARCH_FLAG=""
            ;;
    esac
else
    # Auto-detect if ARCHS not set
    ARCH_FLAG=""
fi

# Run the build script
./client/flutter/macos/build_rust_lib.sh $BUILD_MODE $ARCH_FLAG
```

**关键点：**
- 从 Xcode 的 `$ARCHS` 变量读取目标架构
- Flutter 构建时会设置此变量
- 转换为脚本参数传递

---

### 3.3 Flutter 构建命令

#### 本地开发

```bash
# 自动检测架构（推荐）
flutter run -d macos

# 或显式指定（如果需要）
flutter build macos --debug
```

---

#### 发布构建

```bash
# Apple Silicon
flutter build macos --release --target-arch=arm64

# Intel Mac
flutter build macos --release --target-arch=x64
```

**构建产物位置：**
- `build/macos/Build/Products/Release/rdcs_client.app`

---

### 3.4 打包脚本

#### `scripts/package_macos.sh`

**功能：** 创建 DMG 安装包

```bash
#!/bin/bash
# Package macOS app into DMG

set -e

VERSION=$(grep "version:" client/flutter/pubspec.yaml | awk '{print $2}')
ARCH=$1  # arm64 or x64

if [ -z "$ARCH" ]; then
    echo "Usage: $0 <arm64|x64>"
    exit 1
fi

APP_PATH="client/flutter/build/macos/Build/Products/Release/rdcs_client.app"
DMG_NAME="rdcs_client-${ARCH}-v${VERSION}.dmg"

# Verify app exists
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

**使用：**
```bash
# 先构建
flutter build macos --release --target-arch=arm64

# 再打包
./scripts/package_macos.sh arm64
```

---

## 4. CI/CD 实现

### 4.1 GitHub Actions 配置

**文件：** `.github/workflows/build-macos.yml`

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
          - runner: macos-14  # Apple Silicon
            arch: arm64
            rust-target: aarch64-apple-darwin
            flutter-arch: arm64
            
          - runner: macos-13  # Intel
            arch: x64
            rust-target: x86_64-apple-darwin
            flutter-arch: x64
    
    runs-on: ${{ matrix.runner }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.rust-target }}
      
      - name: Setup Flutter
        uses: subosito/flutter-action@v2
        with:
          channel: 'stable'
      
      - name: Get dependencies
        run: |
          cd client/flutter
          flutter pub get
      
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

**关键配置：**
- `matrix.runner`：选择正确架构的 runner
- `matrix.rust-target`：Rust 编译目标
- `matrix.flutter-arch`：Flutter 构建架构参数
- 并行构建两个架构
- 自动上传到 GitHub Release

---

### 4.2 本地测试 CI 流程

```bash
# 测试 arm64 构建
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin --package rdcs-ffi
cd client/flutter && flutter build macos --release --target-arch=arm64
cd ../.. && ./scripts/package_macos.sh arm64

# 测试 x64 构建（需要在 Intel Mac 上）
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin --package rdcs-ffi
cd client/flutter && flutter build macos --release --target-arch=x64
cd ../.. && ./scripts/package_macos.sh x64
```

---

## 5. 验证方案

### 5.1 开发环境验证

**Apple Silicon Mac：**
```bash
# 1. 开发构建
flutter run -d macos

# 2. 验证架构
file build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib
# 预期输出：arm64

# 3. 测试功能
# - 启动应用
# - 生成邀请码
# - 屏幕捕获
```

**Intel Mac：**
```bash
# 同样的验证步骤
# 预期输出：x86_64
```

---

### 5.2 发布构建验证

**测试 arm64 构建：**
```bash
# 1. 发布构建
flutter build macos --release --target-arch=arm64

# 2. 验证架构
file build/macos/Build/Products/Release/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib
# 预期输出：arm64

# 3. 打包 DMG
./scripts/package_macos.sh arm64

# 4. 安装测试
open rdcs_client-arm64-*.dmg
# 拖拽安装，运行验证功能
```

**测试 x64 构建：**
```bash
# 同样的步骤，预期输出：x86_64
```

---

### 5.3 CI 流程验证

**触发构建：**
```bash
# 1. 创建测试标签
git tag v0.1.0-test
git push origin v0.1.0-test

# 2. 观察 GitHub Actions
# - 两个并行任务（arm64 + x64）
# - 查看构建日志
# - 验证构建产物上传

# 3. 下载验证
# - 下载两个 DMG 文件
# - 在对应架构的 Mac 上安装测试
```

---

## 6. 依赖要求

### 6.1 开发环境

**Apple Silicon Mac：**
- macOS 11+ (Big Sur)
- Xcode 13+
- Rust 1.70+
- Flutter 3.x
- `aarch64-apple-darwin` target (自动安装)

**Intel Mac：**
- macOS 10.15+ (Catalina)
- Xcode 13+
- Rust 1.70+
- Flutter 3.x
- `x86_64-apple-darwin` target (自动安装)

---

### 6.2 CI 环境

**GitHub Actions Runners：**
- `macos-14`：Apple Silicon (M1)
- `macos-13`：Intel (x86_64)

**工具：**
- Rust toolchain action
- Flutter action
- DMG 打包工具（系统自带 hdiutil）

---

## 7. 迁移路径

### 7.1 阶段 1：脚本准备（1 天）

- [ ] 创建 `build_rust_lib.sh`
- [ ] 创建 `package_macos.sh`
- [ ] 更新 Xcode Build Phases

---

### 7.2 阶段 2：本地验证（1 天）

- [ ] Apple Silicon Mac 测试开发构建
- [ ] Apple Silicon Mac 测试发布构建
- [ ] Intel Mac 测试（如有设备）

---

### 7.3 阶段 3：CI 配置（1 天）

- [ ] 创建 GitHub Actions workflow
- [ ] 配置矩阵构建
- [ ] 测试 CI 构建流程

---

### 7.4 阶段 4：文档更新（0.5 天）

- [ ] 更新 README 构建说明
- [ ] 更新下载页面架构说明
- [ ] 添加开发者文档

---

## 8. 潜在问题与解决方案

### 8.1 构建时间

**问题：** 发布构建需要两次完整的 Cargo 编译

**解决方案：**
- CI 并行构建（无影响）
- 本地开发只编译当前架构
- 利用 cargo 缓存（sccache）

---

### 8.2 依赖库兼容性

**问题：** 某些依赖可能只支持特定架构

**解决方案：**
- 使用 `cargo check --target <arch>` 提前验证
- 选择跨平台依赖库
- 当前项目依赖均支持两个架构

---

### 8.3 Intel Mac 性能

**问题：** Intel Mac 运行性能相对较低

**解决方案：**
- 本地编译原生 x86_64（非 Rosetta）
- 编解码参数可针对架构调优（未来优化）
- 用户根据设备选择正确版本

---

## 9. 成功标准

### 9.1 功能标准

- [x] Apple Silicon Mac 开发构建正常
- [ ] Intel Mac 开发构建正常
- [ ] Apple Silicon Mac 发布构建正常
- [ ] Intel Mac 发布构建正常
- [ ] CI 并行构建两个架构
- [ ] DMG 安装包正确生成

---

### 9.2 性能标准

- 开发构建时间：≤ 当前速度的 110%
- 发布构建时间：每个架构 ≤ 当前速度的 120%
- CI 总构建时间：≤ 20 分钟（并行）

---

### 9.3 用户体验标准

- 用户能清晰识别下载哪个版本
- 安装后自动运行在正确架构下
- 所有功能正常（邀请码、屏幕捕获、输入控制）

---

## 10. 未来扩展

### 10.1 Windows 支持

参考当前设计，扩展到 Windows x64 架构。

---

### 10.2 Linux 支持

支持 x86_64 和 arm64 架构的 Linux 分发。

---

### 10.3 Universal Binary 选项

如未来有需求，可添加 `--universal` 标志：
```bash
flutter build macos --release --universal
```

合并两个架构为单一 app（使用 lipo）。

---

## 11. 参考资料

- [Apple: Building a Universal macOS Binary](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary)
- [Rust: Cross-compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Flutter: Build for macOS](https://docs.flutter.dev/deployment/macos)
- [GitHub Actions: Matrix builds](https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs)

---

## 附录 A：架构对照表

| 架构名称 | Rust Target | Flutter Arch | macOS 设备 |
|---------|-------------|--------------|-----------|
| Apple Silicon | aarch64-apple-darwin | arm64 | M1/M2/M3/M4 |
| Intel Mac | x86_64-apple-darwin | x64 | Intel Core i5/i7/i9 |

---

## 附录 B：文件清单

**新增文件：**
```
client/flutter/macos/build_rust_lib.sh       # Rust 库构建脚本
scripts/package_macos.sh                      # DMG 打包脚本
.github/workflows/build-macos.yml             # CI 配置
docs/BUILD_MULTI_ARCH.md                      # 构建说明文档
```

**修改文件：**
```
client/flutter/macos/Runner.xcodeproj/project.pbxproj  # Xcode Build Phase
client/flutter/macos/copy_ffi_lib.sh                   # 删除（被 build_rust_lib.sh 替换）
README.md                                               # 添加架构说明
```

---

**文档状态：** ✅ 完成，等待用户审核
