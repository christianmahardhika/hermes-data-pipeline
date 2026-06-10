# Coding Standards

## Rust (src/)

### Naming
- Crates: lowercase with hyphens (`news-collector`)
- Modules: lowercase with underscores (`mod.rs`)
- Types/Traits: `PascalCase` (`ArticleCleaner`, `CleanStats`)
- Functions/Methods: `snake_case` (`process_pending`, `fetch_feed`)
- Constants: `SCREAMING_SNAKE_CASE` (`MAX_RETRIES`, `DEFAULT_TIMEOUT`)

### Error Handling
- Use `anyhow::Result` for application-level errors
- Use `thiserror` for library-level custom errors
- Always propagate with `?` operator
- Avoid `.unwrap()` in production code
- Log errors at boundaries with `tracing::error!`

### Module Pattern
- One module per pipeline phase
- Struct with `new()` constructor and `process_pending()` method
- Stats struct with Display impl for logging

### Database (rusqlite)
- Parameterized queries: `params![value1, value2]`
- DateTime stored as RFC3339 strings
- JSON stored as TEXT columns
- Use `INSERT OR IGNORE` for dedup

### Security
- No hardcoded secrets — use `std::env::var()`
- Timeout on all HTTP requests
- Input sanitization: `ammonia::clean()` for HTML
- Content hash dedup prevents duplicate processing

## Python (social_intel/)

### Naming
- Modules: `snake_case.py`
- Functions: `snake_case`
- Classes: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Private helpers: `_prefixed`

### Error Handling ("Never Raises")
- All public functions return empty list/None on error
- Log errors to stderr via `_log()` helper
- Never let exceptions propagate to caller
- Use type hints for all function signatures

### Normalized Article Schema
All sources output the same dict structure with: id, title, url, description, source, author, score, num_comments, created_utc, date, relevance, collected_at, content_type, metadata.

### Security
- No API keys in source code
- User-Agent rotation for HTTP requests
- Rate limit awareness (back off on 429/503)
- URL-encode user input
- Timeout on all requests

## General

### Git
- Conventional commits: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
- One logical change per commit
- Branch: `feature/<name>`, `fix/<name>`

### Quality Checklist
- No hardcoded secrets
- All errors handled
- Functions < 50 lines
- Proper logging with context
- Timeouts on external calls
- Deduplication for data integrity
