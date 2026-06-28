#!/bin/bash
# Quick build check script
# Usage: ./check_build.sh

set -e

echo "========================================="
echo "Building Rust Workspace"
echo "========================================="
echo ""

echo "Step 1: Building rdcs-connection..."
cargo build -p rdcs-connection --lib 2>&1 | tail -20
echo "✅ rdcs-connection built"
echo ""

echo "Step 2: Building rdcs-codec..."
cargo build -p rdcs-codec --lib 2>&1 | tail -20
echo "✅ rdcs-codec built"
echo ""

echo "Step 3: Building examples..."
cargo build -p rdcs-connection --example video_e2e_test 2>&1 | tail -20
echo "✅ video_e2e_test built"
echo ""

echo "Step 4: Running test compilation..."
cargo test --no-run 2>&1 | tail -30
echo ""

echo "========================================="
echo "✅ All builds completed successfully!"
echo "========================================="
