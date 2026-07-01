#!/bin/bash
# Package macOS app into architecture-specific DMG

set -e

ARCH=$1

if [ -z "$ARCH" ]; then
    echo "Usage: $0 <arm64|x64>"
    echo "  arm64 = Apple Silicon (M1/M2/M3/M4)"
    echo "  x64   = Intel Mac"
    exit 1
fi

if [ "$ARCH" != "arm64" ] && [ "$ARCH" != "x64" ]; then
    echo "❌ Invalid architecture: $ARCH (must be arm64 or x64)"
    exit 1
fi

# Robust version parse: skip comment lines, take first match
VERSION=$(grep -v '^\s*#' client/flutter/pubspec.yaml | grep "^version:" | head -1 | awk '{print $2}')
if [ -z "$VERSION" ]; then
    echo "❌ Could not read version from client/flutter/pubspec.yaml"
    exit 1
fi

APP_PATH="client/flutter/build/macos/Build/Products/Release/rdcs_client.app"
DMG_NAME="rdcs_client-${ARCH}-v${VERSION}.dmg"
DYLIB_PATH="$APP_PATH/Contents/Frameworks/librdcs_core.dylib"

# Verify app bundle exists
if [ ! -d "$APP_PATH" ]; then
    echo "❌ App not found: $APP_PATH"
    echo "   Run: cd client/flutter && flutter build macos --release"
    exit 1
fi

# Verify dylib exists
if [ ! -f "$DYLIB_PATH" ]; then
    echo "❌ Library not found: $DYLIB_PATH"
    echo "   The Rust FFI library was not copied into the app bundle."
    exit 1
fi

# Determine expected architecture string from file(1) output
case "$ARCH" in
    arm64) EXPECTED_ARCH="arm64" ;;
    x64)   EXPECTED_ARCH="x86_64" ;;
esac

ACTUAL_ARCH=$(file "$DYLIB_PATH" | grep -oE 'arm64|x86_64' | head -1)
echo "📋 Requested arch: $ARCH  (expected binary: $EXPECTED_ARCH)"
echo "📋 Actual binary arch: ${ACTUAL_ARCH:-unknown}"

if [ "$ACTUAL_ARCH" != "$EXPECTED_ARCH" ]; then
    echo "❌ Architecture mismatch!"
    echo "   DMG label says '$ARCH' but the library is '$ACTUAL_ARCH'."
    echo "   Rebuild for the correct target before packaging."
    exit 1
fi

# Verify install_name uses @rpath (required for Flutter FFI loading)
INSTALL_NAME=$(otool -D "$DYLIB_PATH" | tail -1)
if [ "$INSTALL_NAME" != "@rpath/librdcs_core.dylib" ]; then
    echo "❌ Library install_name is wrong: $INSTALL_NAME"
    echo "   Expected: @rpath/librdcs_core.dylib"
    echo "   Rebuild using build_rust_lib.sh which fixes this automatically."
    exit 1
fi

echo "✅ Architecture verified: $ACTUAL_ARCH"
echo "✅ install_name verified: $INSTALL_NAME"

# Create DMG
echo "📦 Creating $DMG_NAME  (version $VERSION)..."
hdiutil create -volname "RDCS Client" \
    -srcfolder "$APP_PATH" \
    -ov -format UDZO \
    "$DMG_NAME"

echo "✅ DMG created: $DMG_NAME"
