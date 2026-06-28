#!/usr/bin/env bash
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0
#
# RDCS Development Environment Reset
# Tears down all containers/volumes and rebuilds from scratch.
# Usage: make reset

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

COMPOSE_FILES="-f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml"

echo ""
printf "${YELLOW}RDCS Development Environment Reset${NC}\n"
echo "======================================"
echo ""

# Step 1: Tear down everything
printf "${YELLOW}[1/3]${NC} Stopping containers and removing volumes...\n"
docker compose $COMPOSE_FILES down -v --remove-orphans 2>/dev/null || true
printf "  ${GREEN}Done.${NC}\n"

# Step 2: Remove dangling images
printf "${YELLOW}[2/3]${NC} Pruning dangling build images...\n"
docker image prune -f 2>/dev/null || true
printf "  ${GREEN}Done.${NC}\n"

# Step 3: Rebuild and start
printf "${YELLOW}[3/3]${NC} Building and starting development environment...\n"
echo ""
docker compose $COMPOSE_FILES up -d --build

echo ""
printf "${GREEN}Development environment reset complete.${NC}\n"
echo ""
echo "Next steps:"
echo "  make health    # Check service status"
echo "  make dev-logs  # View service logs"
echo ""
