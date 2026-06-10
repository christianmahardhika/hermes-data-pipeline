# Hermes Data Pipeline

Collection of data pipelines for the Hermes Agent ecosystem.

## Pipelines

| Pipeline | Language | Description | Status |
|----------|----------|-------------|--------|
| [news-social-intelligence](./news-social-intelligence-data-pipeline/) | Rust | RSS news collection, game theory labeling, vector embeddings | ✅ Active |
| [market-data](./market-data-pipeline/) | Python | IDX stocks, forex, commodities pricing | 📋 Planned |
| [social-media](./social-media-pipeline/) | Python | X/Twitter, Reddit sentiment monitoring | 📋 Planned |
| [knowledge-ingestion](./knowledge-ingestion-pipeline/) | Python | PDF/EPUB to vector DB for RAG | 📋 Planned |

## Architecture Overview

```
                         ┌─────────────────────────────────────────┐
                         │         Hermes Data Pipeline            │
                         └─────────────────────────────────────────┘
                                            │
        ┌───────────────┬───────────────────┼───────────────┬───────────────┐
        │               │                   │               │               │
        ▼               ▼                   ▼               ▼               ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│    News &    │ │   Market     │ │    Social    │ │  Knowledge   │ │    Infra     │
│    Social    │ │    Data      │ │    Media     │ │  Ingestion   │ │   (shared)   │
│  Intelligence│ │   Pipeline   │ │   Pipeline   │ │   Pipeline   │ │              │
│    (Rust)    │ │   (Python)   │ │   (Python)   │ │   (Python)   │ │              │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘ └──────┬───────┘ └──────────────┘
       │                │                │                │
       └────────────────┴────────────────┴────────────────┘
                                  │
                                  ▼
                    ┌─────────────────────────────┐
                    │     Shared Infrastructure   │
                    │  ┌───────┐ ┌───┐ ┌───────┐  │
                    │  │Qdrant │ │TEI│ │Kiromania│ │
                    │  └───────┘ └───┘ └───────┘  │
                    └─────────────────────────────┘
```

## Shared Infrastructure

All pipelines use common services defined in [`infrastructure/`](./infrastructure/):

| Service | Port | Purpose |
|---------|------|---------|
| Qdrant | 6333/6334 | Vector database |
| TEI | 8082 | Text embeddings (multilingual-e5-base, 768 dim) |
| Kiromania | 9000 | LLM gateway (Claude, etc.) |

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
