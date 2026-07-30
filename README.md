# Hermes Data Pipeline

Collection of data pipelines for the Hermes Agent ecosystem.

## Pipelines

| Pipeline | Language | Description | Status |
|----------|----------|-------------|--------|
| [news-social-intelligence](./news-social-intelligence-data-pipeline/) | Rust | RSS news collection, game theory labeling, vector embeddings | ✅ Active |
| [market-data](./market-data-pipeline/) | Python | IDX stocks, forex, commodities pricing | ✅ Active |
| [nextjs-intelligence-dashboard](./nextjs-intelligence-dashboard/) | Next.js | Real-time intelligence dashboard with graph viz | ✅ Active |
| [social-media](./social-media-pipeline/) | Python | X/Twitter, Reddit sentiment monitoring | 📋 Planned |
| [knowledge-ingestion](./knowledge-ingestion-pipeline/) | Python | PDF/EPUB to vector DB for RAG | 📋 Planned |

## Architecture Overview

```
                    ┌─────────────────────────────────────────────────┐
                    │           Hermes Intelligence Pipeline          │
                    └─────────────────────────────────────────────────┘
                                            │
        ┌───────────────┬───────────────────┼───────────────┬───────────────┐
        │               │                   │               │               │
        ▼               ▼                   ▼               ▼               ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│    News &    │ │   Market     │ │    Web       │ │  Knowledge   │ │    Infra     │
│    Social    │ │    Data      │ │  Dashboard   │ │  Ingestion   │ │   (shared)   │
│  Intelligence│ │   Pipeline   │ │   (Next.js)  │ │   Pipeline   │ │              │
│    (Rust)    │ │   (Python)   │ │    :3000     │ │   (Python)   │ │              │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘ └──────┬───────┘ └──────────────┘
       │                │                │                │
       └────────────────┴────────────────┴────────────────┘
                                  │
                                  ▼
                    ┌─────────────────────────────┐
                    │     Processing Layer        │
                    │  ┌───────┐ ┌────────────┐   │
                    │  │ArangoDB│ │ Rust API   │   │
                    │  │ :8529  │ │ :8888     │   │
                    │  └───────┘ └────────────┘   │
                    └─────────────────────────────┘
                                  │
                                  ▼
                    ┌─────────────────────────────┐
                    │     Infrastructure Layer    │
                    │  ┌───────┐ ┌───┐ ┌───────┐  │
                    │  │Qdrant │ │TEI│ │Kiromania│ │
                    │  │:6333  │ │:82│ │ :9000  │ │
                    │  └───────┘ └───┘ └───────┘  │
                    └─────────────────────────────┘
```

## System Wiring & Data Flow

### Complete Architecture Stack

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          CLIENT LAYER                                   │
│  ┌─────────────────┐  ┌──────────────────┐  ┌─────────────────────┐     │
│  │   Browser       │  │   Mobile App     │  │   Hermes Agent      │     │
│  │  localhost:3000 │  │   (Future)       │  │   CLI Interface     │     │
│  └─────────────────┘  └──────────────────┘  └─────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ HTTP/WebSocket
┌─────────────────────────────────────────────────────────────────────────┐
│                         PRESENTATION LAYER                              │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    Next.js Dashboard (:3000)                    │    │
│  │  • Real-time intelligence visualization                         │    │
│  │  • Graph network analysis (D3.js/Recharts)                    │    │
│  │  • Sentiment analysis charts                                   │    │
│  │  • News timeline & market correlation                          │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ REST API/GraphQL
┌─────────────────────────────────────────────────────────────────────────┐
│                           API LAYER                                     │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                   Rust Intelligence API (:8888)                 │    │
│  │  • /api/dashboard (metrics, network, sentiment)               │    │
│  │  • /api/weekly (compiled intelligence reports)                │    │
│  │  • /api/correlations (market-news relationships)              │    │
│  │  • WebSocket endpoint (:8889) for real-time updates          │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ AQL/SQL Queries
┌─────────────────────────────────────────────────────────────────────────┐
│                          DATA LAYER                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   ArangoDB      │  │    Qdrant       │  │   File System   │         │
│  │   (:8529)       │  │   (:6333)       │  │   (Backups)     │         │
│  │                 │  │                 │  │                 │         │
│  │ • Articles      │  │ • Vectors       │  │ • Logs          │         │
│  │ • Actors        │  │ • Embeddings    │  │ • Archives      │         │
│  │ • Correlations  │  │ • Collections   │  │ • Reports       │         │
│  │ • Graph data    │  │ • Semantic      │  │ • Configs       │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
└─────────────────────────────────────────────────────────────────────────┘
                                    ▲
                                    │ Data Ingestion
┌─────────────────────────────────────────────────────────────────────────┐
│                        COLLECTION LAYER                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │ News Intelligence│  │ Market Pipeline │  │ Social Media    │         │
│  │    (Rust)       │  │   (Python)      │  │   (Python)      │         │
│  │                 │  │                 │  │                 │         │
│  │ • RSS Feeds     │  │ • Yahoo Finance │  │ • Twitter/X     │         │
│  │ • HackerNews    │  │ • IDX Stocks    │  │ • Reddit        │         │
│  │ • Reddit API    │  │ • Commodities   │  │ • Sentiment     │         │
│  │ • Game Theory   │  │ • Forex Rates   │  │ • Trends        │         │
│  └────────────────┘  └─────────────────┘  └─────────────────┘         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Shared Infrastructure

All pipelines use common services defined in [`infrastructure/`](./infrastructure/):

| Service | Port | Purpose | Status |
|---------|------|---------|--------|
| Qdrant | 6333/6334 | Vector database | ✅ Running |
| TEI | 8082 | Text embeddings (multilingual-e5-base, 768 dim) | ✅ Running |
| Kiromania | 9000 | LLM gateway (Claude, etc.) | ✅ Running |
| ArangoDB | 8529 | Graph database (news correlations, actors) | ✅ Running |
| Rust Intelligence API | 8888 | RESTful API backend | ✅ Running |
| WebSocket Server | 8889 | Real-time updates | ✅ Running |
| Next.js Dashboard | 3000 | Web visualization interface | ✅ Running |

### Dashboard Architecture

The intelligence dashboard provides real-time visualization of news analysis, market correlations, and social sentiment:

**Frontend Stack:**
- **Next.js 16** with React 19 and TypeScript
- **Ant Design** for UI components  
- **Recharts** for data visualization
- **Socket.io** for real-time updates
- **Tailwind CSS** for styling

**API Integration:**
```javascript
// Real-time dashboard data
const dashboardData = await fetch('http://localhost:8888/api/dashboard');
const weeklyReports = await fetch('http://localhost:8888/api/weekly');
const correlations = await fetch('http://localhost:8888/api/correlations');
```

**Key Features:**
- Graph network visualization of actor relationships
- Sentiment analysis timeline charts
- Market-news correlation heatmaps  
- Prof Jiang Xueqin predictive framework insights
- Bilingual Indonesian-English content analysis

### Quick Start

```bash
# Clone repo
git clone https://github.com/christianmahardhika/hermes-data-pipeline.git
cd hermes-data-pipeline

# Start shared services
cd infrastructure
cp .env.example .env
# Edit .env with your API keys
docker compose up -d

# Run a specific pipeline
cd ../news-social-intelligence-data-pipeline
cargo run --release -- daemon
```

## Data Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Collect   │ ──▶ │    Clean    │ ──▶ │    Label    │ ──▶ │    Embed    │
│  (sources)  │     │  (process)  │     │    (LLM)    │     │  (Qdrant)   │
└─────────────┘     └────────────┘     └─────────────┘     └─────────────┘
```

## Collections (Qdrant)

| Collection | Purpose | Pipeline |
|------------|---------|----------|
| `news_articles` | News with game theory labels | news-social-intelligence |
| `pagupon-kb` | Investment/business books | knowledge-ingestion |
| `pondo-business-kb` | F&B business knowledge | knowledge-ingestion |

## License

Apache 2.0
