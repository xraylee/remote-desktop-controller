#!/bin/bash
# Quick start script for ICE connection testing
# Usage: ./scripts/ice-test-server.sh

set -e

echo "=========================================="
echo "🚀 RDCS ICE Connection Test - SERVER"
echo "=========================================="
echo ""
echo "This Mac will act as the ICE Server (Offerer)"
echo ""
echo "Architecture: $(uname -m)"
echo "OS: $(sw_vers -productVersion)"
echo ""
echo "=========================================="
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found"
    echo ""
    echo "Please install Rust:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "📦 Building ice_server example..."
cargo build -p rdcs-connection --example ice_server --release

echo ""
echo "=========================================="
echo "✅ Build complete!"
echo "=========================================="
echo ""
echo "Starting ICE Server..."
echo ""
echo "Instructions:"
echo "  1. Wait for OFFER JSON output"
echo "  2. Copy the entire JSON block"
echo "  3. Send it to the client machine"
echo "  4. Wait for ANSWER from client"
echo "  5. Paste the ANSWER JSON here"
echo "  6. Press Ctrl+D to complete input"
echo ""
echo "=========================================="
echo ""

# Run the server
cargo run -p rdcs-connection --example ice_server --release
