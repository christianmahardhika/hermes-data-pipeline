# Shared Infrastructure

Common services used by all pipelines.

## Services

| Service | Port | Purpose |
|---------|------|---------|
| Qdrant | 6333 (REST), 6334 (gRPC) | Vector database |
| TEI | 8082 | Text embeddings (multilingual-e5-base) |
| Kiromania | 9000 | LLM gateway |
| PostgreSQL | 5432 | Relational data (optional) |
| Redis | 6379 | Caching, queues (optional) |

## Quick Start

```bash
# Start all services
docker compose up -d

# Check status
docker compose ps

# View logs
docker compose logs -f tei
```

## Service Details

### Qdrant (Vector DB)
- REST API: http://localhost:6333
- gRPC: http://localhost:6334
- Dashboard: http://localhost:6333/dashboard

### TEI (Text Embeddings)
- Model: `intfloat/multilingual-e5-base`
- Dimensions: 768
- Endpoint: http://localhost:8082

### Kiromania (LLM Gateway)
- OpenAI-compatible API
- Endpoint: http://localhost:9000/v1

## Data Persistence

All data stored in Docker volumes:
- `qdrant_data` — Vector collections
- `tei_cache` — Model cache
