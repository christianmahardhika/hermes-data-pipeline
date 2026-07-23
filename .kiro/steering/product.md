# Product: Hermes Data Pipeline

Hermes Data Pipeline is a collection of ETL pipelines that feed the Hermes Agent ecosystem with curated intelligence data from multiple sources.

## Core Pipelines

| Pipeline | Language | Status | Description |
|----------|----------|--------|-------------|
| News & Social Intelligence | Rust + Python | Active | RSS news + social media collection |
| Economic Data | Rust | Active | Commodities, crypto, macro indicators |
| Market Data | Python | Planned | IDX stocks, forex, commodities |
| Social Media (dedicated) | Python | Planned | X/Twitter sentiment monitoring |
| Knowledge Ingestion | Python | Planned | PDF/EPUB to vector DB for RAG |

## Business Context

- **Purpose**: Feed AI agents with structured intelligence data for investment/business analysis
- **Framework**: Prof Jiang Game Theory — analyze news as strategic moves on geopolitical/economic board
- **Users**: Pagupon group internal AI agents (not end-user facing)
- **Data volume**: 31 RSS feeds every 15 min + social media every 2 hours + economic data hourly
- **Output**: ArangoDB (primary, graph + vectors) + Qdrant (legacy fallback) for semantic search and RAG

## Prof Jiang Game Theory Framework

Each article is analyzed for:

| Field | Description |
|-------|-------------|
| `actors` | Key players (governments, companies, individuals) with incentives/constraints |
| `events` | What happened, trigger, significance, tense (past/present/future) |
| `relations` | Power dynamics, alliances, conflicts between actors |
| `context` | Geopolitical, economic, social background factors |
| `pattern_match` | Historical parallel patterns (trade_war, currency_crisis, etc.) |
| `investment_signal` | bullish / bearish / neutral / hold / defensive with confidence |

## News Sources (31 feeds)

### Indonesian National (11)
Tempo, CNN Indonesia, Antara, Republika, Detik News, Detik Finance, Kompas, Okezone, Sindonews, Kontan, CNBC Indonesia

### International Business (6)
BBC Business, BBC World, CNBC, Bloomberg, Financial Times, MarketWatch

### International General (4)
Al Jazeera, The Guardian, NPR, AP News

### Asia Pacific (4)
Channel News Asia, Nikkei Asia, South China Morning Post, Straits Times

## Social Media Sources

| Source | Auth | Topics |
|--------|------|--------|
| HackerNews | No (Algolia API) | Tech, AI, Startups, Business |
| Reddit | No (RSS feeds) | News, Geopolitics, Finance, Conspiracy, Indonesia |
| YouTube | No (yt-dlp) | Tech podcasts, Business, Politics |
| X/Twitter | Yes (disabled) | — |

## ArangoDB Collections (Primary Intelligence Store)

Controlled by `STORAGE_BACKEND` env var: `"arangodb"` (default) or `"qdrant"` (legacy fallback).

### Document Collections (7)
| Collection | Content |
|------------|---------|
| `articles` | News articles with Prof Jiang labels |
| `sources` | RSS feed source metadata |
| `actors` | Extracted actors (governments, companies, people) |
| `topics` | Extracted topics and themes |
| `economic_indicators` | Commodity, crypto, macro data points |
| `signals` | Investment signals with confidence |
| `social_posts` | Social media posts |

### Edge Collections (6)
| Collection | From → To | Relationship |
|------------|-----------|-------------|
| `article_mentions_actor` | articles → actors | Actor mentioned in article |
| `article_has_topic` | articles → topics | Topic tagged on article |
| `actor_relates_actor` | actors → actors | Power dynamics, alliances |
| `signal_source` | signals → articles/economic_indicators | Signal derived from source |
| `article_similar` | articles → articles | Near-duplicate similarity |
| `topic_correlates` | topics → topics | Cross-topic correlation |

### Graph & Views
- **Graph**: `intelligence_graph` — connects all document/edge collections
- **ArangoSearch View**: `articles_search` — full-text + BM25 search on articles

## Qdrant Collections (Legacy Fallback)

| Collection | Dimensions | Content |
|------------|-----------|---------|
| `news_articles` | 768 (multilingual-e5-base) | RSS news + Prof Jiang labels |
| `social_intelligence` | 384 (all-MiniLM-L6-v2) / 768 (Rust port) | Social media posts |
| `unlimited_indonesian_current` | 768 | Indonesian RSS daemon |
| `unlimited_international_current` | 768 | International RSS daemon |
| `pagupon-kb` | 768 | Investment/business books |
| `pondo-business-kb` | 768 | F&B business knowledge |

## Economic Data Sources

| Source | Data | Auth |
|--------|------|------|
| Yahoo Finance | 11 commodity symbols (Gold, Oil WTI, Brent, CPO, Silver, Copper, Natural Gas, Nickel, Aluminum, Coffee Arabica, Coffee Robusta) | No |
| CoinGecko | BTC, ETH, USDT, BNB, XRP | No |
| FRED (St. Louis Fed) | 6 series: GDP, CPI, Unemployment, Fed Funds Rate, 10Y Treasury, USD Index | FRED_API_KEY (optional) |
| Bank Indonesia | BI Rate, JIBOR, USD/IDR, Inflation | No |
| GDELT | Event-based: Indonesian events, instability indices | No |

## Intelligence Correlation Layer

Cross-pipeline intelligence fusion connecting news events to market signals:

| From | To | Correlation Type |
|------|----|-----------------|
| News (Prof Jiang) | IDX Analyst | Actor-based sector mapping |
| Social sentiment | News events | Same-event detection (0.85 threshold) |
| Commodity prices | IDX tickers | Direct price linkage |
| News pattern_match | Portfolio risk | Defensive signal activation |
| Economic indicators | ArangoDB graph | signal_source edges to ExternalSignal |

Key principle: No pipeline operates in isolation. Every data point feeds into the correlation engine via ArangoDB graph for multi-source signal validation.
