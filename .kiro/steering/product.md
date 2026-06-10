# Product: Hermes Data Pipeline

Hermes Data Pipeline is a collection of ETL pipelines that feed the Hermes Agent ecosystem with curated intelligence data from multiple sources.

## Core Pipelines

| Pipeline | Language | Status | Description |
|----------|----------|--------|-------------|
| News & Social Intelligence | Rust + Python | Active | RSS news + social media collection |
| Market Data | Python | Planned | IDX stocks, forex, commodities |
| Social Media (dedicated) | Python | Planned | X/Twitter sentiment monitoring |
| Knowledge Ingestion | Python | Planned | PDF/EPUB to vector DB for RAG |

## Business Context

- **Purpose**: Feed AI agents with structured intelligence data for investment/business analysis
- **Framework**: Prof Jiang Game Theory — analyze news as strategic moves on geopolitical/economic board
- **Users**: Pagupon group internal AI agents (not end-user facing)
- **Data volume**: 29 RSS feeds every 15 min + social media every 2 hours
- **Output**: Qdrant vector collections for semantic search and RAG

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

## News Sources (29 feeds)

### Indonesian National (15)
Tempo, CNN Indonesia, Antara, Republika, Merdeka, Tribunnews, Detik, Kompas, Liputan6, Okezone, Sindonews, Bisnis Indonesia, Kontan, CNBC Indonesia, IDN Times

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

## Qdrant Collections

| Collection | Dimensions | Content |
|------------|-----------|---------|
| `news_articles` | 768 (multilingual-e5-base) | RSS news + Prof Jiang labels |
| `social_intelligence` | 384 (all-MiniLM-L6-v2) | Social media posts |
| `pagupon-kb` | 768 | Investment/business books |
| `pondo-business-kb` | 768 | F&B business knowledge |
