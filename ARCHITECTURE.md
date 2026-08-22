# Architecture — Current State (2026-08-22)

**System:** Hermes Data Pipeline  
**Status:** Alpha — Active Development  
**Profile:** social-politic-lab  

## System Context

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Hermes Data Pipeline (Alpha)                                               │
│                                                                             │
│  ┌─────────────┐  ┌─────────────────────┐  ┌─────────────────────────────┐ │
│  │  Rust CLI   │  │  Python Scripts     │  │  Hermes Agent               │ │
│  │  (partial)  │  │  (partial)          │  │  (fully operational)        │ │
│  └──────┬──────┘  └──────────┬──────────┘  └──────────────┬──────────────┘ │
│         │                    │                            │                 │
│         └────────┬───────────┘                            │                 │
│                  ▼                                       ▼                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Docker Infrastructure                                              │   │
│  │  • ArangoDB (graph + docs) ✅                                      │   │
│  │  • Qdrant (vectors) ✅                                            │   │
│  │  • TEI (embeddings) ✅                                            │   │
│  │  • Redis (cache) ❌                                               │   │
│  │  • Kiromania (LLM gateway) ❌                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Next.js Dashboard (separate repo)                                 │   │
│  │  • Real-time portfolio: Rp 13.86M ✅                               │   │
│  │  • Monitoring API: 6 sources ✅                                    │   │
│  │  • Market widgets: ❌                                              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Working Components

### 1. IDX Analyst (Rust)
- **File:** `news-social-intelligence-data-pipeline/src/main.rs`
- **Command:** `cargo run --release -- idx-analyst BMRI BBRI`
- **Status:** Functional with real data from backend API
- **Features:**
  - 5-persona debate engine (Bull, Bear, Analyst, Skeptic, Insider)
  - Yahoo Finance integration (fallback to mock data)
  - Telegram-formatted output or full RTI report
  - Real Indonesian stock data (BMRI, BBRI, INCO, ANTM, PTBA, TAPG)
- **LLM:** Kiromania gateway (external, via network call)

### 2. Economic Data Pipeline (Rust)
- **Command:** `cargo run --release -- economic`
- **Status:** Functional with real data
- **Features:**
  - Portfolio digest generation
  - BPS (Indonesian government statistics) integration
  - Real-time portfolio value calculation (Rp 13.86M)
  - ArangoDB storage for market data

### 3. Unlimited Indonesian News (Rust)
- **Command:** `cargo run --release -- unlimited`
- **Status:** Functional with real collection
- **Features:**
  - Indonesian news collection from multiple sources
  - Prof Jiang knowledge base integration
  - MCP tool integration for querying

### 4. Infrastructure (Docker)
- **File:** `infrastructure/docker-compose.yml`
- **Status:** ArangoDB, Qdrant, TEI start successfully
- **Services:**
  - `arangodb:3.12` - Document and graph database ✅
  - `qdrant/qdrant:latest` - Vector database ✅
  - `ghcr.io/huggingface/text-embeddings-inference:latest` - Embeddings ✅
  - `redis:7-alpine` - ❌ Not configured
  - `kiromania` - ❌ Separate service, not included

### 5. Hermes Agent Integration
- **Profile:** `social-politic-lab`
- **Status:** Fully operational with 7 active cron jobs
- **Jobs:**
  1. Advanced Portfolio Intelligence System (30-min)
  2. Advanced Social Intelligence Collection (2-hour)
  3. Social-Economic Correlation Analysis (4-hour)
  4. Daily Curated Tech Collection (daily at 8:00)
  5. arangodb-news-monitor (daily at midnight)
  6. BI Currency Rate Collection (every 8 hours)
  7. Agent-Reach Fixed Monitor (30-min)
- **Delivery:** Telegram topics with automated reporting

### 6. Dashboard API
- **URL:** `http://localhost:3002` (Next.js)
- **Tailscale:** `http://100.70.96.84:3002`
- **Endpoints:**
  - `/api/monitoring/sources` - Health status of 6 data sources ✅
  - `/api/portfolio` - Real portfolio value and stock data ✅
  - `/api/news-summary` - Global and domestic news summaries ✅
- **Backend:** Rust intelligence system on `localhost:8888`

## Broken Components

### 1. RSS Pipeline (Rust) — STUBBED
- **Problem:** Command stubs exist but implementations are incomplete
- **Commands affected:** `collect`, `clean`, `label`, `embed`, `daemon`
- **Root Cause:** Migration from SQLite to ArangoDB left modules in limbo
- **Current State:**
  ```rust
  // In main.rs - function stubs exist
  async fn run_collect(config: &Config) -> Result<()> {
      info!("Collecting RSS feeds...");
      // TODO: Implement
      Ok(())
  }
  ```

### 2. Market Data Collector (Python) — HARDCODED
- **File:** `scripts/hermes-config/commodity_collector.py`
- **Problem:** Returns hardcoded commodity prices instead of live data
- **Impact:** No real market data collection
- **Root Cause:** yfinance integration planned but not implemented
- **Current State:**
  ```python
  # Hardcoded data instead of yfinance API
  commodity_prices = {
      "LME_Nickel": 18000.0,  # USD/ton - hardcoded
      "Coal": 120.0,          # USD/ton - hardcoded
      "CPO": 900.0,           # USD/ton - hardcoded
  }
  ```

### 3. BI Currency Scraper — MISSING DEPENDENCIES
- **File:** `scripts/hermes-config/bi_currency_scraper.py`
- **Problem:** Requires `beautifulsoup4` and `lxml` packages
- **Impact:** Cron job fails with `ModuleNotFoundError`
- **Fix:** `pip install beautifulsoup4 lxml`

### 4. Qdrant Integration — NOT IMPLEMENTED
- **Problem:** No vector storage for embeddings
- **Impact:** Semantic search and RAG not possible
- **Root Cause:** TEI is running but no code writes to Qdrant

## Data Flow (Current vs Target)

### Current Data Flow
```
Indonesian Stocks → Rust Backend (:8888) → ArangoDB → Dashboard API → Next.js (:3002)
     │
     └──→ Hermes Agent (cron jobs) → Telegram Delivery
```

### Target Data Flow (Not Yet Implemented)
```
RSS Feeds → Collector → Cleaner → Labeler → Embedder → Qdrant
                                              ↓
                                         ArangoDB (metadata)
                                              ↓
                                       Rust API (:8888)
                                              ↓
                                    Next.js Dashboard (:3000)
```

## Storage Architecture

### Current Storage
- **ArangoDB:** `hermes_intelligence` database with collections:
  - `indonesian_stocks` - Real-time stock data (BMRI, BBRI, INCO, etc.)
  - `portfolio_data` - Portfolio tracking and correlations
  - `news_articles` - Indonesian news articles (limited)
  - `prof_jiang_kb` - Knowledge base chunks (130 items)
- **Local Files:** JSON files in `scripts/hermes-config/` (generated data)
- **Prof Jiang KB:** 130 chunks across 3 categories (geostrategy, game theory, secret history)

### Missing Storage
- **Qdrant:** No vector collections yet
- **Redis:** No caching layer for rate limiting
- **SQLite:** No development staging database

## Technology Choices (Current)

| Component | Choice | Version | Status |
|-----------|--------|---------|--------|
| **Core Language** | Rust | 1.80+ | ✅ Working (partial) |
| **Script Language** | Python | 3.11+ | ✅ Working (partial) |
| **Frontend** | Next.js | 16+ | ✅ Working (separate repo) |
| **Vector DB** | Qdrant | 1.11+ | ✅ Infrastructure only |
| **Document DB** | ArangoDB | 3.12+ | ✅ Fully operational |
| **Embeddings** | TEI | 1.5+ | ✅ Infrastructure only |
| **LLM Gateway** | Kiromania | custom | ✅ External service |
| **Container** | Docker | 24+ | ✅ Fully operational |
| **Agent Platform** | Hermes Agent | latest | ✅ Fully operational |

## Known Technical Debt

### 1. Storage Abstraction Missing
- **Problem:** `storage.rs` is a stub with no implementations
- **Impact:** Cannot switch between SQLite (dev) and ArangoDB (prod)
- **Fix:** Implement `Storage` trait with SQLite and ArangoDB backends

### 2. No Tests
- **Problem:** 0% test coverage across all languages
- **Impact:** Changes risk breaking existing functionality
- **Fix:** Add unit tests for Rust modules and Python scripts

### 3. No CI/CD
- **Problem:** No automated checks on push/PR
- **Impact:** Code quality and integration issues not caught early
- **Fix:** Add GitHub Actions for Rust + Python testing

### 4. Hardcoded Values
- **Problem:** Paths, prices, API endpoints hardcoded
- **Examples:**
  - Commodity prices in Python (`commodity_collector.py`)
  - Local paths in Rust (`/home/ctianm/` in health checks)
- **Fix:** Environment variables and configuration files

### 5. No Observability
- **Problem:** Logs only, no metrics or tracing
- **Impact:** Cannot monitor performance or debug issues
- **Fix:** Add Prometheus metrics and OpenTelemetry tracing

## Performance Characteristics

### Current Performance
- **Rust Backend:** Responds in < 100ms for stock data
- **ArangoDB Queries:** < 50ms for portfolio data
- **Dashboard API:** < 200ms for monitoring endpoints
- **Hermes Cron Jobs:** Run reliably every 30 min to 8 hours

### Bottlenecks
1. **Python Scripts:** No async I/O, sequential execution
2. **No Caching:** Repeated API calls to same endpoints
3. **No Batching:** Individual LLM calls instead of batch processing

## Security Considerations

### Current Security
- **✅ Secrets Management:** `.env` files not committed to git
- **✅ Network Isolation:** Docker containers on internal network
- **✅ Access Control:** Tailscale for secure remote access
- **❌ No Rate Limiting:** API endpoints have no rate limits
- **❌ No Authentication:** Dashboard API has no auth
- **❌ No Encryption:** No TLS/SSL for internal communications

### Recommendations
1. Add API key authentication for dashboard endpoints
2. Implement rate limiting with Redis
3. Add TLS certificates for production deployment
4. Regular security audits of dependencies

## Recovery Procedures

### Database Recovery
```bash
# ArangoDB backup
docker exec hermes-arangodb arangodump --output-directory /backup

# Qdrant snapshot (when implemented)
curl -X POST http://localhost:6333/snapshots
```

### Pipeline Recovery
```bash
# Check Hermes cron jobs
hermes cron list

# Restart failed jobs
hermes cron run --job-id JOB_ID

# Restart infrastructure
cd infrastructure && docker compose restart
```

### Data Validation
```bash
# Check portfolio data
curl http://localhost:3002/api/portfolio

# Check monitoring sources
curl http://localhost:3002/api/monitoring/sources

# Check ArangoDB health
docker exec hermes-arangodb arangosh --server.endpoint tcp://arangodb:8529 --server.database hermes_intelligence --eval "db._query('RETURN 1')"
```

## Development Workflow

### Local Development
```bash
# Start infrastructure
cd infrastructure && docker compose up -d

# Build Rust CLI
cd ../news-social-intelligence-data-pipeline
cargo build --release

# Test commands
cargo run --release -- idx-analyst BMRI BBRI
cargo run --release -- economic

# Run Python scripts (fix dependencies first)
pip install beautifulsoup4 lxml requests pandas
python scripts/hermes-config/commodity_collector.py --test
```

### Testing Strategy (To Be Implemented)
1. **Unit Tests:** Rust modules, Python functions
2. **Integration Tests:** Docker compose with test data
3. **End-to-End Tests:** Full pipeline with mock sources
4. **Performance Tests:** Load testing with realistic data

## Monitoring & Alerting (To Be Implemented)

### Required Metrics
- Articles collected per hour
- LLM API call latency and success rate
- Database query performance
- Pipeline stage completion times
- Error rates per component

### Alerting Rules
- No articles collected in 2 hours
- LLM error rate > 10%
- Database connection failures
- High latency (> 1s) in critical paths

## Evolution Plan

### Phase 1: Foundation (Week 1-2)
- ✅ Document current state (AUDIT.md, ARCHITECTURE.md)
- ✅ Create honest roadmap (ROADMAP.md)
- [ ] Fix market data collector (yfinance integration)
- [ ] Add basic testing framework

### Phase 2: Core Pipeline (Week 3-5)
- [ ] Implement RSS collector (Rust)
- [ ] Implement HTML cleaner (Rust)
- [ ] Implement LLM labeler (Rust)
- [ ] Implement TEI embedder (Rust)
- [ ] Add Qdrant integration

### Phase 3: Production (Week 6-8)
- [ ] Add monitoring (Prometheus + Grafana)
- [ ] Implement backup strategy
- [ ] Add security hardening
- [ ] Performance optimization

### Phase 4: Features (Post-v1.0)
- [ ] Social media pipeline
- [ ] Knowledge ingestion (PDF/EPUB)
- [ ] Advanced analytics (graph queries, predictions)
- [ ] Mobile application

---

**Document Version:** 1.0  
**Last Updated:** 2026-08-22  
**Next Review:** After Week 2 completion  
**Maintainer:** christianmahardhika