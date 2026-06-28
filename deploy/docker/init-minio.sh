#!/bin/sh
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0

# RDCS MinIO Initialization Script
# Creates required storage buckets and configures lifecycle policies.
# Runs as a one-shot init container after MinIO is healthy.

set -e

MINIO_HOST="${MINIO_HOST:-http://minio:9000}"
MINIO_USER="${MINIO_ROOT_USER:-minioadmin}"
MINIO_PASSWORD="${MINIO_ROOT_PASSWORD:-minioadmin_dev}"
ALIAS_NAME="rdcs"

echo "Waiting for MinIO to be ready..."
RETRIES=30
until curl -sf http://minio:9000/minio/health/live >/dev/null 2>&1; do
  RETRIES=$((RETRIES - 1))
  if [ "$RETRIES" -le 0 ]; then
    echo "ERROR: MinIO did not become ready in time"
    exit 1
  fi
  echo "  MinIO not ready yet, retrying... ($RETRIES attempts left)"
  sleep 2
done
echo "MinIO is ready."

# Configure mc alias
mc alias set "$ALIAS_NAME" "$MINIO_HOST" "$MINIO_USER" "$MINIO_PASSWORD"

# Create buckets (ignore errors if they already exist)
echo "Creating storage buckets..."

mc mb --ignore-existing "$ALIAS_NAME/recordings"
echo "  - recordings (session recordings)"

mc mb --ignore-existing "$ALIAS_NAME/file-transfers"
echo "  - file-transfers (file transfer staging)"

mc mb --ignore-existing "$ALIAS_NAME/backups"
echo "  - backups (database backups)"

# Set recordings bucket to allow read-only download
mc anonymous set download "$ALIAS_NAME/recordings"
echo "  - recordings: set to download-only anonymous access"

# Set lifecycle rule: auto-delete file-transfers objects after 7 days
mc ilm rule add --expiry-days 7 "$ALIAS_NAME/file-transfers"
echo "  - file-transfers: 7-day auto-expiry lifecycle rule"

echo ""
echo "MinIO initialization complete."
echo "Buckets: recordings, file-transfers, backups"
