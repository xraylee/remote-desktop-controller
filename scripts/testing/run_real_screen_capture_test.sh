#!/bin/bash
# Real Screen Capture + Hardware Encoder Test Runner
# Run this script in your macOS terminal

set -e

cd "$(dirname "$0")"

echo "========================================"
echo "Real Screen Capture Test"
echo "========================================"
echo ""

# Build first
echo "Building test..."
cargo build -p rdcs-connection --example real_screen_capture_test --features hardware-accel

echo ""
echo "Running test..."
echo ""

# Run with info logging
RUST_LOG=info cargo run -p rdcs-connection --example real_screen_capture_test --features hardware-accel

echo ""
echo "========================================"
echo "Test completed!"
echo "========================================"
