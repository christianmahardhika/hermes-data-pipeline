# Security Review Guide

## When to Trigger Security Review

- Adding new external API integration
- Handling API keys or credentials
- Adding new RSS feed sources
- Modifying HTTP request patterns
- Changing data storage logic
- Adding new Docker services

## Security Checklist

### 1. Secrets Management
- [ ] No hardcoded API keys, tokens, or passwords
- [ ] All secrets from `std::env::var()` (Rust) or `os.environ` (Python)
- [ ] `.env` files in `.gitignore`
- [ ] Secrets never logged (even at debug level)

### 2. Network Security
- [ ] All HTTP clients have timeout configured
- [ ] TLS used for external connections (rustls-tls)
- [ ] User-Agent set on all outgoing requests
- [ ] Rate limiting respected (back off on 429/503)

### 3. Input Validation
- [ ] RSS XML parsed with error handling
- [ ] HTML stripped via ammonia before storage
- [ ] JSON from LLM validated before use
- [ ] UTF-8 encoding errors handled gracefully

### 4. Data Integrity
- [ ] Parameterized SQL queries (rusqlite params![])
- [ ] SHA256 hash dedup prevents duplicate processing
- [ ] Status tracking ensures no items processed twice

### 5. Dependency Security
- Rust: cargo audit, cargo deny check
- Python: pip-audit, bandit
- Container: trivy scan

## Deployment Security Gate
- [ ] cargo audit clean
- [ ] No secrets in codebase
- [ ] All external calls have timeouts
- [ ] Health check endpoint works
