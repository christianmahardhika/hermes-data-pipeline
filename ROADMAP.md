# Hermes Data Pipeline — Roadmap

## Current Status: Alpha (v0.1)
**Completion:** 33% (14/42 features working)  
**Next:** Week 1 - Foundation & Honesty

## v0.1 (Current — 2026-08-22)
- ✅ Docker infrastructure (ArangoDB, Qdrant, TEI)
- ✅ IDX Analyst with real Indonesian stock data
- ✅ Unlimited Indonesian news collector
- ✅ Economic data pipeline (partial)
- ✅ Hermes Agent cron integration (7 active jobs)
- ✅ Dashboard API with real portfolio data (Rp 13.86M)
- ❌ **Missing:** RSS pipeline, social media, Qdrant integration

## v0.2 — Market Data (Week 2: 2026-08-29)
**Goal:** End-to-end working market data pipeline

### Features
- [ ] Live commodity prices via yfinance (LME Nickel, Coal, CPO)
- [ ] Live IDX stock prices (BMRI, BBRI, INCO, PTBA, TAPG, ANTM)
- [ ] Forex rates (USD/IDR, other pairs)
- [ ] ArangoDB storage for market data with TTL indices
- [ ] Dashboard market widget (real-time ticker)
- [ ] Fix BI currency scraper dependencies
- [ ] Add market data to weekly digest

### Technical Tasks
1. **Replace hardcoded commodity data** with yfinance API
2. **Implement Pydantic models** for data validation
3. **Add Redis cache** for rate limiting (5-minute TTL)
4. **Create market data API endpoint** in dashboard
5. **Update Hermes cron jobs** for reliable collection

### Success Criteria
- Commodity prices update every 30 minutes via Hermes cron
- Dashboard shows real-time market data (not placeholders)
- No breaking changes to existing IDX analyst

## v0.3 — RSS News Pipeline (Week 4–5: 2026-09-12)
**Goal:** Complete Rust-based news processing pipeline

### Features
- [ ] RSS collection (25+ feeds: Kompas, Detik, Tempo, CNN Indonesia, etc.)
- [ ] HTML cleaning and text normalization
- [ ] LLM labeling with Prof Jiang framework
- [ ] TEI embedding + Qdrant storage
- [ ] Deduplication and rate limiting
- [ ] Daemon mode with health checks
- [ ] Article search API

### Technical Tasks
1. **Implement Rust collector** (`collect` command)
2. **Implement HTML cleaner** (`clean` command)
3. **Implement Kiromania labeler** (`label` command)
4. **Implement TEI embedder** (`embed` command)
5. **Add Qdrant integration** for vector storage
6. **Create daemon mode** with scheduled runs

### Success Criteria
- Pipeline processes 100+ articles/day automatically
- Search API returns semantically similar articles
- LLM costs < $0.003 per article (via batching)
- No silent failures (all errors logged and alerted)

## v0.4 — Production Readiness (Week 6–7: 2026-09-26)
**Goal:** Monitoring, observability, and reliability

### Features
- [ ] Prometheus metrics for all services
- [ ] Grafana dashboards (pipeline health, costs, performance)
- [ ] Alerting via Alertmanager (Telegram notifications)
- [ ] Automated backups (daily ArangoDB + Qdrant snapshots)
- [ ] Security hardening (no exposed DB ports, secrets management)
- [ ] Rate limiting and circuit breakers
- [ ] Load testing and performance optimization

### Technical Tasks
1. **Add metrics** to Rust and Python code
2. **Deploy monitoring stack** (Prometheus, Grafana, Alertmanager)
3. **Implement backup scripts** with S3/MinIO upload
4. **Security audit** and fixes
5. **Load testing** with realistic data volumes

### Success Criteria
- 99% uptime for critical components
- < 200ms API latency (p95)
- Automated alerts for any pipeline failure
- 30-day data retention with automated cleanup

## v0.5 — Social Intelligence (Week 8: 2026-10-10)
**Goal:** Multi-source social media intelligence

### Features
- [ ] HackerNews collector (tech news)
- [ ] Reddit collector (social sentiment)
- [ ] YouTube analysis (transcript extraction)
- [ ] Near-duplicate detection (MinHash LSH)
- [ ] Sentiment analysis pipeline
- [ ] Correlation with market data
- [ ] Social dashboard widget

### Technical Tasks
1. **Implement Python collectors** for each platform
2. **Add deduplication** (Jaccard similarity > 0.85)
3. **Integrate with existing pipeline** (ArangoDB + Qdrant)
4. **Create correlation analysis** between social sentiment and market moves
5. **Add to dashboard** with real-time visualizations

### Success Criteria
- Collects 500+ social posts/day across platforms
- Detects market-moving sentiment shifts
- Correlates social buzz with stock performance
- Adds < 1s latency to existing pipeline

## v0.6 — Knowledge Base (Post-v1.0)
**Goal:** PDF/EPUB ingestion and RAG capabilities

### Features
- [ ] PDF text extraction (pymupdf, marker-pdf)
- [ ] EPUB parsing
- [ ] Chunking and embedding
- [ ] Qdrant storage for knowledge
- [ ] RAG API for querying documents
- [ ] Prof Jiang framework applied to documents

### Technical Tasks
1. **Research PDF extraction libraries** for Indonesian content
2. **Implement chunking strategy** (semantic, 500-1000 tokens)
3. **Add to embedding pipeline** (reuse TEI + Qdrant)
4. **Create RAG API endpoint** with source citations
5. **Integrate with dashboard** (knowledge search)

### Success Criteria
- Ingests 100+ PDF/EPUB documents
- Returns accurate answers with citations
- Supports Indonesian and English content
- Adds minimal overhead to existing system

## v1.0 — Stable Release (2026-11-01)
**Goal:** Production-ready, fully documented system

### Features
- [ ] 30-day daemon uptime (no crashes)
- [ ] 75% test coverage across all languages
- [ ] Complete documentation (API docs, deployment guide, troubleshooting)
- [ ] Public demo environment
- [ ] Contributor guide and code standards
- [ ] Performance benchmarks
- [ ] Security audit report

### Technical Tasks
1. **Achieve test coverage target** (Rust + Python)
2. **Document all APIs** with OpenAPI/Swagger
3. **Create demo deployment** with sample data
4. **Write contributor documentation**
5. **Performance optimization** pass
6. **Security audit** by external reviewer

### Success Criteria
- New contributor can set up system in 5 minutes
- API documented with 100% accuracy
- Zero critical security vulnerabilities
- Community adoption (5+ external contributors)

## Post-v1.0 Ideas
- **Graph analysis** (ArangoDB graph queries on actor relationships)
- **Predictive models** (LSTM on market + news data)
- **Mobile app** (React Native dashboard)
- **Multi-tenancy** (support multiple users/portfolios)
- **Local LLM fine-tuning** (Indonesian BERT for cheaper labeling)
- **Blockchain integration** (data provenance and immutability)
- **Enterprise features** (SSO, audit logs, compliance)

## Success Metrics

| Metric | v0.1 (Current) | v0.3 (Week 5) | v1.0 (Goal) |
|--------|----------------|---------------|-------------|
| **Test Coverage** | 0% | 40% | 75% |
| **Articles/Day** | 0 | 500 | 3,000 |
| **API Latency (p95)** | N/A | 500ms | 100ms |
| **LLM Cost/Article** | N/A | $0.005 | $0.002 |
| **Uptime** | N/A | 95% | 99.9% |
| **Onboarding Time** | 30 min | 15 min | 5 min |
| **Active Data Sources** | 6 | 20 | 50+ |

## Decision Points

### Week 1 Decision (Today): Choose First Vertical
**Options:**
1. **Market Data (Recommended)** — Fastest win, validates infrastructure
2. **RSS Pipeline** — Core product but complex
3. **Social Media** — High business value but dependency-heavy

**Recommendation:** Market Data → RSS → Social Media

### Week 4 Decision: LLM Provider Strategy
**Options:**
1. **Kiromania only** — Your existing gateway
2. **Multi-provider** — Fallback to OpenAI/Claude
3. **Local models** — Llama 3.3 70B local inference

**Recommendation:** Kiromania + local fallback for neutral articles

### Week 7 Decision: Deployment Strategy
**Options:**
1. **Single VPS** — Docker Compose (simplest)
2. **Kubernetes** — Scalable but complex
3. **Managed services** — AWS/GCP (expensive)

**Recommendation:** Single VPS for v1.0, Kubernetes post-v1.0

## Dependencies & Risks

### High-Risk Dependencies
1. **yfinance API** — Yahoo Finance rate limits and stability
2. **Kiromania gateway** — Your custom LLM gateway availability
3. **TEI embeddings** — Local embedding model performance

### Mitigation Strategies
- **yfinance**: Implement aggressive caching (Redis, 5-min TTL)
- **Kiromania**: Circuit breaker pattern, fallback to mock data
- **TEI**: Health checks, automatic restart on failure

## Resource Requirements

### Development
- **Time:** 2-3 hours/day for 8 weeks
- **Cost:** ~$100/month (VPS, LLM API costs)
- **Skills:** Rust, Python, Docker, DevOps

### Production
- **VPS:** 4 vCPU, 8GB RAM, 200GB SSD ($40/month)
- **LLM API:** $50-100/month (batching reduces cost)
- **Monitoring:** Free tier (Prometheus + Grafana)

## Timeline Summary

| Week | Focus | Deliverable |
|------|-------|-------------|
| **1** | Foundation & Honesty | AUDIT.md, honest README, choose vertical |
| **2** | Market Data | Live commodity prices, fix BI scraper |
| **3** | Market Data Polish | Dashboard integration, testing |
| **4** | RSS Pipeline | Rust collector + cleaner |
| **5** | RSS Pipeline | Labeler + embedder + Qdrant |
| **6** | Production Readiness | Metrics, monitoring, backups |
| **7** | Production Polish | Security, performance, documentation |
| **8** | Social Intelligence | HackerNews + Reddit collectors |
| **9-10** | v1.0 Polish | Testing, documentation, demo |
| **11** | v1.0 Release | Stable production release |

---

**Last Updated:** 2026-08-22  
**Next Review:** After Week 2 completion  
**Owner:** christianmahardhika  
**Status:** ACTIVE — Week 1 in progress