# Tech Stack & Build Commands

## RSS News Pipeline (Rust)

- **Language**: Rust 2021 edition
- **Async runtime**: tokio (full features)
- **HTTP client**: reqwest 0.12 (with rustls-tls, JSON)
- **RSS parsing**: feed-rs 2
- **JSON**: serde 1 + serde_json 1
- **Database**: rusqlite 0.32 (SQLite, bundled)
- **Graph DB**: ArangoDB 3.12 (via reqwest HTTP API, no official Rust driver)
- **Vector DB client**: qdrant-client 1 (legacy fallback)
- **Scheduling**: tokio-cron-scheduler 0.13
- **HTML processing**: scraper 0.21, ammonia 4
- **Hashing**: sha2 0.10, hex 0.4
- **Date/Time**: chrono 0.4 (with serde)
- **Logging**: tracing 0.1, tracing-subscriber 0.3 (env-filter)
- **Error handling**: anyhow 1, thiserror 1
- **UUID**: uuid 1 (v4)
- **Retry logic**: backoff 0.4 (tokio feature)
- **Testing**: standard `#[cfg(test)]`, no external test crate yet

## Social Media Intelligence (Python)

- **Runtime**: Python 3.10+
- **Embeddings**: sentence-transformers (all-MiniLM-L6-v2, 384 dim)
- **HTTP**: requests, urllib (stdlib)
- **Data**: numpy
- **YouTube**: yt-dlp (CLI tool, no API key)
- **XML parsing**: xml.etree.ElementTree (stdlib)
- **Concurrency**: concurrent.futures (ThreadPoolExecutor)
- **No dependency manager** (no requirements.txt/pyproject.toml yet)

## Shared Infrastructure (Docker Compose)

- **Graph Database**: ArangoDB 3.12 (port 8529, primary intelligence store)
- **Vector Database**: Qdrant (latest, ports 6333 REST / 6334 gRPC, legacy fallback)
- **Text Embeddings**: HuggingFace TEI (cpu-1.5, multilingual-e5-base, 768 dim, port 8082)
- **LLM Gateway**: Kiromania (OpenAI-compatible, port 9000)
- **Optional**: Redis 7 (port 6379), PostgreSQL 16 (port 5432)
- **Staging DB**: SQLite (local file, `news_staging.db`)

## Environment Variables

### Storage Backend
| Variable | Default | Description |
|----------|---------|-------------|
| `STORAGE_BACKEND` | `arangodb` | Primary store: `"arangodb"` or `"qdrant"` (legacy) |

### ArangoDB
| Variable | Default | Description |
|----------|---------|-------------|
| `ARANGO_URL` | `http://localhost:8529` | ArangoDB HTTP endpoint |
| `ARANGO_DATABASE` | `hermes` | Database name |
| `ARANGO_USERNAME` | `root` | Authentication user |
| `ARANGO_PASSWORD` | — | Authentication password |

### Existing
| Variable | Default | Description |
|----------|---------|-------------|
| `LABELER_API_KEY` | — | LLM gateway API key |
| `LABELER_BASE_URL` | — | LLM gateway URL |
| `LABELER_MODEL` | — | LLM model name |
| `QDRANT_URL` | `http://localhost:6333` | Qdrant REST endpoint |
| `TEI_URL` | `http://localhost:8082` | TEI embeddings endpoint |
| `DB_PATH` | `news_staging.db` | SQLite staging database |
| `FRED_API_KEY` | — | Optional: FRED economic data API key |

## Common Commands

### Infrastructure
```bash
cd infrastructure
docker compose up -d                  # Start ArangoDB + Qdrant + TEI
docker compose --profile full up -d   # Start all (+ Redis + PostgreSQL)
docker compose down                   # Stop all
```

### Rust Pipeline (news-social-intelligence-data-pipeline)
```bash
cd news-social-intelligence-data-pipeline

# Build
cargo build --release

# Run full pipeline
cargo run --release -- run

# Individual phases
cargo run --release -- collect       # Fetch RSS feeds (31 sources)
cargo run --release -- clean         # Strip HTML, normalize
cargo run --release -- label         # Prof Jiang game theory (via LLM)
cargo run --release -- embed         # TEI embeddings → ArangoDB/Qdrant

# Economic data collection
cargo run --release -- economic all        # All economic collectors
cargo run --release -- economic commodity  # Yahoo commodities (9 symbols)
cargo run --release -- economic crypto     # CoinGecko (BTC, ETH, USDT, BNB, XRP)
cargo run --release -- economic fred       # FRED macro data (requires FRED_API_KEY)
cargo run --release -- economic bi         # Bank Indonesia rates

# Daemon mode (every 15 min)
cargo run --release -- daemon

# Health check
cargo run --release -- health

# Prune ingested records
cargo run --release -- prune

# Testing
cargo test -- --test-threads=1       # Run all tests (env-var tests need serial)
cargo test -- --nocapture            # With output
cargo clippy                         # Linting
cargo fmt                            # Format
```

### Python Social Intelligence
```bash
cd news-social-intelligence-data-pipeline

# Run social media collector (cron)
python social_intel_cron.py

# Custom topics
python social_intel_cron.py --topics "AI,geopolitics,business" --depth quick

# Individual modules
python -m social_intel.collector --query "AI" --depth default
python -m social_intel.collector --front-page

# Near-duplicate detection test
python -m social_intel.near_duplicate
```

### ArangoDB Queries
```bash
# Check ArangoDB health
curl http://localhost:8529/_api/version

# List collections
curl http://localhost:8529/_db/hermes/_api/collection

# AQL query example (articles by actor)
curl -X POST http://localhost:8529/_db/hermes/_api/cursor \
  -H "Content-Type: application/json" \
  -d '{"query": "FOR doc IN articles FILTER doc.actors ANY == @actor RETURN doc", "bindVars": {"actor": "Bank Indonesia"}}'
```

### Qdrant Queries (Legacy)
```bash
# Check collections
curl http://localhost:6333/collections

# Scroll points
curl -s "http://localhost:6333/collections/news_articles/points/scroll" \
  -H "Content-Type: application/json" \
  -d '{"limit": 5, "with_payload": true}' | jq

# Search
curl -s "http://localhost:6333/collections/news_articles/points/search" \
  -H "Content-Type: application/json" \
  -d '{"vector": [...], "limit": 10, "with_payload": true}' | jq
```

### TEI Health
```bash
curl http://localhost:8082/health
curl http://localhost:8082/embed -X POST \
  -H "Content-Type: application/json" \
  -d '{"inputs": ["test text"]}'
```
