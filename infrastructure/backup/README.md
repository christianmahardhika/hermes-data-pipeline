# PostgreSQL Backup & Restore Guide

## Overview

This infrastructure uses ORAS (OCI Registry As Storage) to store PostgreSQL backups as OCI artifacts in GitHub Container Registry (GHCR). This approach avoids Docker packaging overhead and leverages GHCR's built-in versioning and retention.

## Backup Schedule

| Type | Schedule | Retention | Tag Pattern |
|------|----------|-----------|-------------|
| Daily | 02:00 UTC (09:00 WIB) | 7 backups | `daily-YYYYMMDD` |
| Weekly | Sunday 03:00 UTC (10:00 WIB) | 4 backups | `weekly-YYYYMMDD` |

## Components

```
infrastructure/backup/
├── backup.sh      # pg_dump wrapper script
├── retention.sh   # GHCR cleanup script
└── README.md      # This file

.github/workflows/
└── postgres-backup.yml  # GitHub Actions workflow
```

## Manual Backup

### Prerequisites
- Docker and docker-compose
- PostgreSQL client (`pg_dump`)
- ORAS CLI: https://oras.land/docs/installation

### Run backup locally

```bash
# Set credentials
export PGPASSWORD="your_password"
export POSTGRES_HOST="localhost"
export POSTGRES_PORT="5432"

# Run backup script
chmod +x infrastructure/backup/backup.sh
BACKUP_PATH=$(./infrastructure/backup/backup.sh)
echo "Backup created: $BACKUP_PATH"
```

### Push to GHCR manually

```bash
# Login to GHCR
echo "$GITHUB_TOKEN" | oras login ghcr.io -u YOUR_USERNAME --password-stdin

# Push backup
DATE_TAG=$(date -u +"%Y%m%d")
oras push ghcr.io/christianmahardhika/hermes-pipelines-backup:manual-${DATE_TAG} \
    --artifact-type "application/vnd.hermes.backup.postgres+gzip" \
    "${BACKUP_PATH}:application/gzip"
```

## Restore Procedure

### Step 1: Install ORAS

```bash
# Linux
curl -LO https://github.com/oras-project/oras/releases/download/v1.2.0/oras_1.2.0_linux_amd64.tar.gz
tar -xzf oras_1.2.0_linux_amd64.tar.gz
sudo mv oras /usr/local/bin/

# macOS
brew install oras
```

### Step 2: List available backups

```bash
# Login to GHCR (read access)
echo "$GITHUB_TOKEN" | oras login ghcr.io -u YOUR_USERNAME --password-stdin

# List all tags
oras repo tags ghcr.io/christianmahardhika/hermes-pipelines-backup

# Example output:
# daily-20260614
# daily-20260613
# weekly-20260609
```

### Step 3: Pull the backup

```bash
# Create restore directory
mkdir -p /tmp/restore && cd /tmp/restore

# Pull specific backup (e.g., daily-20260614)
oras pull ghcr.io/christianmahardhika/hermes-pipelines-backup:daily-20260614

# Verify files
ls -la
# hermes_pipelines_daily_20260614_020000.sql.gz
# hermes_pipelines_daily_20260614_020000.sql.gz.sha256

# Verify checksum
sha256sum -c *.sha256
```

### Step 4: Restore to PostgreSQL

```bash
# Option A: Restore to running container
cd infrastructure
docker compose --profile full up -d postgres

# Wait for postgres
sleep 5

# Restore (drops and recreates)
gunzip -c /tmp/restore/*.sql.gz | docker compose --profile full exec -T postgres \
    psql -U hermes -d hermes_pipelines

# Option B: Restore to fresh database
gunzip -c /tmp/restore/*.sql.gz | docker compose --profile full exec -T postgres \
    psql -U hermes -d postgres -c "DROP DATABASE IF EXISTS hermes_pipelines; CREATE DATABASE hermes_pipelines;" && \
gunzip -c /tmp/restore/*.sql.gz | docker compose --profile full exec -T postgres \
    psql -U hermes -d hermes_pipelines

# Option C: Restore to external PostgreSQL
export PGPASSWORD="your_password"
gunzip -c /tmp/restore/*.sql.gz | psql -h your-host -U hermes -d hermes_pipelines
```

### Step 5: Verify restore

```bash
# Check table counts
docker compose --profile full exec postgres psql -U hermes -d hermes_pipelines -c "\dt"

# Run sanity query
docker compose --profile full exec postgres psql -U hermes -d hermes_pipelines -c "SELECT COUNT(*) FROM your_table;"
```

## Disaster Recovery Scenarios

### Scenario 1: Database corruption

```bash
# 1. Stop application services
docker compose down

# 2. Remove corrupted volume
docker volume rm hermes-data-pipeline_postgres_data

# 3. Start fresh postgres
docker compose --profile full up -d postgres

# 4. Restore from latest daily backup
oras pull ghcr.io/christianmahardhika/hermes-pipelines-backup:daily-$(date -u +%Y%m%d)
gunzip -c *.sql.gz | docker compose --profile full exec -T postgres psql -U hermes -d hermes_pipelines

# 5. Restart all services
docker compose --profile full up -d
```

### Scenario 2: Point-in-time recovery (specific date)

```bash
# List available backups to find the right one
oras repo tags ghcr.io/christianmahardhika/hermes-pipelines-backup | sort -r

# Pull and restore the specific backup
oras pull ghcr.io/christianmahardhika/hermes-pipelines-backup:daily-20260610
# ... follow restore steps above
```

### Scenario 3: New environment setup

```bash
# Clone repo
git clone https://github.com/christianmahardhika/hermes-data-pipeline
cd hermes-data-pipeline/infrastructure

# Setup environment
cp .env.example .env
# Edit .env with your credentials

# Start services
docker compose --profile full up -d

# Pull latest weekly backup (more stable baseline)
LATEST_WEEKLY=$(oras repo tags ghcr.io/christianmahardhika/hermes-pipelines-backup | grep weekly | sort -r | head -1)
oras pull ghcr.io/christianmahardhika/hermes-pipelines-backup:${LATEST_WEEKLY}

# Restore
gunzip -c *.sql.gz | docker compose --profile full exec -T postgres psql -U hermes -d hermes_pipelines
```

## Troubleshooting

### ORAS login fails
```bash
# Ensure token has read:packages scope
# For restore: read:packages
# For push: write:packages, delete:packages (for retention)
```

### Backup file is empty
```bash
# Check postgres is running
docker compose --profile full exec postgres pg_isready -U hermes

# Check PGPASSWORD is set
echo $PGPASSWORD
```

### Restore fails with "relation already exists"
```bash
# Drop and recreate database first
docker compose --profile full exec postgres psql -U hermes -d postgres \
    -c "DROP DATABASE hermes_pipelines; CREATE DATABASE hermes_pipelines;"
```

## GitHub Secrets Required

| Secret | Description |
|--------|-------------|
| `POSTGRES_PASSWORD` | PostgreSQL password for `hermes` user |
| `GITHUB_TOKEN` | Auto-provided, needs `packages: write` permission |

## Monitoring

- Check workflow runs: https://github.com/christianmahardhika/hermes-data-pipeline/actions/workflows/postgres-backup.yml
- Check GHCR packages: https://github.com/christianmahardhika/hermes-data-pipeline/pkgs/container/hermes-pipelines-backup
