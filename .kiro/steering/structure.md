# Project Structure

Monorepo with multiple data pipelines sharing common infrastructure.

## Active Pipeline: News & Social Intelligence

```
news-social-intelligence-data-pipeline/
├── Cargo.toml                    # Rust dependencies
├── README.md                     # Pipeline documentation
├── src/                          # Rust RSS Pipeline
│   ├── main.rs                   # CLI entry (run/collect/clean/label/embed/daemon/health/prune)
│   ├── lib.rs                    # Config struct, module exports
│   ├── collectors/mod.rs         # RSS feed fetching (29 sources, retry with backoff)
│   ├── cleaners/mod.rs           # HTML strip, normalize, SHA256 dedup
│   ├── labelers/mod.rs           # Prof Jiang batch labeling via LLM API
│   ├── embedders/mod.rs          # TEI embeddings + Qdrant ingestion
│   ├── storage/mod.rs            # SQLite operations (raw, cleaned, labeled, ingested)
│   └── health/mod.rs             # Kiromania health check + self-healing
│
├── src/idx_analyst/              # Enhanced IDX Stock Analyst (Rust port)
│   ├── mod.rs                    # Orchestrator (IdxAnalyst struct)
│   ├── config.rs                 # Portfolio tickers, criteria, risk/execution config
│   ├── models.rs                 # StockData, Signal, Confidence enums
│   ├── debate.rs                 # 5-persona bull/bear debate engine
│   ├── trader.rs                 # Trade proposal (entry/stop/target)
│   ├── risk.rs                   # Portfolio constraint validation
│   ├── memory.rs                 # Decision logging to markdown
│   ├── formatter.rs              # RTI Business + Telegram output
│   └── data_source.rs            # Yahoo Finance API + mock data
│
├── social_intel/                 # Python Social Media Pipeline
│   ├── __init__.py               # Module exports
│   ├── collector.py              # Unified collector (SentenceTransformer + Qdrant)
│   ├── hackernews.py             # HackerNews via Algolia API
│   ├── reddit.py                 # Reddit via RSS/Atom feeds
│   ├── youtube.py                # YouTube via yt-dlp
│   ├── x_twitter.py              # X/Twitter via xurl CLI (disabled)
│   ├── near_duplicate.py         # Cross-source duplicate detection
│   ├── pipeline_integration.py   # Integration with Rust pipeline
│   └── lib/                      # Shared utilities
│       ├── __init__.py
│       ├── http.py               # HTTP get_text/get_json with retry
│       └── relevance.py          # Token overlap relevance scoring
│
└── social_intel_cron.py          # Social media cron runner
```

## Planned Pipelines (Stub only)

```
market-data-pipeline/
├── README.md                     # Design doc for IDX/forex/commodities

social-media-pipeline/
├── README.md                     # Design doc for dedicated social monitoring

knowledge-ingestion-pipeline/
├── README.md                     # Design doc for PDF/EPUB → Qdrant
```

## Shared Infrastructure

```
infrastructure/
├── .env.example                  # Environment variables template
├── README.md                     # Infrastructure setup guide
├── docker-compose.yml            # Qdrant + TEI + optional Redis/PostgreSQL

docs/
├── PIPELINE.md                   # Complete data flow documentation
```

## Key Files to Know

- `news-social-intelligence-data-pipeline/Cargo.toml` — Rust dependencies
- `news-social-intelligence-data-pipeline/src/lib.rs` — Config (env vars, defaults)
- `news-social-intelligence-data-pipeline/src/main.rs` — CLI commands
- `news-social-intelligence-data-pipeline/src/collectors/mod.rs` — All 29 RSS feed URLs
- `news-social-intelligence-data-pipeline/src/storage/mod.rs` — SQLite schema and operations
- `infrastructure/docker-compose.yml` — Service definitions
- `infrastructure/.env.example` — Required environment variables
- `docs/PIPELINE.md` — Full data flow diagrams

## Conventions

### Rust (src/)
- One module per pipeline phase (collectors, cleaners, labelers, embedders, storage, health)
- Each module is a single `mod.rs` file (no sub-modules yet)
- Structs exported from `lib.rs`: `RssCollector`, `ArticleCleaner`, `KiroLabeler`, `TeiEmbedder`
- Error handling: `anyhow::Result` everywhere, `tracing` for logging
- All operations are async (tokio runtime)
- CLI: simple string match on args (no clap yet)

### Python (social_intel/)
- "Never raises" pattern — all public functions return empty/None on error
- Logging to stderr via `_log()` helper
- Normalized article dict schema across all sources
- ThreadPoolExecutor for parallel fetching
- Lazy encoder loading (SentenceTransformer loaded on first use)

### Environment Variables
- `LABELER_API_KEY` / `LABELER_BASE_URL` / `LABELER_MODEL` — LLM gateway
- `QDRANT_URL` — Vector database
- `TEI_URL` — Embeddings service
- `DB_PATH` — SQLite staging database path
