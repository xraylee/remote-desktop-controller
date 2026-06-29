#!/bin/bash
# RDCS Display Module - Build and Test Script

set -e

echo "========================================="
echo "🖥️  RDCS Display Module Builder"
echo "========================================="
echo ""

# Check SDL2
echo "1️⃣  Checking SDL2 dependencies..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    if ! brew list sdl2 &>/dev/null; then
        echo "   ⚠️  SDL2 not found, installing via Homebrew..."
        brew install sdl2
    else
        echo "   ✓ SDL2 already installed"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if ! dpkg -l | grep -q libsdl2-dev; then
        echo "   ⚠️  SDL2 not found, please install:"
        echo "      sudo apt-get install libsdl2-dev"
        exit 1
    else
        echo "   ✓ SDL2 already installed"
    fi
fi
echo ""

# Build rdcs-display
echo "2️⃣  Building rdcs-display..."
cargo build -p rdcs-display --release
echo "   ✓ Build successful"
echo ""

# Run display test
echo "3️⃣  Running display test (will open window)..."
echo "   Press ESC or close window to continue"
echo ""
cargo run --example display_test -p rdcs-display --release

echo ""
echo "========================================="
echo "✅ rdcs-display module ready!"
echo "========================================="
echo ""
echo "Next steps:"
echo "  • Run end-to-end test:"
echo "    cargo run --example display_roundtrip --features software-encoder --release"
echo ""
