# News & Social Intelligence Data Pipeline

Multi-source intelligence collection pipeline combining:
1. **RSS News** (Rust) — 25 feeds with fallback URLs, game theory labeling via Prof Jiang framework
2. **Social Media** (Python) — HackerNews, Reddit, YouTube monitoring

## Architecture

```
RSS Feeds (25 sources)
       │
       ▼
┌─────────────┐
│  Collector  │  Fetch RSS, dedupe by URL
└─────────────┘
       │
       ▼
┌─────────────┐
│   Cleaner   │  HTML strip, normalize text
└─────────────┘
       │
       ▼
┌─────────────┐
│   Labeler   │  Prof Jiang game theory analysis via LLM
└─────────────┘
       │
       ▼
┌─────────────┐
│  Embedder   │  TEI multilingual-e5-base (768 dim)
└─────────────┘
       │
       ▼
┌─────────────┐
│   Qdrant    │  Vector storage + metadata
└─────────────┘
```

## Features

- **Multi-source RSS collection**: 25 feeds with fallback URL support (Indonesian national + international business/general + Asia Pacific)
- **Batch LLM labeling**: 20 articles per API call for efficiency
- **Game theory analysis**: Actors, events, relations, market sentiment, investment signals
- **Semantic search**: 768-dim embeddings via TEI, stored in Qdrant
- **Daemon mode**: Continuous collection every 15 minutes

## News Sources

### Indonesian National (11)
- Tempo, CNN Indonesia, Antara, Republika, Kompas
- Detik News, Detik Finance (with cross-fallbacks)
- Okezone, Sindonews, Kontan, CNBC Indonesia

### International Business (6)
- BBC Business, BBC World, CNBC, Bloomberg, Financial Times, MarketWatch

### International General (4)
- Al Jazeera, The Guardian, NPR, AP News

### Asia Pacific (4)
- Channel News Asia, Nikkei Asia, South China Morning Post, The Straits Times

## Requirements

- Rust 1.70+
- SQLite (staging database)
- Qdrant (vector database) on port 6333 (REST) / 6334 (gRPC)
- TEI (Text Embeddings Inference) on port 8082
- LLM API endpoint (OpenAI-compatible)

## Configuration

Set environment variables:
```bash
export LABELER_API_KEY="your-api-key"
export LABELER_BASE_URL="http://localhost:8787/v1"  # LLM endpoint
export LABELER_MODEL="claude-sonnet-4"
```

## Usage

### One-time full pipeline
```bash
cargo run --release -- --full-pipeline
```

### Daemon mode (continuous)
```bash
cargo run --release -- daemon
```

### Individual phases
```bash
cargo run --release -- collect   # Fetch RSS feeds
cargo run --release -- clean     # Process raw content
cargo run --release -- label     # Game theory analysis
cargo run --release -- embed     # Generate embeddings & store
```

### Health check
```bash
cargo run --release -- health
```

### IDX Analyst (5-persona debate engine)
```bash
cargo run --release -- idx-analyst --portfolio --mock   # All tickers, mock data
cargo run --release -- idx-analyst BMRI BBRI            # Specific tickers, live Yahoo
cargo run --release -- idx-analyst digest               # Full portfolio digest (cron mode)
cargo run --release -- idx-analyst digest --mock        # Digest with mock data
```

## Prof Jiang Game Theory Framework

Each article is analyzed for:

| Field | Description |
|-------|-------------|
| `actors` | Key players (governments, companies, individuals) |
| `events` | What happened and its significance |
| `relations` | Power dynamics, alliances, conflicts |
| `context` | Background factors affecting the situation |
| `pattern_match` | Similar historical patterns |
| `investment_signal` | bullish / bearish / neutral / hold / defensive |

## Qdrant Schema

Collection: `news_articles` (768 dimensions, Cosine distance)

Payload fields:
- `title`, `content`, `url`, `source`
- `published_at`, `collected_at`
- `actors`, `events`, `relations`, `context`
- `pattern_match`, `investment_signal`

## Project Structure

```
.
├── src/                      # Rust RSS Pipeline
│   ├── main.rs               # CLI entry point
│   ├── lib.rs                # Shared config (Qdrant connection)
│   ├── collectors/           # RSS feed fetching
│   ├── cleaners/             # HTML processing
│   ├── labelers/             # LLM game theory analysis
│   ├── embedders/            # TEI + Qdrant ingestion
│   ├── storage/              # SQLite operations
│   └── health/               # Service health checks
│
├── social_intel/             # Python Social Media Pipeline
│   ├── collector.py          # Main orchestrator
│   ├── hackernews.py         # HackerNews API (tech, business)
│   ├── reddit.py             # Reddit API (news, conspiracy, finance)
│   ├── youtube.py            # YouTube search (podcasts, trending)
│   ├── x_twitter.py          # X/Twitter (disabled, needs auth)
│   └── near_duplicate.py     # Deduplication logic
│
└── social_intel_cron.py      # Social media cron runner
```

---

## Social Media Intelligence

### Sources

| Source | Topics | Auth Required |
|--------|--------|---------------|
| HackerNews | Tech, AI, Business, Startups | ❌ No |
| Reddit | Global News, Geopolitics, Conspiracy, Finance | ❌ No |
| YouTube | Tech Podcasts, Business, Politics | ❌ No |
| X/Twitter | — | ✅ Yes (disabled) |

### Topics Monitored

**HackerNews & YouTube:**
- AI / Machine Learning / LLM
- Startup / Venture Capital / Business
- Tech News / Podcasts

**Reddit Subreddits:**
- Tech: r/MachineLearning, r/LocalLLaMA, r/technology, r/Futurology
- Business: r/business, r/Economics, r/stocks, r/investing, r/wallstreetbets
- News: r/worldnews, r/geopolitics, r/anime_titties, r/neutralnews
- Conspiracy: r/conspiracy, r/actualconspiracies, r/HighStrangeness
- Indonesia: r/indonesia, r/finansial

### Usage (Social Media)

```bash
# Run social media collector
python social_intel_cron.py

# Custom topics
python social_intel_cron.py --topics "AI,geopolitics,business" --depth quick
```

### Qdrant Collection

Collection: `social_intelligence` (768 dimensions)

Payload: `source`, `title`, `url`, `content`, `score`, `published_at`
