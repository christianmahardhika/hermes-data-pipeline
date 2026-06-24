#!/bin/bash
# PostgreSQL Backup Script for hermes-data-pipeline
# Outputs compressed pg_dump to stdout or file, suitable for ORAS upload to GHCR

set -euo pipefail

# Configuration (override via environment)
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_USER="${POSTGRES_USER:-hermes}"
POSTGRES_DB="${POSTGRES_DB:-hermes_pipelines}"
BACKUP_DIR="${BACKUP_DIR:-/tmp/backups}"
BACKUP_TYPE="${BACKUP_TYPE:-daily}"  # daily or weekly

# Generate timestamp and filename
TIMESTAMP=$(date -u +"%Y%m%d_%H%M%S")
DATE_TAG=$(date -u +"%Y%m%d")
BACKUP_FILE="hermes_pipelines_${BACKUP_TYPE}_${TIMESTAMP}.sql.gz"

log() {
    echo "[$(date -u +"%Y-%m-%d %H:%M:%S UTC")] $*" >&2
}

die() {
    log "ERROR: $*"
    exit 1
}

# Validate PGPASSWORD is set
[[ -z "${PGPASSWORD:-}" ]] && die "PGPASSWORD environment variable required"

# Create backup directory
mkdir -p "$BACKUP_DIR"

log "Starting ${BACKUP_TYPE} backup of ${POSTGRES_DB}@${POSTGRES_HOST}:${POSTGRES_PORT}"

# Run pg_dump with compression
pg_dump \
    --host="$POSTGRES_HOST" \
    --port="$POSTGRES_PORT" \
    --username="$POSTGRES_USER" \
    --dbname="$POSTGRES_DB" \
    --format=plain \
    --no-owner \
    --no-acl \
    --verbose \
    2>&1 | gzip -9 > "${BACKUP_DIR}/${BACKUP_FILE}"

BACKUP_SIZE=$(stat -c%s "${BACKUP_DIR}/${BACKUP_FILE}" 2>/dev/null || stat -f%z "${BACKUP_DIR}/${BACKUP_FILE}")
log "Backup complete: ${BACKUP_FILE} (${BACKUP_SIZE} bytes)"

# Output backup path for caller
echo "${BACKUP_DIR}/${BACKUP_FILE}"
