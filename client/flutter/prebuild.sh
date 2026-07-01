#!/bin/bash
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0

# Flutter 构建前钩子：自动构建并复制 Rust FFI 库

set -e

echo "━━━ Flutter prebuild: Building Rust FFI library ━━━"

# 项目根目录（2 层向上：flutter/ -> client/ -> 项目根）
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RUST_LIB="$PROJECT_ROOT/target/debug/librdcs_core.dylib"

# 构建 Rust 库
cd "$PROJECT_ROOT"
if ! cargo build -p rdcs-ffi --quiet; then
    echo "❌ Rust build failed"
    exit 1
fi

echo "✅ Rust library built: $RUST_LIB"

# 如果在 Xcode 构建环境中，直接复制到 Frameworks
if [ -n "$BUILT_PRODUCTS_DIR" ] && [ -n "$PRODUCT_NAME" ]; then
    FRAMEWORKS_DIR="$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/Contents/Frameworks"
    mkdir -p "$FRAMEWORKS_DIR"
    cp "$RUST_LIB" "$FRAMEWORKS_DIR/"
    echo "✅ Library copied to app bundle: $FRAMEWORKS_DIR"
fi
