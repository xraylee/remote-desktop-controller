#!/bin/bash
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0

# Fix Flutter FFI library deployment issue
# This script rebuilds the FFI library with the correct name and deploys it

set -e

echo "🔧 Flutter FFI Library Fix Script"
echo "=================================="
echo ""

# Check if we're in the project root
if [ ! -f "Cargo.toml" ] || [ ! -d "crates/rdcs-ffi" ]; then
    echo "❌ Error: Must run from project root"
    exit 1
fi

echo "📦 Step 1: Building Rust FFI library (rdcs-core)..."
cargo build -p rdcs-ffi
echo "✅ Build complete"
echo ""

# Verify the library was built with correct name
if [ -f "target/debug/librdcs_core.dylib" ]; then
    echo "✅ Library name verified: librdcs_core.dylib"
else
    echo "❌ Error: librdcs_core.dylib not found"
    echo "   Check if Cargo.toml has: [lib] name = \"rdcs_core\""
    exit 1
fi
echo ""

# Flutter app bundle path
FLUTTER_APP="client/flutter/build/macos/Build/Products/Debug/rdcs_client.app"
FRAMEWORKS_DIR="$FLUTTER_APP/Contents/Frameworks"

echo "📂 Step 2: Checking Flutter app bundle..."
if [ ! -d "$FLUTTER_APP" ]; then
    echo "⚠️  Flutter app not built yet"
    echo "   Run: cd client/flutter && flutter build macos --debug"
    echo ""
    echo "   Skipping deployment to app bundle"
    echo "   (Library will be copied automatically on next Flutter build)"
else
    echo "✅ Flutter app bundle found"
    echo ""

    echo "📋 Step 3: Deploying library to app bundle..."
    mkdir -p "$FRAMEWORKS_DIR"
    cp -v "target/debug/librdcs_core.dylib" "$FRAMEWORKS_DIR/"
    echo "✅ Library deployed to: $FRAMEWORKS_DIR/librdcs_core.dylib"
fi
echo ""

echo "🎉 Fix complete!"
echo ""
echo "Next steps:"
echo "  1. cd client/flutter"
echo "  2. flutter clean"
echo "  3. flutter run -d macos"
echo ""
echo "Expected output:"
echo "  ✅ Loading from: .../Contents/Frameworks/librdcs_core.dylib"
