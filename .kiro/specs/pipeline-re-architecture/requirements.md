# Requirements Document

## Introduction

The Hermes Data Pipeline has evolved from a focused RSS news collector into a comprehensive intelligence platform handling 31 RSS feeds, 3 social media sources, 27+ economic indicators, and a 5-persona stock analyst.

The re-architecture aims to decompose this monolith into focused, independently deployable services while preserving current functionality and improving maintainability, scalability, and developer experience.

## Current State Analysis

### 1. Monolithic Binary Structure

**Problem**: Single Rust binary handles 11 CLI commands with distinct concerns:

| Command | Domain | Dependencies |
|---------|--------|--------------|
| `collect` | News ingestion | Network, SQLite |
| `clean` | Text processing | SQLite only (no network) |
| `label` | LLM integration | Kiromania API |
| `embed` | Vector generation | TEI + ArangoDB/Qdrant |
| `daemon` | Scheduler | All of the above |
| `health` | Monitoring | Kiromania API |
| `prune` | Maintenance | SQLite only |
| `social` | Social collection | TEI, Qdrant |
| `unlimited` | Parallel collector | TEI, Qdrant |
| `idx-analyst` | Stock analysis | Yahoo Finance, ArangoDB |
| `economic` | Market data | Yahoo/CoinGecko/FRED/BI, ArangoDB |

**Impact**:
- Changes to IDX Analyst require rebuilding entire pipeline
- Cannot scale collectors independently from processors
- Daemon mode blocks on LLM API latency (labeling)
- Single failure point affects all functionality

### 2. Python ↔ Rust Duplication

**Problem**: Social intelligence exists in two implementations:

| Capability | Python (`social_intel/`) | Rust (`src/social/`) |
|------------|--------------------------|---------------------|
| HackerNews | `hackernews.py` (Algolia) | `hackernews.rs` (Algolia) |
| Reddit | `reddit.py` (RSS/Atom) | `reddit.rs` (RSS/Atom) |
| YouTube | `youtube.py` (yt-dlp) | `youtube.rs` (metadata) |
| Embeddings | SentenceTransformer (384d) | TEI (768d) |
| Storage | Qdrant only | Qdrant + ArangoDB |
| Tests | None | 44 tests |

**Impact**:
- Maintenance burden (two codebases doing same thing)
- Inconsistent embedding dimensions (384 vs 768)
- Python version is legacy but still scheduled via cron
- Rust port is feature-complete but Python not deprecated

### 3. SQLite Staging Intermediate Step

**Problem**: Data flows through SQLite before reaching ArangoDB:

```
RSS → SQLite (raw) → Clean → SQLite (cleaned) → Label → SQLite (labeled) → Embed → ArangoDB
```

**Impact**:
- 4 SQLite writes per article (raw, cleaned, labeled, status update)
- Disk I/O bottleneck for high-volume collection
- Schema migration complexity (SQLite + ArangoDB)
- Recovery requires scanning SQLite for incomplete items

### 4. Configuration Scatter

**Problem**: Environment variables defined in multiple places:

| Location | Variables | Purpose |
|----------|-----------|---------|
| `lib.rs` Config struct | 12 vars | Rust defaults |
| `infrastructure/.env.example` | 14 vars | Docker compose |
| `.kiro/steering/tech.md` | 10 vars | Documentation |
| `social_intel/__init__.py` | 4 vars | Python hardcoded |

**Impact**:
- No single source of truth for configuration
- Easy to miss required variables during deployment
- Different defaults between documentation and code
- Python hardcodes values instead of reading env

### 5. IDX Analyst Coupling

**Problem**: Stock analyst (`src/idx_analyst/`) is tightly coupled to main binary:

- 13 files, ~2500 lines of code
- Own domain models (`StockData`, `Signal`, `Confidence`)
- Own data sources (Yahoo Finance, ArangoDB graph queries)
- Own output formats (RTI Business, Telegram)
- Scheduled separately from news pipeline

**Impact**:
- Cannot update analyst without affecting news pipeline
- Different release cadence needs
- Portfolio configuration embedded in code (`config.rs`)
- No API for external consumers

### 6. Economic Collector Fragmentation

**Problem**: 5 different economic collectors with inconsistent patterns:

| Collector | Auth | Rate Limit | Storage | Error Handling |
|-----------|------|------------|---------|----------------|
| Yahoo Commodities | No | Implicit | ArangoDB | Retry |
| CoinGecko | No | 10-50/min | ArangoDB | Skip on error |
| FRED | API Key | 120/min | ArangoDB | Skip on error |
| Bank Indonesia | No | None | ArangoDB | Parse error → None |
| GDELT | No | None | ArangoDB | HTTP error → empty |

**Impact**:
- No unified interface for economic data
- Rate limiting handled differently per source
- Error handling inconsistent
- Difficult to add new economic sources

### 7. Missing Observability

**Problem**: Limited visibility into pipeline health:

- Logging: `tracing` with INFO level, no structured fields
- Metrics: None (no Prometheus/statsd)
- Tracing: None (no distributed tracing)
- Alerting: Telegram only for feed failures (> 10 consecutive)

**Impact**:
- Cannot measure LLM API latency distributions
- No visibility into embedding throughput
- Alert fatigue (all-or-nothing Telegram)
- Debugging production issues requires log grep

## Goals & Objectives

### Primary Goals

1. **Modularity**: Decompose monolith into focused services that can be developed, tested, and deployed independently
2. **Maintainability**: Reduce cognitive load for developers by clear separation of concerns
3. **Scalability**: Enable independent scaling of collection vs processing vs analysis
4. **Reliability**: Improve fault isolation so one component failure doesn't cascade

### Secondary Goals

5. **Developer Experience**: Faster feedback loops (rebuild only changed service)
6. **Operational Excellence**: Comprehensive observability (metrics, traces, alerts)
7. **Configuration Management**: Single source of truth for all configuration

### Non-Goals

- Complete rewrite in different language
- Changing core business logic (Prof Jiang framework, 5-persona debate)
- Migrating away from ArangoDB or TEI
- Adding new data sources (handled separately)

## Requirements

### FR-1: Service Decomposition

#### FR-1.1: News Collection Service
The system SHALL provide a standalone news collection service that:
- Fetches RSS feeds from 31 configured sources
- Implements circuit breaker pattern for feed health
- Stores raw articles directly to ArangoDB (bypassing SQLite staging)
- Exposes metrics for collection success/failure rates
- Can be scheduled independently (cron or daemon mode)

#### FR-1.2: Processing Pipeline Service
The system SHALL provide a processing service that:
- Retrieves unprocessed articles from ArangoDB
- Executes clean → label → embed phases sequentially
- Updates article status in ArangoDB after each phase
- Can process articles from any collection source
- Handles LLM API failures gracefully with retry/circuit breaker

#### FR-1.3: Social Intelligence Service
The system SHALL consolidate social media collection into a single implementation that:
- Supports HackerNews, Reddit, YouTube (and future X/Twitter)
- Uses consistent 768-dim embeddings (TEI multilingual-e5-base)
- Stores to ArangoDB with graph edges for correlation
- Deprecates Python `social_intel/` module

#### FR-1.4: Economic Data Service
The system SHALL provide a unified economic data collector that:
- Implements common interface for all sources
- Handles rate limiting per-source configuration
- Supports graceful degradation (continue on single source failure)
- Stores to ArangoDB `economic_indicators` collection

#### FR-1.5: IDX Analyst Service
The system SHALL extract IDX Analyst as a standalone service that:
- Provides HTTP API for stock analysis requests
- Accepts external signal injection (from news/economic correlation)
- Returns analysis in multiple formats (JSON, RTI, Telegram)
- Supports both real-time and batch portfolio analysis

### FR-2: Configuration Management

#### FR-2.1: Centralized Configuration
The system SHALL provide a centralized configuration mechanism that:
- Defines all environment variables in single canonical location
- Supports environment-specific overrides (dev/staging/prod)
- Validates configuration at startup
- Documents all variables with defaults and descriptions

#### FR-2.2: Service Discovery
The system SHALL enable services to discover each other through:
- Environment variables for internal service URLs
- Health check endpoints for liveness/readiness
- Graceful degradation when dependencies unavailable

### FR-3: Direct ArangoDB Integration

#### FR-3.1: Collection Storage
The news collection service SHALL store articles directly to ArangoDB:
- Insert raw article to `articles` collection with status "raw"
- Update status through processing phases
- Eliminate SQLite intermediate staging
- Maintain backward compatibility with existing schema

#### FR-3.2: Schema Migration
The system SHALL migrate existing SQLite data to ArangoDB:
- One-time migration script for pending items
- Validate data integrity post-migration
- Preserve article hashes for deduplication

### FR-4: Observability

#### FR-4.1: Structured Logging
All services SHALL emit structured logs with:
- Correlation IDs for request tracing
- Standard fields: service, level, timestamp, message
- JSON format for log aggregation compatibility

#### FR-4.2: Metrics
All services SHALL expose Prometheus metrics for:
- Request counts and latencies (p50, p95, p99)
- Queue depths and processing rates
- External API call success/failure rates
- Resource utilization (memory, connections)

#### FR-4.3: Health Endpoints
All services SHALL provide health endpoints:
- `/health/live` - service is running
- `/health/ready` - service can accept requests
- Dependency health included in readiness check

## Non-Functional Requirements

### NFR-1: Performance

#### NFR-1.1: Collection Throughput
- News collection SHALL process 31 feeds in < 5 minutes
- Social collection SHALL handle 3 sources in < 10 minutes
- Economic collection SHALL complete all sources in < 2 minutes

#### NFR-1.2: Processing Latency
- Clean phase SHALL process 100 articles in < 5 seconds
- Label phase SHALL process 20 articles in < 60 seconds (LLM-bound)
- Embed phase SHALL process 100 articles in < 30 seconds

#### NFR-1.3: Resource Efficiency
- Each service SHALL use < 256MB RSS memory at steady state
- Container images SHALL be < 50MB (Rust release build)

### NFR-2: Reliability

#### NFR-2.1: Fault Isolation
- Failure in one service SHALL NOT crash other services
- Network partitions SHALL trigger circuit breaker
- Queue overflow SHALL apply backpressure, not drop messages

#### NFR-2.2: Idempotency
- Collection operations SHALL be idempotent (re-running safe)
- Processing SHALL skip already-processed articles
- SHA256 content hash ensures no duplicate ingestion

### NFR-3: Maintainability

#### NFR-3.1: Code Organization
- Each service SHALL have its own Cargo.toml (workspace member)
- Shared code SHALL live in `hermes-common` crate
- Service-specific code SHALL NOT leak to common crate

#### NFR-3.2: Testing
- Unit test coverage SHALL be > 70% per service
- Integration tests SHALL cover service boundaries
- E2E tests SHALL validate full pipeline flow

### NFR-4: Security

#### NFR-4.1: Secrets Management
- No secrets in source code or container images
- All secrets via environment variables
- API keys rotatable without code changes

#### NFR-4.2: Network Security
- All external APIs accessed via HTTPS
- Internal services MAY use HTTP (within Docker network)

## Success Criteria

### Phase 1: Service Extraction (Weeks 1-4)

| Metric | Target | Measurement |
|--------|--------|-------------|
| News collection service deployed | Yes | Running in production |
| Processing service deployed | Yes | Running in production |
| SQLite staging eliminated | Yes | Direct ArangoDB writes |
| Python social_intel deprecated | Yes | Cron switched to Rust |
| Build time per service | < 2 min | CI metrics |

### Phase 2: IDX Analyst & Economic (Weeks 5-8)

| Metric | Target | Measurement |
|--------|--------|-------------|
| IDX Analyst HTTP API | Yes | API documentation published |
| Economic collectors unified | Yes | Single interface for all |
| Configuration centralized | Yes | Single .env.example |
| Service health endpoints | 100% | All services have /health |

### Phase 3: Observability & Polish (Weeks 9-12)

| Metric | Target | Measurement |
|--------|--------|-------------|
| Structured logging | 100% | JSON logs with correlation ID |
| Prometheus metrics | 100% | All services expose metrics |
| Unit test coverage | > 70% | Cargo tarpaulin report |
| Documentation | Complete | README per service |

### Rollback Criteria

- If news collection success rate drops below 80%, rollback to monolith
- If processing latency increases > 2x, investigate before proceeding
- If any data loss detected, immediate rollback and investigation

## Dependencies

### External Services
- ArangoDB 3.12 (primary storage)
- TEI multilingual-e5-base (embeddings)
- Kiromania LLM gateway (labeling)
- Yahoo Finance API (stock data)
- CoinGecko API (crypto data)
- FRED API (macro data)

### Internal Dependencies
```
hermes-common (shared types, config, ArangoDB client)
  ↑
  ├── hermes-collector (news collection)
  ├── hermes-processor (clean → label → embed)
  ├── hermes-social (social intelligence)
  ├── hermes-economic (market data)
  └── hermes-analyst (IDX stock analyst)
```

## Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| SQLite removal breaks recovery | High | Medium | Keep SQLite as optional fallback initially |
| Service communication overhead | Medium | Medium | Start with shared-nothing, add queues if needed |
| Configuration drift | Medium | High | Strict CI validation of .env coverage |
| Increased deployment complexity | Medium | High | Document deployment order, health checks |
| Team unfamiliarity with microservices | Medium | Low | Start with 2 services, expand gradually |

## Appendix: Current CLI Commands Mapping

| Current Command | Target Service | Notes |
|-----------------|---------------|-------|
| `cargo run -- collect` | hermes-collector | Standalone |
| `cargo run -- clean` | hermes-processor | Part of processing |
| `cargo run -- label` | hermes-processor | Part of processing |
| `cargo run -- embed` | hermes-processor | Part of processing |
| `cargo run -- daemon` | hermes-collector + hermes-processor | Orchestration |
| `cargo run -- health` | hermes-common (health check lib) | Shared utility |
| `cargo run -- prune` | hermes-collector | Maintenance command |
| `cargo run -- social` | hermes-social | Standalone |
| `cargo run -- unlimited` | hermes-collector | Alternative collector mode |
| `cargo run -- idx-analyst` | hermes-analyst | Standalone service |
| `cargo run -- economic` | hermes-economic | Standalone |

## Glossary

| Term | Definition |
|------|------------|
| Monolith | Current single Rust binary handling all pipeline commands |
| Service | Independently deployable Rust crate with its own binary |
| Cargo Workspace | Rust's native multi-crate project structure |
| hermes-common | Shared library crate containing config, types, and ArangoDB client |
| Circuit Breaker | Pattern to skip failing feeds after threshold failures |
| Prof Jiang | Game theory framework for analyzing news as strategic moves |
| TEI | Text Embeddings Inference server for generating vector embeddings |
| Kiromania | Internal LLM gateway for Prof Jiang labeling |
| AQL | ArangoDB Query Language for graph and document queries |
| SQLite Staging | Current intermediate storage between pipeline phases (to be eliminated) |
| IDX Analyst | 5-persona stock analysis engine with bull/bear debate |
| ExternalSignal | Economic indicator correlated to stock ticker for IDX Analyst |
| Strangler Fig | Incremental migration pattern replacing monolith piece by piece |
| RSS | Really Simple Syndication feed format for news sources |
| Near-duplicate Detection | Identifying similar articles using embedding similarity (threshold 0.95) |
| Graph Edge | ArangoDB relationship connecting two document collections |
| Intelligence Graph | Named graph connecting articles, actors, topics, and signals |
