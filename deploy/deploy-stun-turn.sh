#!/bin/bash
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0

# Deploy STUN/TURN servers for NAT traversal testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "RDCS STUN/TURN Server Deployment"
echo "=========================================="
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose is not installed. Please install docker-compose first."
    exit 1
fi

echo "✓ Docker and docker-compose found"
echo ""

# Stop existing containers
echo "Stopping existing STUN/TURN containers..."
docker-compose -f "$SCRIPT_DIR/docker-compose.stun.yml" down 2>/dev/null || true
echo ""

# Start services
echo "Starting STUN/TURN servers..."
docker-compose -f "$SCRIPT_DIR/docker-compose.stun.yml" up -d

echo ""
echo "Waiting for services to be ready..."
sleep 5

# Check service status
echo ""
echo "=========================================="
echo "Service Status"
echo "=========================================="

if docker ps | grep -q rdcs-stun-server; then
    echo "✓ STUN server is running"
else
    echo "❌ STUN server failed to start"
    docker logs rdcs-stun-server
    exit 1
fi

if docker ps | grep -q rdcs-turn-server; then
    echo "✓ TURN server is running"
else
    echo "❌ TURN server failed to start"
    docker logs rdcs-turn-server
    exit 1
fi

echo ""
echo "=========================================="
echo "Connection Information"
echo "=========================================="
echo "STUN Server: stun://localhost:3478"
echo "TURN Server: turn://localhost:3478"
echo "TURN Username: rdcs-user"
echo "TURN Password: rdcs-test-password"
echo ""

# Test connectivity
echo "=========================================="
echo "Testing Connectivity"
echo "=========================================="

# Check if stunclient is available
if command -v stunclient &> /dev/null; then
    echo "Testing STUN server..."
    if stunclient localhost 3478; then
        echo "✓ STUN server is responding"
    else
        echo "⚠ STUN server test failed (but server may still work)"
    fi
else
    echo "⚠ stunclient not found, skipping STUN test"
    echo "  Install with: apt-get install stun-client (or brew install stuntman)"
fi

echo ""
echo "=========================================="
echo "Next Steps"
echo "=========================================="
echo "1. Update client configuration:"
echo "   export STUN_SERVER=stun://localhost:3478"
echo "   export TURN_SERVER=turn://localhost:3478"
echo "   export TURN_USERNAME=rdcs-user"
echo "   export TURN_PASSWORD=rdcs-test-password"
echo ""
echo "2. Run NAT traversal tests:"
echo "   cd $PROJECT_ROOT"
echo "   cargo test --test nat_traversal_test"
echo ""
echo "3. View logs:"
echo "   docker logs -f rdcs-stun-server"
echo "   docker logs -f rdcs-turn-server"
echo ""
echo "4. Stop servers:"
echo "   docker-compose -f $SCRIPT_DIR/docker-compose.stun.yml down"
echo ""
echo "✓ Deployment complete!"
