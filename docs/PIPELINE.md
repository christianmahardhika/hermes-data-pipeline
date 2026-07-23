# Data Pipeline Documentation

Complete data flow documentation for all Hermes data pipelines.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        HERMES DATA PIPELINE (Current State)                      │
└─────────────────────────────────────────────────────────────────────────────────┘

╔═══════════════════════════════════════════════════════════════════════════════════╗
║                           DATA SOURCES (Ingestion)                               ║
╚═══════════════════════════════════════════════════════════════════════════════════╝

┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  31 RSS Feeds   │  │  Social Media   │  │ Economic Data   │  │   GDELT Events  │
│  (every 15min)  │  │  (every 2hr)    │  │  (every 1hr)    │  │   (every 2hr)   │
├─────────────────┤  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤
│ • 11 ID Nasional│  │ • HackerNews    │  │ • Yahoo Finance │  │ • Indonesia     │
│ • 6 Intl Bisnis │  │ • Reddit        │  │   (11 commodity)│  │   events from   │
│ • 4 Intl General│  │ • YouTube       │  │ • CoinGecko     │  │   global media  │
│ • 4 Asia Pacific│  │ • X/Twitter(off)│  │   (5 crypto)    │  │                 │
│ • 6 CNBC ID sub │  │                 │  │ • FRED (6 macro)│  │                 │
│                 │  │                 │  │ • Bank Indonesia│  │                 │
│                 │  │                 │  │   (4 indicators)│  │                 │
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                    │                    │                     │
         ▼                    ▼                    ▼                     ▼
╔═══════════════════════════════════════════════════════════════════════════════════╗
║                        PROCESSING PIPELINE (Rust)                                ║
╚═══════════════════════════════════════════════════════════════════════════════════╝

┌─────────────────────────────────────────────────────────────────────────────────┐
│ Phase 1: COLLECT                                                                 │
│ ┌─────────────────────────────────────────────────────────────────────────────┐ │
│ │ • Fetch RSS XML/Atom                                                        │ │
│ │ • Circuit breaker per feed (CLOSED → OPEN → HALF-OPEN)                      │ │
│ │ • Retry with exponential backoff                                            │ │
│ │ • Fallback URLs for multi-endpoint sources                                  │ │
│ │ • Feed health tracking (freshness, consecutive failures)                    │ │
│ │ • Category-based stats (Indonesian, Intl Business, Intl General, Asia Pac)  │ │
│ │ → Store raw in SQLite staging (status: "raw")                               │ │
│ └─────────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Phase 2: CLEAN                                                                   │
│ ┌─────────────────────────────────────────────────────────────────────────────┐ │
│ │ • Strip HTML (ammonia sanitizer)                                            │ │
│ │ • Normalize whitespace, encoding                                            │ │
│ │ • SHA256 content hash for dedup                                             │ │
│ │ • Extract: title, description, pub_date, source                             │ │
│ │ → Update SQLite status: "raw" → "cleaned"                                  │ │
│ └─────────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Phase 3: LABEL (Prof Jiang Game Theory via LLM)                                  │
│ ┌─────────────────────────────────────────────────────────────────────────────┐ │
│ │ • Send to Kiromania LLM gateway (OpenAI-compatible)                         │ │
│ │ • Extract: actors, events, relations, context                               │ │
│ │ • Pattern matching: trade_war, currency_crisis, policy_shift, etc.          │ │
│ │ • Investment signal: bullish/bearish/neutral + confidence                   │ │
│ │ • Batch processing (5-20 articles per LLM call)                             │ │
│ │ → Update SQLite status: "cleaned" → "labeled"                              │ │
│ └─────────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Phase 4: EMBED & STORE                                                           │
│ ┌─────────────────────────────────────────────────────────────────────────────┐ │
│ │ • Generate 768-dim embeddings via TEI (multilingual-e5-base)                │ │
│ │ • Near-duplicate detection (cosine > 0.95 = skip)                           │ │
│ │ • Store in ArangoDB:                                                        │ │
│ │   - articles (doc) + vector embedding                                       │ │
│ │   - actors (doc) from Prof Jiang extraction                                 │ │
│ │   - topics (doc) from event classification                                  │ │
│ │   - article_mentions_actor (edge)                                           │ │
│ │   - article_has_topic (edge)                                                │ │
│ │   - actor_relates_actor (edge)                                              │ │
│ │   - article_similar (edge) for near-dups                                    │ │
│ │ → Update SQLite status: "labeled" → "ingested"                             │ │
│ └─────────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────┘

╔═══════════════════════════════════════════════════════════════════════════════════╗
║                            STORAGE LAYER                                         ║
╚═══════════════════════════════════════════════════════════════════════════════════╝

┌────────────────────────┐  ┌────────────────────────┐  ┌────────────────────────┐
│    SQLite Staging      │  │  ArangoDB (Primary)    │  │    Qdrant (Legacy)     │
│    news_staging.db     │  │   localhost:8529       │  │   localhost:6333       │
├────────────────────────┤  ├────────────────────────┤  ├────────────────────────┤
│ • raw_feeds            │  │ Documents:             │  │ • news_articles (768d) │
│ • feed_health          │  │ • articles (+ vec)     │  │ • social_intel (768d)  │
│ • circuit breaker      │  │ • actors               │  │ • unlimited_*  (768d)  │
│   state                │  │ • topics               │  │ • pagupon-kb   (768d)  │
│                        │  │ • economic_indicators  │  │ • pondo-biz-kb (768d)  │
│ Status flow:           │  │ • signals              │  │                        │
│ raw → cleaned →        │  │ • social_posts         │  │ Fallback when          │
│ labeled → ingested     │  │ • events (GDELT)       │  │ STORAGE_BACKEND        │
│                        │  │                        │  │ = "qdrant"             │
│                        │  │ Edges:                 │  │                        │
│                        │  │ • article_mentions_    │  │                        │
│                        │  │   actor                │  │                        │
│                        │  │ • article_has_topic    │  │                        │
│                        │  │ • actor_relates_actor  │  │                        │
│                        │  │ • signal_source        │  │                        │
│                        │  │ • article_similar      │  │                        │
│                        │  │ • topic_correlates     │  │                        │
│                        │  │ • impacts (GDELT)      │  │                        │
│                        │  │                        │  │                        │
│                        │  │ Graph:                 │  │                        │
│                        │  │ • intelligence_graph   │  │                        │
│                        │  │                        │  │                        │
│                        │  │ Search View:           │  │                        │
│                        │  │ • articles_search      │  │                        │
│                        │  │   (BM25 full-text)     │  │                        │
└────────────────────────┘  └────────────────────────┘  └────────────────────────┘

╔═══════════════════════════════════════════════════════════════════════════════════╗
║                        CONSUMERS (Downstream)                                    ║
╚═══════════════════════════════════════════════════════════════════════════════════╝

┌────────────────────────┐  ┌────────────────────────┐  ┌────────────────────────┐
│   IDX Analyst (Rust)   │  │  Hermes AI Agents      │  │   Pondo Ngopi (Future) │
├────────────────────────┤  ├────────────────────────┤  ├────────────────────────┤
│ • 5-persona debate     │  │ • RAG queries via      │  │ • Coffee Arabica/      │
│ • Signal lookup via    │  │   ArangoSearch +       │  │   Robusta tracking     │
│   AQL graph queries    │  │   vector similarity    │  │ • Oil/PET packaging    │
│ • Trade proposals      │  │ • News correlation     │  │   cost correlation     │
│ • Portfolio risk       │  │ • Social sentiment     │  │ • USD/IDR import cost  │
│ • Yahoo Finance        │  │                        │  │                        │
│   (live stock price)   │  │                        │  │                        │
└────────────────────────┘  └────────────────────────┘  └────────────────────────┘

╔═══════════════════════════════════════════════════════════════════════════════════╗
║                    CRON JOBS (Pagupon Finance Scripts)                            ║
╚═══════════════════════════════════════════════════════════════════════════════════╝

┌────────────────────────┐  ┌────────────────────────┐  ┌────────────────────────┐
│ Portfolio Analysis     │  │ Market Intelligence    │  │ Specific Monitoring    │
├────────────────────────┤  ├────────────────────────┤  ├────────────────────────┤
│ • portfolio_sentiment  │  │ • daily_digest         │  │ • bumn_export_monitor  │
│ • portfolio_fundament  │  │ • daily_digest_fin_int │  │ • stock_alerts_bumn    │
│ • portfolio_retirement │  │ • market_summary       │  │ • inco_deepdive        │
│ • sospol_accumulation  │  │ • screener-fetch       │  │ • inco_fundamental     │
│                        │  │ • screener-digest      │  │ • inco_sentiment       │
│                        │  │ • idx_ai_analyst       │  │ • fundamental_check    │
└────────────────────────┘  └────────────────────────┘  └────────────────────────┘
Note: Standalone scripts — not yet integrated with ArangoDB graph.
Uses: openai, yfinance, requests+bs4, hermes mcp call

╔═══════════════════════════════════════════════════════════════════════════════════╗
║                        INFRASTRUCTURE (Docker Compose)                            ║
╚═══════════════════════════════════════════════════════════════════════════════════╝

┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐
│ ArangoDB   │ │   Qdrant   │ │    TEI     │ │ Kiromania  │ │  Redis     │
│  :8529     │ │ :6333/6334 │ │   :8082    │ │   :9000    │ │  :6379     │
│            │ │            │ │ multilingual│ │ LLM Gateway│ │ (optional) │
│ Graph +    │ │ Vector     │ │ -e5-base   │ │ OpenAI API │ │            │
│ Document + │ │ search     │ │ 768-dim    │ │ compatible │ │            │
│ Search     │ │ (legacy)   │ │            │ │            │ │            │
└────────────┘ └────────────┘ └────────────┘ └────────────┘ └────────────┘
```

---

## Data Sources Detail

### RSS News Feeds (31 sources)

| Category | Count | Sources |
|----------|-------|---------|
| Indonesian National | 11 | Tempo, CNN Indonesia, Antara, Republika, Detik News, Detik Finance, Kompas, Okezone, Sindonews, Kontan, CNBC Indonesia |
| International Business | 6 | BBC Business, BBC World, CNBC, Bloomberg, Financial Times, MarketWatch |
| International General | 4 | Al Jazeera, The Guardian, NPR, AP News |
| Asia Pacific | 4 | Channel News Asia, Nikkei Asia, SCMP, Straits Times |
| CNBC ID Sub-feeds | 6 | News, Market, Tech, Entrepreneur + others |

### Economic Data Sources

| Source | Symbols/Indicators | Auth | Schedule |
|--------|-------------------|------|----------|
| Yahoo Finance | 11 commodities: Gold, Oil WTI, Brent, CPO, Silver, Copper, Natural Gas, Nickel, Aluminum, **Coffee Arabica**, **Coffee Robusta** | No | Hourly |
| CoinGecko | 5 crypto: BTC, ETH, USDT, BNB, XRP | No | Hourly |
| FRED (St. Louis Fed) | 6 series: GDP, CPI, Unemployment, Fed Funds, 10Y Treasury, USD Index | FRED_API_KEY | Hourly |
| Bank Indonesia | 4: BI Rate, JIBOR, USD/IDR, Inflation YoY | No | Hourly |
| GDELT | Indonesian events from global media | No | Every 2hr |

### Social Media Sources

| Source | Auth | Method |
|--------|------|--------|
| HackerNews | No | Algolia API |
| Reddit | No | RSS/Atom feeds |
| YouTube | No | yt-dlp metadata |
| X/Twitter | Yes (disabled) | xurl CLI |

---

## Intelligence Correlation

```
News (Prof Jiang)  ──→  actors  ──→  IDX Analyst (sector mapping)
                         │
Social sentiment   ──→  article_similar  (same-event, 0.85 threshold)
                         │
Commodity prices   ──→  signal_source  ──→  ExternalSignal
                         │
Economic indicators ──→  signal_source  ──→  Portfolio risk
                         │
GDELT events       ──→  impacts  ──→  market_context
```

**Correlation chain for Pondo Ngopi (future):**
```
Coffee futures (KC=F, RC=F)  → raw material COGS
Crude Oil (CL=F, BZ=F)      → PET packaging cost (polyethylene terephthalate)
USD/IDR (Bank Indonesia)     → import cost multiplier
```

---

## Scheduling Summary

| Pipeline | Interval | Mode | Command |
|----------|----------|------|---------|
| RSS News (full cycle) | 15 min | Daemon | `cargo run -- daemon` |
| Economic (all) | 1 hour | CLI/Cron | `cargo run -- economic all` |
| Social Media | 2 hours | Cron | `python social_intel_cron.py` |
| GDELT Events | 2 hours | Part of economic all | — |
| Feed Health Check | Daily | GitHub Actions | `.github/workflows/feed-health.yml` |
| Screener Fetch | 08:45 & 14:45 WIB | Cron | `screener-fetch-cron.sh` |
| Screener Digest | 09:00 & 15:00 WIB | Cron | `screener-digest-cron.sh` |
| IDX AI Analyst (digest) | 09:00 & 15:00 WIB | Cron | `cargo run -- idx-analyst digest` |
| IDX AI Analyst (legacy) | Daily | Cron | `idx_ai_analyst_enhanced.sh` |

---

## Cron Job Scripts (Pagupon Finance)

Standalone scripts for portfolio intelligence. Located in `scripts/`.

### Portfolio Analysis
| Script | Function | Schedule |
|--------|----------|----------|
| `portfolio_sentiment_analysis.py` | Sentiment per sector (11 holdings) | On-demand |
| `portfolio_fundamental_scraper.py` | IDX fundamental data scraping | On-demand |
| `portfolio_retirement_assessment.py` | Portfolio vs retirement target | On-demand |
| `sospol_accumulation_digest.py` | Sospol-lab → accumulation buy signals | On-demand |

### Market Intelligence
| Script | Function | Schedule |
|--------|----------|----------|
| `daily_digest.py` | Daily news digest | Daily |
| `daily_digest_financial_intelligence.py` | EPUB chapter digest (book learning) | Daily 17:00 WIB |
| `market_summary.py` | IHSG market summary + top movers | Daily 16:00 WIB |
| `screener-fetch-cron.sh` | Fetch 14 tickers market data | 08:45 & 14:45 WIB |
| `screener-digest-cron.sh` | Generate screener digest | 09:00 & 15:00 WIB |
| `idx_ai_analyst_enhanced.sh` | 5-persona debate analysis | Daily |

### Specific Monitoring
| Script | Function | Trigger |
|--------|----------|---------|
| `bumn_export_monitor.py` | PP BUMN Ekspor Prabowo news | On-demand |
| `stock_alerts_bumn_export.py` | Alert BUMN beneficiaries (PTBA, ANTM) | On-demand |
| `inco_deepdive.py` | INCO deep fundamental | On-demand |
| `inco_full_fundamental.py` | INCO full fundamental report | On-demand |
| `inco_sentiment_analysis.py` | INCO sentiment + news correlation | On-demand |
| `fundamental_analysis_check.py` | Quick fundamental check | On-demand |

### Portfolio Tickers Tracked
```
KLBF, TLKM, BBRI, PTBA, BJTM, ADMF, TAPG, JPFA, TSPC, BMRI, ASII, ULTJ, HMSP, MNCN
```

---

## Quick Start

```bash
# 1. Start infrastructure
cd infrastructure
docker compose up -d              # ArangoDB + Qdrant + TEI

# 2. Run RSS News pipeline (Rust)
cd ../news-social-intelligence-data-pipeline
cargo run --release -- daemon     # Runs every 15 min

# 3. Run Economic data collection
cargo run --release -- economic all

# 4. Run Social Media pipeline (Python)
python social_intel_cron.py

# 5. Run individual economic collectors
cargo run --release -- economic commodity  # Yahoo (11 symbols incl. coffee)
cargo run --release -- economic crypto     # CoinGecko
cargo run --release -- economic fred       # FRED (needs FRED_API_KEY)
cargo run --release -- economic bi         # Bank Indonesia

# 6. IDX Analyst (5-persona debate)
cargo run --release -- idx-analyst --portfolio --mock   # All tickers, mock data
cargo run --release -- idx-analyst BMRI BBRI            # Specific tickers, live Yahoo
cargo run --release -- idx-analyst digest               # Full portfolio digest (cron mode)
cargo run --release -- idx-analyst digest --mock        # Digest with mock data

# 7. Health check
cargo run --release -- health

# 7. Query ArangoDB
curl -X POST http://localhost:8529/_db/hermes/_api/cursor \
  -H "Content-Type: application/json" \
  -d '{"query": "FOR doc IN articles SORT doc.published_at DESC LIMIT 5 RETURN doc"}'
```

---

## Service Dependencies

```
Required:
├── ArangoDB :8529  → News + Economic + Intelligence Fusion (primary)
├── TEI :8082       → All pipelines (embeddings, 768-dim)
├── Kiromania :9000 → RSS News only (Prof Jiang labeling)
├── SQLite (local)  → RSS News staging (news_staging.db)
│
Optional:
├── Qdrant :6333    → Social Media + legacy news (fallback)
└── Redis :6379     → Future caching layer
```
