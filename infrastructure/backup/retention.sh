#!/bin/bash
# Retention Policy Script for GHCR backups
# Keeps: 7 daily backups, 4 weekly backups
# Requires: gh CLI authenticated with delete:packages scope

set -euo pipefail

REPO="${GITHUB_REPOSITORY:-christianmahardhika/hermes-data-pipeline}"
PACKAGE_NAME="hermes-pipelines-backup"
DAILY_KEEP=7
WEEKLY_KEEP=4

log() {
    echo "[$(date -u +"%Y-%m-%d %H:%M:%S UTC")] $*"
}

# Get all package versions with tags
get_versions() {
    local tag_pattern="$1"
    gh api \
        -H "Accept: application/vnd.github+json" \
        "/user/packages/container/${PACKAGE_NAME}/versions" \
        --paginate \
        --jq ".[] | select(.metadata.container.tags[] | test(\"${tag_pattern}\")) | {id: .id, tags: .metadata.container.tags, created: .created_at}" \
        2>/dev/null || echo "[]"
}

# Delete a package version by ID
delete_version() {
    local version_id="$1"
    log "Deleting version ID: ${version_id}"
    gh api \
        --method DELETE \
        -H "Accept: application/vnd.github+json" \
        "/user/packages/container/${PACKAGE_NAME}/versions/${version_id}" \
        2>/dev/null || log "Warning: Failed to delete version ${version_id}"
}

# Apply retention to a set of versions
apply_retention() {
    local tag_pattern="$1"
    local keep_count="$2"
    local type_name="$3"
    
    log "Applying ${type_name} retention: keep ${keep_count}"
    
    # Get versions sorted by creation date (newest first)
    local versions
    versions=$(get_versions "$tag_pattern" | jq -s 'sort_by(.created) | reverse')
    
    local total
    total=$(echo "$versions" | jq 'length')
    
    if [[ "$total" -le "$keep_count" ]]; then
        log "${type_name}: ${total} versions found, keeping all (threshold: ${keep_count})"
        return 0
    fi
    
    local to_delete=$((total - keep_count))
    log "${type_name}: ${total} versions found, deleting ${to_delete} oldest"
    
    # Delete oldest versions (skip first $keep_count)
    echo "$versions" | jq -r ".[$keep_count:][].id" | while read -r version_id; do
        delete_version "$version_id"
    done
}

log "Starting retention cleanup for ${PACKAGE_NAME}"

# Apply retention policies
apply_retention "^daily-" "$DAILY_KEEP" "Daily"
apply_retention "^weekly-" "$WEEKLY_KEEP" "Weekly"

log "Retention cleanup complete"
