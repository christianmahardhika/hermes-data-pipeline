# Implementation Plan: Pipeline Re-Architecture

## Overview
Decompose Hermes Data Pipeline monolith into 6 Cargo workspace service crates. Uses Strangler Fig pattern for incremental migration. ArangoDB as shared state store, eliminates SQLite staging layer.

## Tasks

### Phase 1: Foundation (Week 1)

- [ ] 1. Initialize Cargo workspace structure
  - Create `crates/` directory at project root
  - Update root `Cargo.toml` to workspace configuration with `members = ["crates/*"]`
  - Verify: `cargo build` succeeds (no changes to existing binary yet)
  - **Requirement**: Design Section "Workspace Structure"

- [ ] 2. Create hermes-common crate skeleton
  - Create `crates/hermes-common/Cargo.toml` with shared dependencies (anyhow, thiserror, serde, tokio, reqwest, tracing)
  - Create `crates/hermes-common/src/lib.rs` with module declarations
  - Create empty module files: `config.rs`, `types/mod.rs`, `observability/mod.rs`
  - Verify: `cargo build -p hermes-common`
  - **Requirement**: Property 5 (Config Consistency)

- [ ] 3. Extract ArangoDB client to hermes-common
  - Move `src/arangodb/mod.rs` → `crates/hermes-common/src/arangodb/mod.rs`
  - Move `src/arangodb/schema.rs` → `crates/hermes-common/src/arangodb/schema.rs`
  - Move `src/arangodb/ingester.rs` → `crates/hermes-common/src/arangodb/ingester.rs`
  - Update imports to use `hermes_common::arangodb`
  - Update monolith `Cargo.toml` to depend on `hermes-common = { path = "crates/hermes-common" }`
  - Verify: `cargo test` all existing tests pass
  - **Requirement**: Design Section "hermes-common (Shared Library)"

- [ ] 4. Extract shared domain types to hermes-common
  - Create `crates/hermes-common/src/types/article.rs` with `Article`, `ArticleStatus` structs
  - Create `crates/hermes-common/src/types/actor.rs` with `Actor` struct
  - Create `crates/hermes-common/src/types/topic.rs` with `Topic` struct
  - Create `crates/hermes-common/src/types/signal.rs` with `Signal`, `ExternalSignal` structs
  - Update `types/mod.rs` to re-export all types
  - Verify: `cargo build -p hermes-common`
  - **Requirement**: Design Section "Core Domain Models"

- [ ] 5. Create centralized config in hermes-common
  - Create `crates/hermes-common/src/config.rs` with `HermesConfig` struct
  - Consolidate env var reading: `ARANGO_URL`, `ARANGO_DATABASE`, `ARANGO_USERNAME`, `ARANGO_PASSWORD`, `TEI_URL`, `LABELER_BASE_URL`, `LABELER_API_KEY`, `LABELER_MODEL`, `STORAGE_BACKEND`
  - Implement `HermesConfig::from_env()` with validation
  - Add unit tests for config parsing with missing/invalid env vars
  - Verify: `cargo test -p hermes-common`
  - **Requirement**: Property 5 (Config Consistency), Design Table "Key Exports"

- [ ] 6. Add observability primitives to hermes-common
  - Create `crates/hermes-common/src/observability/logging.rs` with `init_logging()` function
  - Create `crates/hermes-common/src/observability/metrics.rs` with `MetricsRegistry` struct (stub for Prometheus)
  - Export via `observability/mod.rs`
  - Update monolith to use `hermes_common::observability::init_logging()`
  - Verify: `cargo run -- health` still works with new logging
  - **Requirement**: Design Table "Key Exports" - observability module

### Phase 2: Extract hermes-collector (Week 2)

- [ ] 7. Create hermes-collector crate skeleton
  - Create `crates/hermes-collector/Cargo.toml` with dependencies on `hermes-common`, `feed-rs`, `reqwest`, `tokio`
  - Create `crates/hermes-collector/src/main.rs` with CLI entry (clap)
  - Create empty modules: `feeds.rs`, `collector.rs`, `circuit.rs`
  - Verify: `cargo build -p hermes-collector`
  - **Requirement**: Design Section "hermes-collector (News Collection Service)"

- [ ] 8. Move feed configuration to hermes-collector
  - Move `FeedConfig`, `FeedCategory` structs from `src/collectors/mod.rs` to `crates/hermes-collector/src/feeds.rs`
  - Move all 31 feed entries from `RssCollector::new()` to `feeds.rs`
  - Create `load_feeds() -> Vec<FeedConfig>` function
  - Write unit test verifying 31 feeds with correct categories
  - Verify: `cargo test -p hermes-collector`
  - **Requirement**: Property 2 (Data Integrity) - feed config matches existing

- [ ] 9. Implement collector with direct ArangoDB write
  - Move `fetch_feed()` logic from `src/collectors/mod.rs` to `crates/hermes-collector/src/collector.rs`
  - Create `RssCollector` struct using `hermes_common::arangodb::ArangoClient`
  - Implement `collect_all()` writing directly to ArangoDB `articles` collection with `status: "raw"`
  - **Key change**: Skip SQLite staging, write directly to ArangoDB
  - Verify: `cargo run -p hermes-collector -- collect` inserts articles to ArangoDB
  - **Requirement**: Design Section "Target Data Flow (Direct ArangoDB)"

- [ ] 10. Move circuit breaker to hermes-collector
  - Move `CircuitState`, `determine_circuit_state()`, `handle_half_open_result()` to `crates/hermes-collector/src/circuit.rs`
  - Store circuit state in ArangoDB `feed_health` collection (not SQLite)
  - Implement AQL queries for circuit state read/write
  - Write unit tests for circuit state transitions
  - Verify: `cargo test -p hermes-collector`
  - **Requirement**: Property 1 from news-source-resilience (Circuit Breaker)

- [ ] 11. Add CLI commands to hermes-collector
  - Implement `collect` subcommand: one-shot collection
  - Implement `daemon` subcommand: loop every 15 minutes
  - Implement `health` subcommand: report feed health status
  - Implement `prune --days N` subcommand: remove old articles
  - Verify: all commands work end-to-end
  - **Requirement**: Design Table "CLI Commands" for hermes-collector

### Phase 3: Extract hermes-processor (Week 3)

- [ ] 12. Create hermes-processor crate skeleton
  - Create `crates/hermes-processor/Cargo.toml` with dependencies on `hermes-common`, `ammonia`, `sha2`
  - Create `crates/hermes-processor/src/main.rs` with CLI entry
  - Create empty modules: `cleaner.rs`, `labeler.rs`, `embedder.rs`
  - Verify: `cargo build -p hermes-processor`
  - **Requirement**: Design Section "hermes-processor (Processing Pipeline)"

- [ ] 13. Implement cleaner phase in hermes-processor
  - Move HTML stripping logic from `src/cleaners/mod.rs` to `crates/hermes-processor/src/cleaner.rs`
  - Query ArangoDB for `status: "raw"` articles
  - Process: strip HTML, normalize whitespace, generate SHA256 hash
  - Update article `status` to `"cleaned"` in ArangoDB
  - Write unit tests for HTML cleaning edge cases
  - Verify: `cargo test -p hermes-processor`
  - **Requirement**: Property 1 (Status Transitions) - raw → cleaned

- [ ] 14. Implement labeler phase in hermes-processor
  - Move Prof Jiang labeling logic from `src/labelers/mod.rs` to `crates/hermes-processor/src/labeler.rs`
  - Query ArangoDB for `status: "cleaned"` articles
  - Batch LLM requests (20 articles per batch)
  - Update article with `labels` object and `status: "labeled"`
  - Write unit test with mocked LLM response
  - Verify: `cargo test -p hermes-processor`
  - **Requirement**: Property 1 (Status Transitions) - cleaned → labeled

- [ ] 15. Implement embedder phase in hermes-processor
  - Move embedding logic from `src/embedders/mod.rs` to `crates/hermes-processor/src/embedder.rs`
  - Query ArangoDB for `status: "labeled"` articles
  - Batch TEI requests (100 articles per batch)
  - Update article with `embedding` array and `status: "ingested"`
  - Create graph edges: `article_mentions_actor`, `article_has_topic`
  - Verify: `cargo run -p hermes-processor -- run` processes all phases
  - **Requirement**: Property 1 (Status Transitions) - labeled → ingested

- [ ] 16. Add CLI commands to hermes-processor
  - Implement `run` subcommand: process all pending articles
  - Implement `run --phase clean|label|embed`: run single phase
  - Implement `run --limit N`: process up to N articles
  - Implement `daemon`: continuous processing loop
  - Verify: all commands work end-to-end
  - **Requirement**: Design Table "CLI Commands" for hermes-processor

### Phase 4: SQLite Migration (Week 4)

- [ ] 17. Create SQLite → ArangoDB migration script
  - Create `scripts/migrate_sqlite_to_arango.py` (or Rust binary)
  - Read all articles from SQLite `raw_feeds`, `cleaned`, `labeled` tables
  - Map status: raw → raw, cleaned → cleaned, labeled → labeled, ingested → ingested
  - Insert to ArangoDB with correct status
  - Log progress and handle duplicates (skip if `_key` exists)
  - **Requirement**: Design Section "Phase 4: SQLite Migration"

- [ ] 18. Run migration on staging data
  - Backup SQLite database: `cp news_staging.db news_staging.db.bak`
  - Run migration script against local ArangoDB
  - Verify data integrity: count articles per status in both databases
  - Verify no data loss: all articles migrated with correct fields
  - **Requirement**: Property 2 (Data Integrity)

- [ ] 19. Remove SQLite dependencies from monolith
  - Remove `rusqlite` from root `Cargo.toml` (keep in legacy/ if needed)
  - Remove `src/storage/mod.rs` (SQLite operations)
  - Update monolith to use hermes-collector and hermes-processor
  - Verify: `cargo build` succeeds without rusqlite
  - **Requirement**: Design Section "After: Services with Direct ArangoDB"

### Phase 5: Extract hermes-social (Week 5)

- [ ] 20. Create hermes-social crate skeleton
  - Create `crates/hermes-social/Cargo.toml` with dependencies on `hermes-common`, `quick-xml`
  - Create `crates/hermes-social/src/main.rs` with CLI entry
  - Create empty modules: `hackernews.rs`, `reddit.rs`, `youtube.rs`
  - Verify: `cargo build -p hermes-social`
  - **Requirement**: Design Section "hermes-social (Social Intelligence Service)"

- [ ] 21. Move HackerNews collector to hermes-social
  - Move `src/social/hackernews.rs` → `crates/hermes-social/src/hackernews.rs`
  - Update to use `hermes_common::arangodb` for storage
  - Update to use 768-dim TEI embeddings (unified with news)
  - Store in ArangoDB `social_posts` collection
  - Write integration test with mock Algolia response
  - Verify: `cargo test -p hermes-social`
  - **Requirement**: Design Table "deprecates Python social_intel"

- [ ] 22. Move Reddit collector to hermes-social
  - Move `src/social/reddit.rs` → `crates/hermes-social/src/reddit.rs`
  - Update to use `hermes_common::arangodb` for storage
  - Verify RSS/Atom parsing works for all subreddits
  - Write unit test for atom feed parsing
  - Verify: `cargo test -p hermes-social`
  - **Requirement**: Same as Task 21

- [ ] 23. Move YouTube collector to hermes-social
  - Move `src/social/youtube.rs` → `crates/hermes-social/src/youtube.rs`
  - Update to use `hermes_common::arangodb` for storage
  - Verify metadata extraction works
  - Write unit test for metadata parsing
  - Verify: `cargo test -p hermes-social`
  - **Requirement**: Same as Task 21

- [ ] 24. Add CLI commands to hermes-social
  - Implement `collect --topics "AI,tech"`: topic-based collection
  - Implement `collect --front-page`: front page collection
  - Implement `collect --depth quick|default|deep`: collection depth
  - Implement `daemon`: run collection every 2 hours
  - Verify: all commands work end-to-end
  - **Requirement**: Design Table "CLI Commands" for hermes-social

- [ ] 25. Deprecate Python social_intel module
  - Create `legacy/social_intel/` directory
  - Move `news-social-intelligence-data-pipeline/social_intel/` to `legacy/`
  - Update `social_intel_cron.py` to call `hermes-social` binary instead
  - Document migration in `legacy/README.md`
  - Verify: cron still works with new binary
  - **Requirement**: Design "Deprecates: Python social_intel/ module"

### Phase 6: Extract hermes-economic (Week 6)

- [ ] 26. Create hermes-economic crate skeleton
  - Create `crates/hermes-economic/Cargo.toml` with dependencies on `hermes-common`
  - Create `crates/hermes-economic/src/main.rs` with CLI entry
  - Create empty modules: `collector.rs`, `yahoo.rs`, `coingecko.rs`, `fred.rs`, `bank_indonesia.rs`
  - Verify: `cargo build -p hermes-economic`
  - **Requirement**: Design Section "hermes-economic (Economic Data Service)"

- [ ] 27. Create unified collector trait in hermes-economic
  - Create `crates/hermes-economic/src/collector.rs` with `EconomicCollector` trait
  - Define methods: `collect(&self) -> Result<Vec<EconomicIndicator>>`, `source(&self) -> EconomicSource`
  - Define `RateLimitConfig` struct for per-source rate limiting
  - Export via `lib.rs`
  - **Requirement**: Design "Unified economic data collector with consistent interface"

- [ ] 28. Move Yahoo commodities to hermes-economic
  - Move `src/economic/yahoo_commodities.rs` → `crates/hermes-economic/src/yahoo.rs`
  - Implement `EconomicCollector` trait for `YahooCollector`
  - Add rate limiting: 150ms stagger between symbols
  - Write unit test with mock response
  - Verify: `cargo test -p hermes-economic`
  - **Requirement**: Design Table "CLI Commands" - `collect commodity`

- [ ] 29. Move CoinGecko to hermes-economic
  - Move `src/economic/coingecko.rs` → `crates/hermes-economic/src/coingecko.rs`
  - Implement `EconomicCollector` trait for `CoinGeckoCollector`
  - Add rate limiting: 10 calls/min
  - Write unit test with mock response
  - Verify: `cargo test -p hermes-economic`
  - **Requirement**: Design Table "CLI Commands" - `collect crypto`

- [ ] 30. Move FRED to hermes-economic
  - Move `src/economic/fred.rs` → `crates/hermes-economic/src/fred.rs`
  - Implement `EconomicCollector` trait for `FredCollector`
  - Graceful degradation if `FRED_API_KEY` not set
  - Write unit test with mock response
  - Verify: `cargo test -p hermes-economic`
  - **Requirement**: Design Table "CLI Commands" - `collect fred`

- [ ] 31. Move Bank Indonesia to hermes-economic
  - Move `src/economic/bank_indonesia.rs` → `crates/hermes-economic/src/bank_indonesia.rs`
  - Implement `EconomicCollector` trait for `BankIndonesiaCollector`
  - Write unit test with mock response
  - Verify: `cargo test -p hermes-economic`
  - **Requirement**: Design Table "CLI Commands" - `collect bi`

- [ ] 32. Add CLI commands to hermes-economic
  - Implement `collect all`: collect from all sources
  - Implement `collect commodity|crypto|fred|bi`: single source
  - Implement `daemon`: run collection every hour
  - Add Prometheus metrics per source
  - Verify: all commands work end-to-end
  - **Requirement**: Design Table "CLI Commands" for hermes-economic

### Phase 7: Extract hermes-analyst (Week 7-8)

- [ ] 33. Create hermes-analyst crate skeleton
  - Create `crates/hermes-analyst/Cargo.toml` with dependencies on `hermes-common`, `axum`, `tokio`
  - Create `crates/hermes-analyst/src/main.rs` with HTTP server setup
  - Create empty modules: `api.rs`, `debate.rs`, `trader.rs`, `formatter.rs`
  - Verify: `cargo build -p hermes-analyst`
  - **Requirement**: Design Section "hermes-analyst (IDX Analyst API Service)"

- [ ] 34. Move debate engine to hermes-analyst
  - Move `src/idx_analyst/debate.rs` → `crates/hermes-analyst/src/debate.rs`
  - Move `src/idx_analyst/models.rs` → `crates/hermes-analyst/src/models.rs`
  - Update to use `hermes_common::types::ExternalSignal`
  - Write unit test for 5-persona debate logic
  - Verify: `cargo test -p hermes-analyst`
  - **Requirement**: Design "5-persona bull/bear debate engine"

- [ ] 35. Move trader and risk modules to hermes-analyst
  - Move `src/idx_analyst/trader.rs` → `crates/hermes-analyst/src/trader.rs`
  - Move `src/idx_analyst/risk.rs` → `crates/hermes-analyst/src/risk.rs`
  - Write unit tests for trade proposal generation
  - Write unit tests for portfolio constraint validation
  - Verify: `cargo test -p hermes-analyst`
  - **Requirement**: Design "Trade proposal generation"

- [ ] 36. Move formatter to hermes-analyst
  - Move `src/idx_analyst/formatter.rs` → `crates/hermes-analyst/src/formatter.rs`
  - Support multiple output formats: JSON, RTI Business, Telegram Markdown
  - Write unit tests for each format
  - Verify: `cargo test -p hermes-analyst`
  - **Requirement**: Design "Multiple output formats"

- [ ] 37. Implement REST API endpoints
  - Implement `GET /api/v1/analyze/{ticker}`: single ticker analysis
  - Implement `GET /api/v1/portfolio`: full portfolio analysis
  - Implement `GET /api/v1/digest`: daily digest
  - Implement `GET /api/v1/signals/{ticker}`: external signals lookup
  - Add request validation and error handling
  - Write integration tests for each endpoint
  - Verify: `cargo test -p hermes-analyst`
  - **Requirement**: Design Table "API Endpoints"

- [ ] 38. Implement health check endpoints
  - Implement `GET /health/live`: liveness probe (always 200)
  - Implement `GET /health/ready`: readiness probe (check ArangoDB connection)
  - Add Prometheus metrics endpoint `/metrics`
  - Write integration tests for health endpoints
  - Verify: `curl localhost:8080/health/ready` returns 200
  - **Requirement**: Design Table "API Endpoints" - health checks

### Phase 8: Observability & Cleanup (Week 9-10)

- [ ] 39. Add structured JSON logging to all services
  - Update `hermes_common::observability::init_logging()` to output JSON format
  - Add log levels configurable via `RUST_LOG` env var
  - Add request_id/trace_id for request correlation
  - Update all services to use structured logging
  - Verify: logs are JSON parseable
  - **Requirement**: Design Section "Observability"

- [ ] 40. Add Prometheus metrics to all services
  - Create `hermes_common::observability::metrics` with `Counter`, `Histogram` types
  - Add collection metrics: `articles_collected_total`, `collection_duration_seconds`
  - Add processing metrics: `articles_processed_total`, `phase_duration_seconds`
  - Add API metrics: `http_requests_total`, `http_request_duration_seconds`
  - Expose `/metrics` endpoint on each service
  - Verify: `curl localhost:9090/metrics` returns Prometheus format
  - **Requirement**: Design Section "Observability" - Prometheus

- [ ] 41. Update docker-compose.yml for new services
  - Add service definitions for: hermes-collector, hermes-processor, hermes-social, hermes-economic, hermes-analyst
  - Remove SQLite volume mount
  - Add Prometheus service for metrics collection
  - Add healthcheck directives for each service
  - Verify: `docker compose up` starts all services
  - **Requirement**: Design Section "Infrastructure Update"

- [ ] 42. Create .env.canonical configuration reference
  - Create `infrastructure/.env.canonical` with all env vars documented
  - Group by service: common, collector, processor, analyst
  - Add validation rules as comments
  - Update `infrastructure/README.md` with setup instructions
  - **Requirement**: Property 5 (Config Consistency)

- [ ] 43. Update steering and documentation
  - Update `product.md` with new service topology
  - Update `tech.md` with new build commands per service
  - Update `structure.md` with workspace layout
  - Update `PIPELINE.md` data flow diagrams
  - Update root `README.md` with quick start guide
  - **Requirement**: Design Section "Documentation"

- [ ] 44. Move legacy code to legacy/ directory
  - Move deprecated monolith code to `legacy/monolith/`
  - Move Python social_intel to `legacy/social_intel/`
  - Add `legacy/README.md` explaining deprecation
  - Ensure legacy code is excluded from workspace build
  - Verify: `cargo build` only builds crates/
  - **Requirement**: Design Section "Workspace Structure" - legacy/

## Task Dependency Graph

```json
{
  "waves": [
    {
      "name": "Phase 1 - Foundation",
      "tasks": [1, 2, 3, 4, 5, 6],
      "description": "Workspace setup + hermes-common extraction (some parallel)"
    },
    {
      "name": "Phase 2 - Collector",
      "tasks": [7, 8, 9, 10, 11],
      "description": "hermes-collector extraction (sequential)"
    },
    {
      "name": "Phase 3 - Processor",
      "tasks": [12, 13, 14, 15, 16],
      "description": "hermes-processor extraction (sequential)"
    },
    {
      "name": "Phase 4 - Migration",
      "tasks": [17, 18, 19],
      "description": "SQLite → ArangoDB migration (sequential)"
    },
    {
      "name": "Phase 5 - Social",
      "tasks": [20, 21, 22, 23, 24, 25],
      "description": "hermes-social extraction (21-23 parallel)"
    },
    {
      "name": "Phase 6 - Economic",
      "tasks": [26, 27, 28, 29, 30, 31, 32],
      "description": "hermes-economic extraction (28-31 parallel)"
    },
    {
      "name": "Phase 7 - Analyst",
      "tasks": [33, 34, 35, 36, 37, 38],
      "description": "hermes-analyst extraction (sequential)"
    },
    {
      "name": "Phase 8 - Observability",
      "tasks": [39, 40, 41, 42, 43, 44],
      "description": "Logging, metrics, docs, cleanup (some parallel)"
    }
  ]
}
```

```
Visual dependency:
Phase 1 (Foundation):
  1 (Workspace) ──→ 2 (common skeleton) ──→ 3 (ArangoDB) ──┐
                                            4 (types) ─────┤
                                            5 (config) ────┤──→ 6 (observability)
                                                           │
Phase 2 (Collector):                                       │
  7 (collector skeleton) ←─────────────────────────────────┘
    ↓
  8 (feeds) ──→ 9 (direct ArangoDB) ──→ 10 (circuit) ──→ 11 (CLI)

Phase 3 (Processor):
  12 (processor skeleton) ──→ 13 (cleaner) ──→ 14 (labeler) ──→ 15 (embedder) ──→ 16 (CLI)

Phase 4 (Migration):
  17 (migration script) ──→ 18 (run migration) ──→ 19 (remove SQLite)

Phase 5 (Social):
  20 (social skeleton) ──→ 21 (HN) ──┐
                           22 (Reddit) ──┤──→ 24 (CLI) ──→ 25 (deprecate Python)
                           23 (YouTube) ─┘

Phase 6 (Economic):
  26 (economic skeleton) ──→ 27 (trait) ──→ 28 (Yahoo) ──┐
                                            29 (Gecko) ──┤──→ 32 (CLI)
                                            30 (FRED) ───┤
                                            31 (BI) ─────┘

Phase 7 (Analyst):
  33 (analyst skeleton) ──→ 34 (debate) ──→ 35 (trader/risk) ──→ 36 (formatter) ──→ 37 (REST API) ──→ 38 (health)

Phase 8 (Observability):
  39 (logging) ──┐
  40 (metrics) ──┤──→ 41 (docker) ──→ 42 (env) ──→ 43 (docs) ──→ 44 (legacy)
```

## Notes

- **Phase 1** must complete before any other phase starts (hermes-common is dependency for all services)
- **Phase 2-3** can run in parallel after Phase 1 (collector and processor are independent)
- **Phase 4** (SQLite migration) blocks Phase 5-6 (must ensure data integrity first)
- **Phase 5-6** (social + economic) can run in parallel
- **Phase 7** (analyst) depends on hermes-common but can start after Phase 1
- **Phase 8** (observability) should be done last to ensure all services are stable

### Quality Gates

Each phase must pass before proceeding:

| Phase | Gate Criteria |
|-------|---------------|
| 1 | `cargo build` + `cargo test -p hermes-common` pass |
| 2 | `hermes-collector -- collect` writes to ArangoDB |
| 3 | `hermes-processor -- run` processes raw → ingested |
| 4 | All articles migrated, counts match |
| 5 | Python social_intel replaced by Rust binary |
| 6 | All economic collectors work via unified interface |
| 7 | REST API returns correct analysis |
| 8 | All services have metrics + JSON logging |

### Breaking Changes

- **SQLite removal**: After Phase 4, SQLite staging is gone. Existing scripts reading SQLite will break.
- **Python deprecation**: After Phase 5, Python social_intel cron must use Rust binary.
- **Monolith removal**: After Phase 8, `cargo run --` commands change to `cargo run -p hermes-<service> --`

### Rollback Strategy

Each phase can be rolled back independently:

1. **Phase 1-6**: Revert to monolith binary (still in repo until Phase 8)
2. **Phase 4**: Restore SQLite from backup, revert code changes
3. **Phase 8**: Move legacy code back, rebuild monolith
