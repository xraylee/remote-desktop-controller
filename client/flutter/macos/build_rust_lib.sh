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

# Check Rust target
echo "🔍 Checking Rust target..."
if rustup target list | grep -q "^${RUST_TARGET} (installed)"; then
    echo "✅ Target installed: $RUST_TARGET"
else
    echo "📥 Installing target: $RUST_TARGET"
    rustup target add "$RUST_TARGET"
fi

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
