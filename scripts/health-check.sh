#!/usr/bin/env bash
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0
#
# Health check script for RDCS services.
# Usage: make health

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color
BOLD='\033[1m'

COMPOSE_FILE="-f deploy/docker/docker-compose.yml"

services=(
  "signaling:Signaling Server (Rust)"
  "relay:Relay Server (Rust)"
  "api:Management API (Go)"
  "web:Web Admin Console (React)"
  "postgres:PostgreSQL"
  "redis:Redis"
  "minio:MinIO Object Storage"
)

echo ""
printf "${BOLD}RDCS Service Health Check${NC}\n"
echo "========================="
echo ""

all_healthy=true
healthy_count=0
total=${#services[@]}

for entry in "${services[@]}"; do
  IFS=':' read -r service label <<< "$entry"
  result=$(docker compose $COMPOSE_FILE ps --format json "$service" 2>/dev/null | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    if isinstance(data, list):
        data = data[0] if data else {}
    state = data.get('State', 'not_found')
    health = data.get('Health', '')
    if health:
        print(f'{state}:{health}')
    else:
        print(f'{state}:none')
except:
    print('not_found:none')
" 2>/dev/null || echo "not_found:none")

  state=$(echo "$result" | cut -d: -f1)
  health=$(echo "$result" | cut -d: -f2)

  if [[ "$state" == "running" && ("$health" == "healthy" || "$health" == "none") ]]; then
    printf "  ${GREEN}✓${NC} %-32s %s\n" "$label" "($service)"
    healthy_count=$((healthy_count + 1))
  elif [[ "$state" == "running" && "$health" == "starting" ]]; then
    printf "  ${YELLOW}◌${NC} %-32s %s [%s]\n" "$label" "($service)" "starting"
    all_healthy=false
  else
    printf "  ${RED}✗${NC} %-32s %s [%s]\n" "$label" "($service)" "$state"
    all_healthy=false
  fi
done

echo ""
printf "  ${healthy_count}/${total} services running\n"
echo ""
if $all_healthy; then
  printf "${GREEN}All services are healthy.${NC}\n"
else
  printf "${RED}Some services are not running.${NC}\n"
  exit 1
fi
