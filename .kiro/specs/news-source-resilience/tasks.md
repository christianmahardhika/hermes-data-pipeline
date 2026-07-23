# Implementation Plan: News Source Resilience & Intelligence Expansion

## Overview
Fix RSS feed resilience (circuit breaker, freshness tracking, CI health) and expand data sources (new RSS feeds, economic data module, ArangoDB integration). Three phases over 3-4 weeks.

## Tasks

- [x] 1. Add FeedCategory enum and update FeedConfig struct
  - Create `FeedCategory` enum: `Indonesian`, `InternationalBusiness`, `InternationalGeneral`, `AsiaPacific`, `Market`, `Tech`
  - Add `category: FeedCategory` field to `FeedConfig`
  - Update all existing feed entries with correct category
  - Implement Display trait, add derive macros
  - Verify: `cargo build` + `cargo test` (46 tests unchanged)

- [x] 2. Schema migration — add circuit breaker columns to feed_health
  - Add `circuit_open_until TEXT`, `backoff_secs INTEGER DEFAULT 3600`, `category TEXT` to feed_health
  - Create `source_freshness` table with index
  - Use ALTER TABLE with existence check for backward compat
  - Verify: `cargo build` + `cargo test`

- [x] 3. Implement circuit breaker state machine
  - Add `CircuitState` enum: Closed, Open, HalfOpen
  - Implement `determine_circuit_state()`, `set_circuit_open()`, `reset_circuit()`, `handle_half_open_result()`
  - Backoff doubling: 3600 → 7200 → 14400 → 21600 (6h cap)
  - Write unit tests for threshold boundary, backoff cap, state transitions
  - Verify: `cargo test` new tests pass

- [x] 4. Integrate circuit breaker into collect_all()
  - Check circuit state before fetch_feed(), skip if Open, probe if HalfOpen
  - Log state transitions with emoji convention
  - Add `skipped` count + `per_category: HashMap<FeedCategory, CategoryStats>` to CollectStats
  - Write integration test with mix of healthy/open/half-open feeds
  - Verify: `cargo test` + `cargo clippy` clean

- [x] 5. Validate and add new RSS feeds
  - HTTP check candidates: Jakarta Globe, Katadata, Reuters, Investing.com, CNBC ID Market, CNBC ID Tech, DW Indonesia
  - Verify returns valid RSS/Atom XML with recent articles (relax 7-day stability check — CI workflow Task 7 catches regressions)
  - Add passing feeds with correct FeedCategory and fallback_urls
  - Verify: `cargo run -- collect` succeeds, feed count >= 30

- [x] 6. Source freshness tracking
  - Implement `update_source_freshness()` and `get_stale_sources()`
  - Integrate into health endpoint output
  - Write test: source stale after 48h flagged
  - Verify: `cargo test` passes

- [x] 7. Feed Health CI workflow
  - Create `.github/workflows/feed-health.yml` with daily cron + manual trigger
  - Curl each feed URL, record status + response time, fail if < 80% success
  - Upload results as artifact

- [x] 8. ArangoDB client module scaffolding
  - Create `src/arangodb/mod.rs` with ArangoClient struct (raw reqwest HTTP — no arangors crate, matches Python script pattern)
  - Implement connection, query_aql, insert_document, insert_edge
  - Config via env vars: ARANGO_URL, ARANGO_DATABASE, ARANGO_USERNAME, ARANGO_PASSWORD
  - Write connection test
  - Verify: `cargo build` + `cargo test`

- [x] 9. ArangoDB schema initialization
  - Implement ensure_collections (7 document), ensure_edge_collections (6 edge), ensure_graph, ensure_views
  - Idempotent: check existence before create
  - Verify ArangoDB version >= 3.10 (required for APPROX_NEAR vector search)
  - Configure vector index on embedding field with 768 dimensions in articles_vector_view
  - Write test: schema init on fresh database
  - Verify: `cargo build`

- [x] 10. Embedder → ArangoDB ingestion (replace Qdrant path)
  - Create `src/arangodb/ingester.rs`
  - Insert articles with embedding (stored as array of floats), extract actors/events into graph
  - Near-dup check via ArangoSearch APPROX_NEAR (0.92 threshold)
  - Dedup by content_hash as _key
  - Add `STORAGE_BACKEND` env var: `arangodb` (default) or `qdrant` (legacy fallback)
  - Keep existing Qdrant code path as opt-in fallback (don't remove yet)
  - Write integration test: insert + verify graph edges
  - Verify: `cargo build` + `cargo test`

- [x] 11. Economic data module — models + storage
  - Create `src/economic/mod.rs` + `models.rs`
  - EconomicIndicator struct, EconomicSource enum
  - Storage: insert into ArangoDB economic_indicators + signal_source edges
  - Add `pub mod economic;` to lib.rs
  - Verify: `cargo build` + unit tests

- [x] 12. Yahoo Finance commodity client
  - Create `src/economic/yahoo_commodities.rs`
  - Fetch 9 commodity symbols with 150ms stagger
  - Parse into EconomicIndicator, store in ArangoDB + signal_source edges
  - Write test with mock response
  - Verify: `cargo build` + `cargo test`

- [x] 13. CoinGecko crypto client
  - Create `src/economic/coingecko.rs`
  - Fetch BTC, ETH, USDT, BNB, XRP with rate limiting (10 calls/min)
  - Parse into EconomicIndicator, store in ArangoDB
  - Write test with mock response
  - Verify: `cargo build` + `cargo test`

- [x] 14. FRED API client
  - Create `src/economic/fred.rs`
  - Fetch 6 series (FEDFUNDS, CPIAUCSL, UNRATE, GDP, DGS10, DTWEXBGS)
  - Graceful degradation if FRED_API_KEY not set
  - Store in ArangoDB + signal_source edges
  - Write test with mock response
  - Verify: `cargo build` + `cargo test`

- [x] 15. Bank Indonesia data client
  - Create `src/economic/bank_indonesia.rs`
  - Fetch BI Rate, JIBOR, USD/IDR, inflation from public API
  - Store in ArangoDB + edges to banking tickers
  - Write test with mock response
  - Verify: `cargo build` + `cargo test`

- [x] 16. CLI integration for economic commands
  - Add `economic` command to main.rs with subcommands (fred, bi, commodity, crypto, daemon)
  - Schedule: commodities 30min, crypto 60min, FRED daily, BI daily
  - Verify: `cargo build` + `cargo run -- economic`

- [x] 17. ExternalSignal struct + debate engine integration
  - Create ExternalSignal struct with source, direction, confidence
  - Add external_signals parameter to PersonaDebateEngine::run_debate()
  - IDX Guru persona: modify confidence based on external signals
  - Existing tests unchanged (empty signals = same behavior)
  - Write unit test: signal modifies debate outcome
  - Verify: `cargo test`

- [x] 18. Graph-based signal lookup for IDX Analyst
  - Query ArangoDB signal_source edges for ticker before debate
  - Convert economic changes into ExternalSignal structs
  - Apply temporal decay function: `strength = base_confidence * exp(-0.693 * hours_elapsed / half_life)` per event type
  - Query mentions edges for recent sentiment context
  - Feed signals into run_debate()
  - Write integration test
  - Verify: `cargo build`

- [x] 19. GDELT event collector (basic)
  - Create `src/economic/gdelt.rs`
  - Query GDELT API v2 for Indonesian events
  - Store in ArangoDB events collection + impacts edges
  - Schedule: every 2 hours
  - Write test with mock response
  - Verify: `cargo build` + `cargo test`

- [x] 20. Update steering + docs to reflect ArangoDB
  - Update product.md, tech.md, structure.md, intelligence-patterns.md
  - Update docker-compose.yml (add ArangoDB, keep Qdrant as optional)
  - Update PIPELINE.md data flow diagram
  - Update Principal Data Intelligence skill

## Task Dependency Graph

```json
{
  "waves": [
    {
      "name": "Phase 1 - Foundation",
      "tasks": [1, 2],
      "description": "FeedCategory enum + schema migration (parallelizable)"
    },
    {
      "name": "Phase 1 - Circuit Breaker",
      "tasks": [3, 4],
      "description": "State machine + collect_all integration (sequential)"
    },
    {
      "name": "Phase 1 - Expansion & Monitoring",
      "tasks": [5, 6, 7],
      "description": "New feeds + freshness + CI (parallelizable)"
    },
    {
      "name": "Phase 2 - ArangoDB Foundation",
      "tasks": [8, 9, 10],
      "description": "Client + schema + ingester (sequential)"
    },
    {
      "name": "Phase 2 - Economic Module",
      "tasks": [11, 12, 13, 14, 15, 16],
      "description": "Models + API clients + CLI (12-15 parallelizable)"
    },
    {
      "name": "Phase 3 - Intelligence Fusion",
      "tasks": [17, 18, 19, 20],
      "description": "ExternalSignal + graph lookup + GDELT + docs"
    }
  ]
}
```

```
Visual dependency:
  1 (FeedCategory) ─┐
  2 (Schema)  ──────┤
                    ├──→ 3 (Circuit Breaker) ──→ 4 (collect_all integration)
                    │
  5 (New Feeds) ────┘    6 (Freshness) ──→ 7 (CI workflow)

  8 (ArangoDB client) ──→ 9 (Schema init) ──→ 10 (Ingester)
  11 (Economic models) ──→ 12 (Yahoo) ──┐
                           13 (Crypto) ──┤──→ 16 (CLI)
                           14 (FRED) ────┤
                           15 (BI) ──────┘

  10 + 11 ──→ 17 (ExternalSignal) ──→ 18 (Graph signal lookup)
  8 + 9 ──→ 19 (GDELT)
  All ──→ 20 (Docs update)
```

## Notes

- Phase 1 can start immediately — no ArangoDB dependency (SQLite staging only)
- Phase 2 requires ArangoDB running locally (docker-compose)
- Phase 3 depends on Phase 2 completion (ArangoDB client + economic data must work first)
- Tasks 12-15 are parallelizable (independent API clients)
- Task 17 is backward-compatible: empty external_signals = unchanged debate behavior
- FRED and BI clients gracefully degrade without API keys (skip with warning, don't fail)
- All tests that require external services (ArangoDB, Yahoo, etc.) should be marked #[ignore] for CI without infra
