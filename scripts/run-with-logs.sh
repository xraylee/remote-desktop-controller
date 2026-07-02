#!/bin/bash
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0
#
# Launch the built RDCS macOS client and capture ALL runtime logs.
#
# Captures both log streams the app produces:
#   - Flutter Dart  `print(...)`      → stdout
#   - Rust FFI       `println!` / `eprintln!` → stdout / stderr
#
# Both are merged (2>&1) into a single timestamped file under logs/, and
# also echoed to the console via `tee` so you can watch live.
#
# Usage:
#   scripts/run-with-logs.sh              # debug build (default)
#   scripts/run-with-logs.sh --release    # release build
#
# The log path is printed at start and end. Stop with Ctrl-C.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BUILD_DIR="Debug"
if [[ "${1:-}" == "--release" ]]; then
    BUILD_DIR="Release"
fi

APP="$PROJECT_ROOT/client/flutter/build/macos/Build/Products/$BUILD_DIR/rdcs_client.app"
BIN="$APP/Contents/MacOS/rdcs_client"

if [[ ! -x "$BIN" ]]; then
    echo "❌ App binary not found: $BIN"
    echo "   Build first:  cd client/flutter && flutter build macos --$(echo "$BUILD_DIR" | tr '[:upper:]' '[:lower:]')"
    exit 1
fi

LOG_DIR="$PROJECT_ROOT/logs"
mkdir -p "$LOG_DIR"
TS="$(date +%Y%m%d-%H%M%S)"
LOG_FILE="$LOG_DIR/runtime-$TS.log"

# Stop any already-running instance so logs come from THIS launch only.
if pgrep -f "rdcs_client.app/Contents/MacOS/rdcs_client" >/dev/null 2>&1; then
    echo "⚠️  Stopping existing rdcs_client instance(s)..."
    pkill -f "rdcs_client.app/Contents/MacOS/rdcs_client" || true
    sleep 1
fi

echo "═══════════════════════════════════════════════════════════════"
echo "  RDCS client ($BUILD_DIR) — logging to:"
echo "  $LOG_FILE"
echo "═══════════════════════════════════════════════════════════════"

# Header with environment context for later diagnosis.
{
    echo "==== RDCS runtime log ===="
    echo "timestamp : $TS"
    echo "build     : $BUILD_DIR"
    echo "binary    : $BIN"
    echo "arch      : $(uname -m)"
    echo "host      : $(hostname) $(ipconfig getifaddr en0 2>/dev/null || true)"
    echo "dylib     : $(otool -D "$APP/Contents/Frameworks/librdcs_core.dylib" 2>/dev/null | tail -1)"
    echo "=========================="
} | tee "$LOG_FILE"

# Run in foreground, merge stderr into stdout, tee to file + console.
# RUST_BACKTRACE surfaces panics from the FFI layer in the log.
RUST_BACKTRACE=1 "$BIN" 2>&1 | tee -a "$LOG_FILE"

echo "═══════════════════════════════════════════════════════════════"
echo "  App exited. Full log saved to:"
echo "  $LOG_FILE"
echo "═══════════════════════════════════════════════════════════════"
