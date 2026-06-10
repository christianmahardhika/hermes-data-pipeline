# News RSS Data Pipeline

A Rust-based news collection pipeline that scrapes RSS feeds, cleans content, labels articles using game theory analysis (Prof Jiang framework), and stores embeddings in Qdrant for semantic search.

## Architecture

```
RSS Feeds (29 sources)
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

- **Multi-source RSS collection**: 29 feeds (Indonesian national + international business/general + Asia Pacific)
- **Batch LLM labeling**: 20 articles per API call for efficiency
- **Game theory analysis**: Actors, events, relations, market sentiment, investment signals
- **Semantic search**: 768-dim embeddings via TEI, stored in Qdrant
- **Daemon mode**: Continuous collection every 15 minutes

## News Sources

### Indonesian National
- Tempo, CNN Indonesia, Antara, Republika, Kompas, Tribunnews
- Okezone, Sindonews, Kontan, CNBC Indonesia, Merdeka

### International Business
- BBC Business, CNBC, Bloomberg, Financial Times, MarketWatch

### International General
- BBC World, Al Jazeera, The Guardian, NPR

### Asia Pacific
- Nikkei Asia, South China Morning Post, The Straits Times

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
src/
├── main.rs           # CLI entry point
├── lib.rs            # Shared config (Qdrant connection)
├── collectors/       # RSS feed fetching
├── cleaners/         # HTML processing
├── labelers/         # LLM game theory analysis
├── embedders/        # TEI + Qdrant ingestion
├── storage/          # SQLite operations
└── health/           # Service health checks
```

## License

Apache 2.0
