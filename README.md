# Hermes Data Pipeline

Collection of data pipelines for the Hermes Agent ecosystem.

## Pipelines

| Pipeline | Description | Status |
|----------|-------------|--------|
| [news-social-intelligence-data-pipeline](./news-social-intelligence-data-pipeline/) | RSS news collection, game theory labeling, vector embeddings | ✅ Active |

## Architecture Overview

```
                    ┌─────────────────────────────────────────┐
                    │         Hermes Data Pipeline            │
                    └─────────────────────────────────────────┘
                                       │
           ┌───────────────────────────┼───────────────────────────┐
           │                           │                           │
           ▼                           ▼                           ▼
┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐
│  News & Social      │   │  (Future)           │   │  (Future)           │
│  Intelligence       │   │  Market Data        │   │  Social Media       │
│  Pipeline           │   │  Pipeline           │   │  Pipeline           │
└─────────────────────┘   └─────────────────────┘   └─────────────────────┘
           │
           ▼
┌─────────────────────┐
│  Qdrant Vector DB   │
└─────────────────────┘
```

## Shared Infrastructure

All pipelines use:
- **Qdrant** — Vector database for semantic search
- **TEI** — Text Embeddings Inference (multilingual-e5-base)
- **Kiromania** — LLM gateway for labeling/analysis
- **SQLite** — Local staging database

## Getting Started

Each pipeline has its own README with specific setup instructions.

```bash
# Clone
git clone https://github.com/christianmahardhika/hermes-data-pipeline.git
cd hermes-data-pipeline

# Run specific pipeline
cd news-social-intelligence-data-pipeline
cargo run --release -- daemon
```

## License

Apache 2.0
