# Data Pipeline Documentation

Complete data flow documentation for all Hermes data pipelines.

## Overview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           HERMES DATA PIPELINE                                │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐     │
│   │    RSS      │   │   Social    │   │  Economic   │   │  Knowledge  │     │
│   │    News     │   │    Media    │   │    Data     │   │  Ingestion  │     │
│   │   (Rust)    │   │  (Python)   │   │   (Rust)    │   │  (Python)   │     │
│   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘     │
│          │                 │                 │                 │            │
│          ▼                 ▼                 ▼                 ▼            │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    SHARED INFRASTRUCTURE                            │   │
│   │  ┌──────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │   │
│   │  │ ArangoDB │ │ Qdrant  │ │   TEI   │ │Kiromania│ │ SQLite  │     │   │
│   │  │  :8529   │ │  :6333  │ │  :8082  │ │  :9000  │ │ (local) │     │   │
│   │  └──────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. News and Social Intelligence Pipeline

### 1.1 RSS News Flow (Rust)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         RSS NEWS PIPELINE (Rust)                            │
└─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────┐
  │                           DATA SOURCES (31 feeds)                        │
  │                                                                          │
  │  Indonesian National (11)         International Business (6)             │
  │  ├── Tempo                        ├── BBC Business                       │
  │  ├── CNN Indonesia                ├── BBC World                          │
  │  ├── Antara                       ├── CNBC                               │
  │  ├── Republika                    ├── Bloomberg                          │
  │  ├── Detik News (+ fallback)      ├── Financial Times                    │
  │  ├── Detik Finance (+ fallback)   └── MarketWatch                        │
  │  ├── Kompas                                                              │
  │  ├── Okezone                      International General (4)              │
  │  ├── Sindonews                    ├── Al Jazeera                         │
  │  ├── Kontan                       ├── The Guardian                       │
  │  └── CNBC Indonesia               ├── NPR                               │
  │                                   └── AP News                            │
  │  Asia Pacific (4)                                                        │
  │  ├── Channel News Asia            + 6 additional feeds (circuit breaker  │
  │  ├── Nikkei Asia                    managed, category-based resilience)  │
  │  ├── South China Morning Post                                            │
  │  └── The Straits Times                                                   │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 1: COLLECT (with Circuit Breaker)                                 │
  │  ──────────────────────────────────────                                  │
  │  • Fetch RSS feeds (31 sources, with fallback URLs)                       │
  │  • Circuit breaker: CLOSED -> OPEN -> HALF-OPEN state machine            │
  │  • FeedCategory enum: per-category stats and freshness tracking          │
  │  • Stale source detection (no new articles threshold)                    │
  │  • Parse XML/Atom                                                        │
  │  • Extract: title, content, url, published_at                            │
  │  • Dedupe by URL hash                                                    │
  │                                                                          │
  │  Output: raw_feeds table (SQLite)                                        │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 2: CLEAN                                                          │
  │  ─────────────────                                                       │
  │  • Strip HTML tags                                                       │
  │  • Normalize whitespace                                                  │
  │  • Truncate content (500 chars for labeling)                             │
  │  • Validate required fields                                              │
  │                                                                          │
  │  Output: cleaned table (SQLite)                                          │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 3: LABEL (Prof Jiang Game Theory)                                 │
  │  ───────────────────────────────────────                                 │
  │  • Batch 20 articles per LLM call                                        │
  │  • Send to Kiromania (Claude Sonnet)                                     │
  │  • Extract: actors, events, relations, context, pattern_match,           │
  │    investment_signal                                                      │
  │                                                                          │
  │  Output: labeled table (SQLite)                                          │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 4: INGEST -> ArangoDB (Primary) or Qdrant (Legacy)               │
  │  ───────────────────────────────────────────────────────                 │
  │                                                                          │
  │  ArangoDB path (STORAGE_BACKEND=arangodb):                               │
  │  • Near-duplicate detection (content hash + AQL similarity)              │
  │  • Insert article document                                               │
  │  • Create actor/topic vertices                                           │
  │  • Create edges: article_mentions_actor, article_has_topic               │
  │  • Generate embeddings via TEI (768-dim)                                 │
  │  • Store vector in article document                                      │
  │                                                                          │
  │  Qdrant path (STORAGE_BACKEND=qdrant):                                   │
  │  • Generate embeddings via TEI (multilingual-e5-base, 768-dim)           │
  │  • Store to Qdrant: news_articles collection                             │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
         ┌─────────────────────┐        ┌─────────────────────┐
         │     ArangoDB        │        │      Qdrant         │
         │  articles + graph   │        │  news_articles      │
         │  (primary)          │        │  (legacy fallback)  │
         └─────────────────────┘        └─────────────────────┘
```

### 1.2 Social Media Flow (Python)

```
  HackerNews/Reddit/YouTube -> Collect -> Deduplicate -> Embed (TEI) -> Qdrant
  Collection: social_intelligence (768 dim, Cosine)
```

---

## 2. Economic Data Pipeline (Rust)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      ECONOMIC DATA PIPELINE (Rust)                          │
└─────────────────────────────────────────────────────────────────────────────┘

  DATA SOURCES:
  ├── Yahoo Commodities: Gold, Oil, Coal, CPO, Nickel, Tin, Copper, Rubber, Gas
  ├── CoinGecko: BTC, ETH, USDT, BNB, XRP
  ├── FRED: GDP, CPI, Unemployment, Fed Funds, 10Y Treasury, USD Index
  ├── Bank Indonesia: BI Rate, JIBOR, USD/IDR, Inflation
  └── GDELT: Indonesian event monitoring

  FLOW:
  Collectors -> economic_indicators (ArangoDB) -> signal_source edges

  CLI: cargo run -- economic [commodity|crypto|fred|bi|all]
```

---

## 3. Intelligence Fusion (ArangoDB Graph)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    INTELLIGENCE FUSION (ArangoDB Graph)                     │
└─────────────────────────────────────────────────────────────────────────────┘

  1. IDX Analyst requests signals for ticker (e.g., "BBRI")

  2. signal_lookup.rs executes AQL graph traversal:
     - Find actors related to ticker's sector
     - Find recent articles mentioning those actors
     - Find economic indicators correlated to sector
     - Aggregate signals with temporal decay

  3. Temporal decay: strength = base * exp(-0.693 * hours / half_life)

  4. ExternalSignal struct feeds into 5-persona debate engine

  Flow:
  Ticker -> AQL Graph Traversal -> ExternalSignal (with decay) -> Debate Engine
```

---

## 4. Market Data Pipeline (Planned)

```
  IDX Stocks + Forex + Commodities -> OHLCV -> Technical Indicators -> Store
  (via yfinance, market hours only)
```

---

## 5. Knowledge Ingestion Pipeline (Planned)

```
  PDF/EPUB/TXT -> Extract -> Chunk (512 tokens) -> Embed (TEI) -> Qdrant
  Collections: pagupon-kb, pondo-business-kb (768 dim, Cosine)
```

---

## Storage Summary

### ArangoDB (Primary)
| Collection | Type | Content |
|------------|------|---------|
| `articles` | Document | News articles + Prof Jiang labels + vectors |
| `economic_indicators` | Document | Commodity, crypto, macro data |
| `actors` | Document | Extracted actors |
| `topics` | Document | Extracted topics |
| `signals` | Document | Investment signals |
| `signal_source` | Edge | Signal -> source article/indicator |
| `article_mentions_actor` | Edge | Article -> Actor |
| `intelligence_graph` | Graph | All collections connected |

### Qdrant (Legacy Fallback)
| Collection | Dimensions | Content |
|------------|------------|---------|
| `news_articles` | 768 | RSS news + Prof Jiang labels |
| `social_intelligence` | 768 | Social media posts |
| `pagupon-kb` | 768 | Investment books |
| `pondo-business-kb` | 768 | F&B business knowledge |

---

## Scheduling

| Pipeline | Schedule | Mode |
|----------|----------|------|
| RSS News | Every 15 min | Daemon |
| Economic Data | Every hour | CLI / Cron |
| Social Media | Every 2 hours | Cron |
| Feed Health Check | Daily | GitHub Actions |
| Market Data | Market hours (planned) | Cron |
| Knowledge | On-demand | Manual |

---

## Service Dependencies

```
  Required for:
  ├── ArangoDB :8529  -> News + Economic + Intelligence Fusion (primary store)
  ├── Qdrant :6333    -> Social Media + legacy news (fallback)
  ├── TEI :8082       -> All pipelines (embeddings)
  ├── Kiromania :9000 -> RSS News only (Prof Jiang labeling)
  └── SQLite (local)  -> RSS News staging
```

---

## Quick Start

```bash
# 1. Start infrastructure
cd infrastructure
docker compose up -d              # ArangoDB + Qdrant + TEI

# 2. Run RSS News pipeline (Rust)
cd ../news-social-intelligence-data-pipeline
cargo run --release -- daemon

# 3. Run Economic data collection
cargo run --release -- economic all

# 4. Run Social Media pipeline (Python)
python social_intel_cron.py

# 5. Query ArangoDB
curl -X POST http://localhost:8529/_db/hermes/_api/cursor \
  -H "Content-Type: application/json" \
  -d '{"query": "FOR doc IN articles SORT doc.published_at DESC LIMIT 5 RETURN doc"}'

# 6. Query Qdrant (legacy)
curl -s "http://localhost:6333/collections/news_articles/points/scroll" \
  -H "Content-Type: application/json" \
  -d '{"limit": 5, "with_payload": true}' | jq
```
