# Debugging Guide

## Quick Diagnosis Flow

1. Reproduce — run specific phase
2. Isolate — which phase? (collect/clean/label/embed)
3. Hypothesize — form ONE theory
4. Verify — test with minimal change
5. Fix and verify — apply fix, run cargo test
6. Prevent — add test that catches this bug

## Common Issues

| Symptom | First Check |
|---------|-------------|
| All feeds failing | Network connectivity |
| Single feed failing | Feed URL changed or blocking |
| LLM 401 error | API key expired, run health |
| LLM malformed JSON | Markdown code blocks in response |
| TEI timeout | Model not loaded yet |
| Qdrant connection refused | Container not running |
| RSS feed 403 | User-Agent blocked by CDN |
| RSS feed 404 | Feed URL removed by publisher |
| RSS feed 000/timeout | DNS/network or domain dead |
| SQLite locked | Concurrent access issue |
| Push blocked by secret scanning | Local main diverged from origin (see below) |

## Git Push Protection — Local Main Divergence

**Symptom**: `git push` blocked by GitHub Push Protection on a commit that should already be resolved (e.g., after squash-merge removed the secret).

**Diagnosis**:
```bash
git log --oneline main | head -5       # Local main
git log --oneline origin/main | head -5  # Remote main
# If they differ → local main has stale pre-squash commits
```

**Fix**:
```bash
git checkout main
git reset --hard origin/main              # Align local to remote
git checkout -b new-branch
git cherry-pick <your-clean-commit-hash>  # Only your changes
git push -u origin new-branch
```

**Prevention**: After squash-merging a PR on GitHub, always `git pull --rebase` on local main before creating new branches.

## Debugging Commands

```bash
cargo run --release -- health
sqlite3 news_staging.db "SELECT status, COUNT(*) FROM raw_feeds GROUP BY status;"
curl -s http://localhost:6333/collections | jq
curl -s http://localhost:8082/health
RUST_LOG=debug cargo run -- run
```

## Two-Strike Rule

If approach fails twice: STOP, re-read module, try different approach.
