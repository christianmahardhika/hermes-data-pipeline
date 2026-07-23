# Project Structure

Monorepo with multiple data pipelines sharing common infrastructure.

## Active Pipeline: News & Social Intelligence

```
news-social-intelligence-data-pipeline/
├── Cargo.toml                    # Rust dependencies
├── README.md                     # Pipeline documentation
├── src/                          # Rust RSS Pipeline
│   ├── main.rs                   # CLI entry (run/collect/clean/label/embed/daemon/health/prune/economic)
│   ├── lib.rs                    # Config struct, module exports
│   ├── collectors/mod.rs         # RSS feed fetching (31 sources, retry with backoff, fallback URLs)
│   ├── cleaners/mod.rs           # HTML strip, normalize, SHA256 dedup
│   ├── labelers/mod.rs           # Prof Jiang batch labeling via LLM API
│   ├── embedders/mod.rs          # TEI embeddings + Qdrant ingestion
│   ├── storage/mod.rs            # SQLite operations (raw, cleaned, labeled, ingested)
│   └── health/mod.rs             # Kiromania health check + self-healing
│
├── src/arangodb/                 # ArangoDB Intelligence Store
│   ├── mod.rs                    # ArangoClient (reqwest HTTP, connection pooling)
│   ├── schema.rs                 # SchemaManager (7 doc + 6 edge collections, graph, views)
│   └── ingester.rs               # ArangoIngester (article + graph ingestion, near-dup detection)
│
├── src/economic/                 # Economic Data Collectors
│   ├── mod.rs                    # Module with models + collector registry
│   ├── yahoo_commodities.rs      # 11 commodity symbols (Gold, Oil, CPO, Nickel, Coffee, etc.)
│   ├── coingecko.rs              # Crypto: BTC, ETH, USDT, BNB, XRP
│   ├── fred.rs                   # 6 FRED series (GDP, CPI, etc. — requires FRED_API_KEY)
│   ├── bank_indonesia.rs         # BI Rate, JIBOR, USD/IDR, inflation
│   └── gdelt.rs                  # GDELT event collector (Indonesian events)
│
├── src/social/                   # Social Intelligence (Rust port)
│   ├── mod.rs                    # SocialArticle, SocialStats, Depth enum
│   ├── collector.rs              # Unified collector (TEI 768-dim + Qdrant)
│   ├── dedup.rs                  # Near-duplicate detection via Qdrant
│   ├── relevance.rs              # Token overlap relevance scoring
│   ├── hackernews.rs             # HackerNews via Algolia
│   ├── reddit.rs                 # Reddit via RSS/Atom + quick-xml
│   ├── youtube.rs                # YouTube metadata
│   └── x_twitter.rs              # X/Twitter (disabled)
│
├── src/unlimited/mod.rs          # Unlimited news daemon (Rust, TEI 768-dim)
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
│   ├── data_source.rs            # Yahoo Finance API + mock data
│   └── signal_lookup.rs          # AQL graph queries for ticker ExternalSignals
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

## CI/CD

```
.github/workflows/
└── feed-health.yml               # Daily feed health check (RSS source availability)
```

## Planned Pipelines (Stub only)

```
market-data-pipeline/
├── README.md                     # Design doc for IDX/forex/commodities
├── commodity_collector.py        # Strategic Indonesian commodities (Coal, Palm, Nickel, Gold, Oil)
├── SOCIAL_ECONOMIC_INTELLIGENCE_TOPOLOGY.md  # Full system architecture vision

enhanced-idx-analyst/             # Python IDX Analyst (original, full-featured)
├── idx_ai_analyst_enhanced.py    # Main orchestrator (debate + trade + risk)
├── config.py                     # Portfolio, criteria, personas, thresholds
├── modules/
│   ├── debate_engine.py          # 5-persona bull/bear with detailed reasoning
│   ├── trader_executor.py        # Signal → concrete trade plan
│   ├── risk_manager.py           # Portfolio constraint validation
│   ├── data_gathering.py         # Multi-analyst team data collection
│   ├── memory_logger.py          # Decision logging + reflection
│   ├── output_formatter.py       # RTI Business + Telegram output
│   ├── idx_scraper.py            # IDX curl_cffi scraper
│   ├── notion_integration.py     # Notion portfolio sync
│   └── dividend_sync.py          # Historical dividend records

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
├── docker-compose.yml            # ArangoDB + Qdrant + TEI + optional Redis/PostgreSQL

docs/
├── PIPELINE.md                   # Complete data flow documentation
```

## Key Files to Know

- `news-social-intelligence-data-pipeline/Cargo.toml` — Rust dependencies
- `news-social-intelligence-data-pipeline/src/lib.rs` — Config (env vars, defaults)
- `news-social-intelligence-data-pipeline/src/main.rs` — CLI commands
- `news-social-intelligence-data-pipeline/src/collectors/mod.rs` — All 31 RSS feed URLs
- `news-social-intelligence-data-pipeline/src/arangodb/mod.rs` — ArangoDB client
- `news-social-intelligence-data-pipeline/src/arangodb/schema.rs` — Collection/graph schema
- `news-social-intelligence-data-pipeline/src/economic/mod.rs` — Economic data module
- `news-social-intelligence-data-pipeline/src/storage/mod.rs` — SQLite schema and operations
- `infrastructure/docker-compose.yml` — Service definitions
- `infrastructure/.env.example` — Required environment variables
- `docs/PIPELINE.md` — Full data flow diagrams

## Conventions

### Rust (src/)
- One module per pipeline phase (collectors, cleaners, labelers, embedders, storage, health, arangodb, economic)
- Each module is a single `mod.rs` file (no sub-modules yet) except arangodb/ and economic/
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
- `ARANGO_URL` / `ARANGO_DATABASE` / `ARANGO_USERNAME` / `ARANGO_PASSWORD` — ArangoDB
- `STORAGE_BACKEND` — `"arangodb"` (default) or `"qdrant"` (legacy)
- `QDRANT_URL` — Vector database (legacy fallback)
- `TEI_URL` — Embeddings service
- `DB_PATH` — SQLite staging database path
- `FRED_API_KEY` — Optional: FRED economic data
