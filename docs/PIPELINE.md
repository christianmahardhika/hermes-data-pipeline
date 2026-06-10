# Data Pipeline Documentation

Complete data flow documentation for all Hermes data pipelines.

## Overview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           HERMES DATA PIPELINE                                │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐     │
│   │    RSS      │   │   Social    │   │   Market    │   │  Knowledge  │     │
│   │    News     │   │    Media    │   │    Data     │   │  Ingestion  │     │
│   │   (Rust)    │   │  (Python)   │   │  (Python)   │   │  (Python)   │     │
│   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘     │
│          │                 │                 │                 │            │
│          ▼                 ▼                 ▼                 ▼            │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    SHARED INFRASTRUCTURE                            │   │
│   │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐               │   │
│   │  │ Qdrant  │  │   TEI   │  │Kiromania│  │ SQLite  │               │   │
│   │  │ :6333   │  │  :8082  │  │  :9000  │  │ (local) │               │   │
│   │  └─────────┘  └─────────┘  └─────────┘  └─────────┘               │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. News & Social Intelligence Pipeline

### 1.1 RSS News Flow (Rust)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         RSS NEWS PIPELINE (Rust)                            │
└─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────┐
  │                           DATA SOURCES                                   │
  │                                                                          │
  │  🇮🇩 Indonesian National          💼 International Business              │
  │  ├── Tempo                        ├── BBC Business                       │
  │  ├── CNN Indonesia                ├── CNBC                               │
  │  ├── Antara                       ├── Bloomberg                          │
  │  ├── Republika                    ├── Financial Times                    │
  │  ├── Kompas                       └── MarketWatch                        │
  │  ├── Tribunnews                                                          │
  │  ├── Okezone                      🌍 International General               │
  │  ├── Sindonews                    ├── BBC World                          │
  │  ├── Kontan                       ├── Al Jazeera                         │
  │  ├── CNBC Indonesia               ├── The Guardian                       │
  │  └── Merdeka                      └── NPR                                │
  │                                                                          │
  │  🌏 Asia Pacific                                                         │
  │  ├── Nikkei Asia                                                         │
  │  ├── South China Morning Post                                            │
  │  └── The Straits Times                                                   │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 1: COLLECT                                                        │
  │  ─────────────────                                                       │
  │  • Fetch RSS feeds (29 sources)                                          │
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
  │  • Extract:                                                              │
  │    ┌──────────────────┬────────────────────────────────────────────┐    │
  │    │ actors           │ Key players (govts, companies, people)     │    │
  │    │ events           │ What happened and significance             │    │
  │    │ relations        │ Power dynamics, alliances, conflicts       │    │
  │    │ context          │ Background factors                         │    │
  │    │ pattern_match    │ Similar historical patterns                │    │
  │    │ investment_signal│ bullish/bearish/neutral/hold/defensive     │    │
  │    └──────────────────┴────────────────────────────────────────────┘    │
  │                                                                          │
  │  Output: labeled table (SQLite)                                          │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 4: EMBED                                                          │
  │  ─────────────────                                                       │
  │  • Generate embeddings via TEI (multilingual-e5-base)                    │
  │  • Vector dimension: 768                                                 │
  │  • Store to Qdrant: news_articles collection                             │
  │                                                                          │
  │  Payload:                                                                │
  │  {                                                                       │
  │    "title": "...",                                                       │
  │    "content": "...",                                                     │
  │    "url": "...",                                                         │
  │    "source": "Tempo",                                                    │
  │    "published_at": "2026-06-10T...",                                     │
  │    "actors": ["KPK", "Menteri X"],                                       │
  │    "events": "...",                                                      │
  │    "relations": "...",                                                   │
  │    "context": "...",                                                     │
  │    "pattern_match": "...",                                               │
  │    "investment_signal": "defensive"                                      │
  │  }                                                                       │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                         ┌─────────────────────┐
                         │      QDRANT         │
                         │  news_articles      │
                         │  (768 dim, Cosine)  │
                         └─────────────────────┘
```

### 1.2 Social Media Flow (Python)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      SOCIAL MEDIA PIPELINE (Python)                         │
└─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────┐
  │                           DATA SOURCES                                   │
  │                                                                          │
  │  📰 HackerNews                    📺 YouTube                              │
  │  ├── AI / Machine Learning        ├── Tech podcasts                      │
  │  ├── Startups / VC                ├── Business / Finance                 │
  │  └── Business / Economics         └── Geopolitics                        │
  │                                                                          │
  │  🗣️ Reddit                                                               │
  │  ├── Tech: r/MachineLearning, r/LocalLLaMA, r/technology, r/Futurology   │
  │  ├── Business: r/business, r/Economics, r/stocks, r/wallstreetbets       │
  │  ├── News: r/worldnews, r/geopolitics, r/anime_titties, r/neutralnews    │
  │  ├── Conspiracy: r/conspiracy, r/actualconspiracies, r/HighStrangeness   │
  │  └── Indonesia: r/indonesia, r/finansial                                 │
  │                                                                          │
  │  🐦 X/Twitter (disabled - needs API key)                                 │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 1: COLLECT                                                        │
  │  ─────────────────                                                       │
  │  • HackerNews: Algolia Search API (no auth)                              │
  │  • Reddit: Public JSON API (no auth for read)                            │
  │  • YouTube: Search API (no auth for basic)                               │
  │  • Extract: title, url, content/description, score, published_at         │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 2: DEDUPLICATE                                                    │
  │  ─────────────────────                                                   │
  │  • Near-duplicate detection (MinHash / SimHash)                          │
  │  • Title similarity threshold: 0.85                                      │
  │  • URL normalization                                                     │
  │  • Skip if already in Qdrant                                             │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 3: EMBED & STORE                                                  │
  │  ───────────────────────                                                 │
  │  • Generate embeddings via TEI                                           │
  │  • Store to Qdrant: social_intelligence collection                       │
  │                                                                          │
  │  Payload:                                                                │
  │  {                                                                       │
  │    "source": "HackerNews",                                               │
  │    "title": "...",                                                       │
  │    "url": "...",                                                         │
  │    "content": "...",                                                     │
  │    "score": 2004,                                                        │
  │    "published_at": "..."                                                 │
  │  }                                                                       │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                         ┌─────────────────────┐
                         │      QDRANT         │
                         │ social_intelligence │
                         │  (768 dim, Cosine)  │
                         └─────────────────────┘
```

---

## 2. Market Data Pipeline (Planned)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       MARKET DATA PIPELINE (Python)                         │
└─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────┐
  │                           DATA SOURCES                                   │
  │                                                                          │
  │  📈 IDX Stocks (via yfinance)     💱 Forex                               │
  │  ├── BBRI.JK                      ├── USD/IDR                            │
  │  ├── BMRI.JK                      ├── EUR/USD                            │
  │  ├── BBNI.JK                      └── JPY/USD                            │
  │  ├── TLKM.JK                                                             │
  │  └── LQ45 constituents            🛢️ Commodities                         │
  │                                   ├── Gold (XAU/USD)                     │
  │                                   ├── Oil (Brent, WTI)                   │
  │                                   └── Coal, CPO                          │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 1: COLLECT                                                        │
  │  ─────────────────                                                       │
  │  • Fetch OHLCV data (Open, High, Low, Close, Volume)                     │
  │  • Interval: 1d, 1h, 5m (configurable)                                   │
  │  • Schedule: Market hours only                                           │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 2: PROCESS                                                        │
  │  ─────────────────                                                       │
  │  • Calculate technical indicators:                                       │
  │    - MA (20, 50, 100, 200)                                               │
  │    - RSI (14)                                                            │
  │    - MACD (12, 26, 9)                                                    │
  │    - Bollinger Bands                                                     │
  │  • Detect signals (Golden Cross, Death Cross, etc.)                      │
  └────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 3: STORE                                                          │
  │  ─────────────────                                                       │
  │  • TimescaleDB or SQLite (time-series optimized)                         │
  │  • Schema:                                                               │
  │    {                                                                     │
  │      "symbol": "BBRI.JK",                                                │
  │      "timestamp": "2026-06-10T09:00:00",                                 │
  │      "open": 4500, "high": 4550, "low": 4480, "close": 4520,             │
  │      "volume": 123456789,                                                │
  │      "ma_20": 4480, "ma_50": 4420, "rsi_14": 55.2                        │
  │    }                                                                     │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                         ┌─────────────────────┐
                         │  SQLite/TimescaleDB │
                         │    market_data      │
                         └─────────────────────┘
```

---

## 3. Knowledge Ingestion Pipeline (Planned)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    KNOWLEDGE INGESTION PIPELINE (Python)                    │
└────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────┐
  │                           DATA SOURCES                                   │
  │                                                                          │
  │  📚 Documents                                                            │
  │  ├── PDF (books, papers, reports)                                        │
  │  ├── EPUB (e-books)                                                      │
  │  └── TXT/MD (notes, articles)                                            │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 1: EXTRACT                                                        │
  │  ─────────────────                                                       │
  │  • PDF: PyMuPDF (fitz) for text, marker-pdf for OCR                      │
  │  • EPUB: ebooklib                                                        │
  │  • Extract metadata: title, author, chapters                             │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 2: CHUNK                                                          │
  │  ─────────────────                                                       │
  │  • Semantic chunking (by paragraph/section)                              │
  │  • Chunk size: 512 tokens                                                │
  │  • Overlap: 50 tokens                                                    │
  │  • Preserve chapter/section context                                      │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │  PHASE 3: EMBED & STORE                                                  │
  │  ───────────────────────                                                 │
  │  • Generate embeddings via TEI                                           │
  │  • Dedupe by content hash                                                │
  │  • Store to Qdrant: pagupon-kb or pondo-business-kb                      │
  │                                                                          │
  │  Payload:                                                                │
  │  {                                                                       │
  │    "source_file": "intelligent_investor.pdf",                            │
  │    "title": "The Intelligent Investor",                                  │
  │    "author": "Benjamin Graham",                                          │
  │    "chapter": "Chapter 8: The Investor and Market Fluctuations",         │
  │    "page": 198,                                                          │
  │    "chunk_index": 42,                                                    │
  │    "content": "...",                                                     │
  │    "content_hash": "abc123..."                                           │
  │  }                                                                       │
  └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                         ┌─────────────────────┐
                         │      QDRANT         │
                         │    pagupon-kb       │
                         │  (768 dim, Cosine)  │
                         └─────────────────────┘
```

---

## Qdrant Collections Summary

| Collection | Pipeline | Dimensions | Content |
|------------|----------|------------|---------|
| `news_articles` | RSS News | 768 | Indonesian + International news with Prof Jiang labels |
| `social_intelligence` | Social Media | 768 | HackerNews, Reddit, YouTube posts |
| `pagupon-kb` | Knowledge | 768 | Investment/business books |
| `pondo-business-kb` | Knowledge | 768 | F&B business knowledge |

---

## Scheduling

| Pipeline | Schedule | Mode |
|----------|----------|------|
| RSS News | Every 15 min | Daemon |
| Social Media | Every 2 hours | Cron |
| Market Data | Market hours (09:00-16:00 WIB) | Cron |
| Knowledge | On-demand | Manual |

---

## Service Dependencies

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SERVICE DEPENDENCY GRAPH                            │
└─────────────────────────────────────────────────────────────────────────────┘

                              ┌─────────────────┐
                              │    Pipelines    │
                              └────────┬────────┘
                                       │
           ┌───────────────────────────┼───────────────────────────┐
           │                           │                           │
           ▼                           ▼                           ▼
   ┌───────────────┐         ┌─────────────────┐         ┌───────────────┐
   │    Qdrant     │         │      TEI        │         │   Kiromania   │
   │    :6333      │         │     :8082       │         │    :9000      │
   │   (vector)    │         │  (embeddings)   │         │    (LLM)      │
   └───────────────┘         └─────────────────┘         └───────────────┘
           │                           │                           │
           │                           │                           │
           ▼                           ▼                           ▼
   ┌───────────────────────────────────────────────────────────────────────┐
   │                           Docker Network                              │
   │                       hermes-data-pipeline                            │
   └───────────────────────────────────────────────────────────────────────┘

   Required for:
   ├── Qdrant     → All pipelines (vector storage)
   ├── TEI        → All pipelines (embeddings)
   └── Kiromania  → RSS News only (Prof Jiang labeling)
```

---

## Quick Start

```bash
# 1. Start infrastructure
cd infrastructure
docker compose up -d

# 2. Run RSS News pipeline (Rust)
cd ../news-social-intelligence-data-pipeline
cargo run --release -- daemon

# 3. Run Social Media pipeline (Python)
python social_intel_cron.py

# 4. Query data
curl -s "http://localhost:6333/collections/news_articles/points/scroll" \
  -H "Content-Type: application/json" \
  -d '{"limit": 5, "with_payload": true}' | jq
```
