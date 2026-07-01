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
