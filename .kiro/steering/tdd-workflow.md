# TDD Workflow Guide

## The Cycle: RED → GREEN → REFACTOR

1. RED — Write a test that FAILS
2. GREEN — Write MINIMUM code to make test pass
3. REFACTOR — Improve without changing behavior

## RED Phase Rules

- Test compiles and runs
- Test FAILS because business logic is missing
- JANGAN edit src/*.rs or social_intel/*.py before test fails

## GREEN Phase Rules

- Write LEAST code that makes test pass
- Don't optimize or refactor yet

## REFACTOR Phase Rules

- Tests MUST still pass
- Improve naming, DRY, extract functions

## Verification

```bash
cargo test -- --nocapture   # Rust
pytest -v                    # Python
```

## Coverage Targets

| Component | Target |
|-----------|--------|
| Rust src/ | 70%+ |
| Python social_intel/ | 70%+ |

## Test Types

### Rust:
1. Unit test — #[test] for pure logic
2. Integration test — with temp SQLite DB
3. Async test — #[tokio::test]

### Python:
1. Unit test — test_*.py for parsing
2. "Never raises" test — returns empty on error

## Git Checkpoint

```
git commit -m "test: add failing test for <feature>"
git commit -m "feat: implement <feature>"
git commit -m "refactor: clean up <feature>"
```
