# Hermes Data Pipeline

**⚠️ Current Status: Alpha — Active Development**

This repo is a work in progress. Some documented features are not yet implemented.
See [ROADMAP.md](ROADMAP.md) for planned features and [AUDIT.md](AUDIT.md) for current code reality.

## What's Working Now

| Feature | Status | How to Run |
|---------|--------|------------|
| Infrastructure (Docker) | ✅ | `cd infrastructure && docker compose up -d` |
| IDX Stock Analyst (Rust) | ✅ | `cargo run --release -- idx-analyst BMRI BBRI` |
| IDX Portfolio Digest | ✅ | `cargo run --release -- economic` |
| Unlimited Indonesian News | ✅ | `cargo run --release -- unlimited` |
| Hermes Cron Integration | ✅ | `hermes cron list` (7 active jobs) |
| Python Scripts (partial) | ⚠️ | See scripts/hermes-config/ |
| Dashboard API | ✅ | Running at localhost:3002 |

## What's In Progress

| Feature | Status | ETA |
|---------|--------|-----|
| Market Data Collector (Python) | 🔄 Fixing | Week 2 |
| RSS News Pipeline (Rust) | 🔄 Restoring | Week 4–5 |
| Social Media Pipeline | 📋 Planned | Week 8 |
| Knowledge Ingestion | 📋 Planned | Post-v1.0 |

## Quick Start (Working Parts Only)

```bash
# 1. Start infrastructure
cd infrastructure
cp .env.example .env
# Edit .env with your API keys
docker compose up -d arangodb qdrant tei

# 2. Run IDX Analyst (mock data, no API keys needed)
cd ../news-social-intelligence-data-pipeline
cargo run --release -- idx-analyst BMRI BBRI

# 3. Run portfolio digest
cargo run --release -- economic

# 4. View Hermes cron jobs
hermes cron list
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           EXTERNAL SOURCES                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  25 RSS Feeds │  │ Yahoo Finance │  │ HackerNews   │  │ Reddit API   │  │
│  │  (ID + Int'l) │  │  (Stocks)     │  │  (Tech)      │  │  (Social)    │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
│         │                 │                 │                 │            │
└─────────┼─────────────────┼─────────────────┼─────────────────┼────────────┘
          │                 │                 │                 │
          ▼                 ▼                 ▼                 ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         COLLECTION LAYER (3 Services)                       │
│                                                                             │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐ │
│  │   News Collector    │  │  Market Collector   │  │  Social Collector   │ │
│  │     (Rust)          │  │     (Python)        │  │     (Python)        │ │
│  │  • RSS fetching     │  │  • yfinance         │  │  • HN API           │ │
│  │  • Deduplication    │  │  • Commodities      │  │  • Reddit RSS       │ │
│  │  • Rate limiting    │  │  • Forex            │  │  • YouTube          │ │
│  │  • Fallback URLs    │  │  • Rate limiting    │  │  • Rate limiting    │ │
│  └──────────┬──────────┘  └──────────┬──────────┘  └──────────┬──────────┘ │
│             │                        │                        │              │
└─────────────┼────────────────────────┼────────────────────────┼──────────────┘
              │                        │                        │
              └────────────────────────┼────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PROCESSING LAYER                                    │
│                                                                             │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐ │
│  │     Cleaner         │  │      Labeler        │  │     Embedder        │ │
│  │     (Rust)          │  │      (Rust)         │  │      (Rust)         │ │
│  │  • HTML sanitize    │  │  • Prof Jiang       │  │  • TEI client       │ │
│  │  • Text normalize   │  │    framework        │  │  • Batch embed      │ │
│  │  • Language detect  │  │  • Batch LLM        │  │  • Qdrant store     │ │
│  │  • Content hash     │  │  • Retry logic      │  │  • Payload enrich   │ │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         STORAGE LAYER                                       │
│                                                                             │
│  ┌─────────────────────────────┐  ┌─────────────────────────────────────┐  │
│  │      ArangoDB               │  │           Qdrant                    │  │
│  │   (Documents + Graph)       │  │        (Vectors)                    │  │
│  │                             │  │                                     │  │
│  │  • articles (doc)          │  │  • news_articles (768d)            │  │
│  │  • actors (doc)            │  │  • social_intelligence (768d)      │  │
│  │  • market_quotes (doc)     │  │  • pagupon-kb (768d)               │  │
│  │  • correlations (edge)     │  │  • pondo-business-kb (768d)        │  │
│  │  • actor_relations (edge)  │  │                                     │  │
│  │  • feed_health (doc)       │  │                                     │  │
│  └─────────────────────────────┘  └─────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Current Status Details

### ✅ Working Features
- **IDX Stock Analyst**: Rust CLI that analyzes Indonesian stocks with 5-persona debate engine
- **Economic Data Collection**: Pulls real Indonesian stock data (BMRI, BBRI, INCO, ANTM, PTBA, TAPG)
- **Hermes Integration**: 7 active cron jobs delivering to Telegram
- **ArangoDB Storage**: Stock data with portfolio tracking (Rp 13.86M real-time)
- **Prof Jiang KB**: 130 chunks across geostrategy, game theory, secret history
- **Dashboard API**: Next.js dashboard at localhost:3002 with real data

### ⚠️ Partially Working
- **Market Data Collector**: Python scripts with hardcoded commodity prices (needs yfinance)
- **BI Currency Scraper**: Needs `beautifulsoup4` and `lxml` dependencies
- **Rust Pipeline**: Command stubs exist but implementation incomplete

### ❌ Not Yet Implemented
- **RSS Feed Processing**: Collector, cleaner, labeler, embedder pipeline
- **Qdrant Integration**: Vector storage for semantic search
- **Social Media Collection**: HackerNews, Reddit, YouTube
- **Redis Cache**: Rate limiting and deduplication

## Repository Structure

```
hermes-data-pipeline/
├── README.md                    # This file (honest status)
├── AUDIT.md                     # Feature audit vs reality
├── ROADMAP.md                   # Future features
├── ARCHITECTURE.md              # Current architecture
├── LICENSE                      # Apache 2.0
├── .gitignore
├── infrastructure/              # Docker infrastructure
│   ├── docker-compose.yml       # ArangoDB + Qdrant + TEI
│   └── .env.example
├── news-social-intelligence-data-pipeline/  # Rust CLI
│   ├── src/main.rs              # CLI commands (partial)
│   └── Cargo.toml
├── scripts/                     # Python scripts
│   ├── hermes-config/          # Hermes Agent integration
│   └── hermes_pipeline_deploy.sh
├── documentation/              # Docs
│   ├── current-setup/         # Current configuration
│   └── OLD_README.md          # Previous dishonest docs
└── dashboard/                  # Next.js (separate repo)
```

## Development Setup

```bash
# Clone repository
git clone https://github.com/christianmahardhika/hermes-data-pipeline.git
cd hermes-data-pipeline

# Start infrastructure
cd infrastructure
docker compose up -d arangodb qdrant tei

# Install Python dependencies
pip install beautifulsoup4 lxml requests pandas python-arango

# Build Rust CLI
cd ../news-social-intelligence-data-pipeline
cargo build --release

# Run tests (minimal)
cargo test -- --test-threads=1
```

## Contributing

1. **Be Honest**: If a feature doesn't work, document that.
2. **One Vertical at a Time**: Perfect one pipeline before moving to the next.
3. **Test Thoroughly**: Add tests for every feature.
4. **Update Documentation**: Keep AUDIT.md, ROADMAP.md, and this README current.

### Development Workflow
```bash
# Check current status
cat AUDIT.md | grep -E "BROKEN|MISSING" | head -5

# Choose what to work on
# 1. Market Data (fastest win) - fix commodity_collector.py
# 2. RSS Pipeline (core) - implement Rust collector
# 3. Social Media (valuable) - Python collectors

# Create feature branch
git checkout -b feat/fix-market-data

# Test changes
python3 scripts/market-data-pipeline/commodity_collector.py --test
cargo test -- --test-threads=1

# Commit with honest message
git commit -m "feat(market): integrate yfinance for live commodity prices

- Replace hardcoded data with Yahoo Finance API
- Add error handling and retry logic
- Store results in ArangoDB"
```

## License

Apache 2.0