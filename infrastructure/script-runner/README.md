# Script Runner

Lightweight script runner with SQLite log aggregation and Dead Letter Queue (DLQ).

## Features

- **Zero daemon** — just a Python wrapper, no background process
- **SQLite storage** — single file, portable, queryable
- **DLQ for transient errors** — token expired, timeouts, rate limits auto-queued for retry
- **CLI for troubleshooting** — view logs, retry failed jobs, stats

## Installation

```bash
# Symlink to PATH
ln -sf ~/hermes-data-pipeline/infrastructure/script-runner/script_runner.py ~/.local/bin/script-runner
chmod +x ~/.local/bin/script-runner

# Or use directly
alias script-runner="python ~/hermes-data-pipeline/infrastructure/script-runner/script_runner.py"
```

## Usage

### Run a script

```bash
# Run with logging
script-runner run daily_digest.py

# With arguments
script-runner run social_intel_cron.py --sources twitter,reddit

# With timeout
script-runner run slow_scraper.py --timeout 300
```

### View logs

```bash
# Recent runs
script-runner logs

# Filter by script
script-runner logs daily_digest

# Failed only
script-runner logs --failed

# Specific run's full output
script-runner logs --run-id 42
```

### Dead Letter Queue

```bash
# List retryable failures
script-runner dlq

# Retry single job
script-runner retry 42

# Retry all DLQ
script-runner retry --all
```

### Other commands

```bash
# Tail recent logs for a script
script-runner tail daily_digest

# Execution stats
script-runner stats
```

## Transient Error Detection

These patterns automatically move failed jobs to DLQ (retryable):

- `token.*expired` — auth token issues (Kiro, API keys)
- `401.*unauthorized` — auth failures
- `connection.*refused` — network issues
- `timeout` — request timeouts
- `rate.?limit` — API rate limiting
- `503.*service.*unavailable` — temporary outages

Non-transient failures stay in `failed` status and won't auto-retry.

## Hermes Cron Integration

Instead of running scripts directly in cron jobs:

```yaml
# Before
script: ~/.hermes/scripts/daily_digest.py

# After  
script: script-runner run daily_digest.py
```

Or wrap in Hermes cron prompt:

```
Run: script-runner run daily_digest.py
If DLQ has items, notify user.
```

## Database Schema

SQLite at `~/.hermes/script-logs.db`:

```sql
-- Run records
runs(id, script, args, started, ended, exit_code, status, error_type, retry_count, parent_run_id)

-- Log lines  
logs(id, run_id, timestamp, stream, line)
```

Query directly:

```bash
sqlite3 ~/.hermes/script-logs.db "SELECT * FROM runs WHERE status='dlq'"
```

## Status Values

| Status | Meaning |
|--------|---------|
| `running` | Currently executing |
| `success` | Completed with exit code 0 |
| `failed` | Failed (non-transient error) |
| `dlq` | Failed with transient error, retryable |
| `retried` | Was in DLQ, has been retried |
