#!/bin/bash
# Quick start script for ICE connection testing
# Usage: ./scripts/ice-test-client.sh

set -e

echo "=========================================="
echo "🚀 RDCS ICE Connection Test - CLIENT"
echo "=========================================="
echo ""
echo "This Mac will act as the ICE Client (Answerer)"
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

echo "📦 Building ice_client example..."
cargo build -p rdcs-connection --example ice_client --release

echo ""
echo "=========================================="
echo "✅ Build complete!"
echo "=========================================="
echo ""
echo "Starting ICE Client..."
echo ""
echo "Instructions:"
echo "  1. Paste the OFFER JSON from the server"
echo "  2. Press Ctrl+D to complete input"
echo "  3. Wait for ANSWER JSON output"
echo "  4. Copy the entire ANSWER JSON"
echo "  5. Send it back to the server"
echo ""
echo "=========================================="
echo ""

# Run the client
cargo run -p rdcs-connection --example ice_client --release
