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
| SQLite locked | Concurrent access issue |

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
