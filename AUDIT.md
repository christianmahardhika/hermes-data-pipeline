# Feature Audit — 2026-08-22

**System:** Hermes Data Pipeline  
**Profile:** social-politic-lab  
**Auditor:** Hermes Agent  
**Timestamp:** 2026-08-22T00:26:26Z  

| Feature | README Claims | Code Reality | Status | Notes |
|---------|--------------|--------------|--------|-------|
| **Rust Commands** |
| `cargo run -- daemon` | ✅ Full pipeline daemon | ⚠️ Function stub exists | BROKEN | `run_daemon()` exists but incomplete |
| `cargo run -- collect` | ✅ RSS collection only | ⚠️ Function stub exists | BROKEN | `run_collect()` exists but incomplete |
| `cargo run -- clean` | ✅ Content cleaning | ⚠️ Function stub exists | BROKEN | `run_clean()` exists but incomplete |
| `cargo run -- label` | ✅ LLM labeling | ⚠️ Function stub exists | BROKEN | `run_label()` exists but incomplete |
| `cargo run -- embed` | ✅ Embedding + Qdrant | ⚠️ Function stub exists | BROKEN | `run_embed()` exists but incomplete |
| `cargo run -- health` | ✅ Service health check | ✅ Implemented | WORKS | `run_health()` calls Kiro health check |
| `cargo run -- idx-analyst` | ✅ 5-persona analyst | ✅ Implemented | WORKS | `run_idx_analyst()` functional |
| `cargo run -- digest` | ✅ Portfolio digest | ✅ Implemented | WORKS | `run_economic()` includes digest |
| `cargo run -- unlimited` | ✅ Unlimited Indonesian news | ✅ Implemented | WORKS | `run_unlimited()` functional |
| `cargo run -- social` | ✅ Social media collection | ✅ Function stub | PARTIAL | `run_social()` exists but limited |
| `cargo run -- economic` | ✅ Economic data collection | ✅ Implemented | WORKS | `run_economic()` functional |
| `cargo run -- prune` | ❌ Not documented | ✅ Implemented | WORKS | `run_prune()` exists |
| **Python Scripts** |
| `python commodity_collector.py` | ✅ Live commodity prices | ⚠️ Hardcoded mock data | BROKEN | Returns placeholder data |
| `python collector.py --source idx` | ❌ Not documented | ❌ File doesn't exist | MISSING | No such file in repository |
| `python arangodb_news_monitor.py` | ✅ News database monitor | ✅ Implemented | WORKS | Fully functional |
| `python daily_tech_curation.py` | ✅ Daily tech curation | ✅ Implemented | WORKS | Fully functional |
| `python social_economic_analysis.py` | ✅ Correlation analysis | ✅ Implemented | WORKS | Fully functional |
| `python bi_currency_scraper.py` | ✅ BI currency rates | ⚠️ Missing dependencies | PARTIAL | Needs bs4, lxml |
| `python indonesian_news_collector.py` | ✅ Indonesian news | ✅ Implemented | WORKS | Fully functional |
| **Infrastructure** |
| `docker compose up -d` | ✅ Starts all services | ⚠️ Missing Kiromania | PARTIAL | ArangoDB, Qdrant, TEI work |
| `docker compose up -d arangodb` | ✅ ArangoDB service | ✅ Implemented | WORKS | Fully functional |
| `docker compose up -d qdrant` | ✅ Qdrant vector DB | ✅ Implemented | WORKS | Fully functional |
| `docker compose up -d tei` | ✅ TEI embeddings | ✅ Implemented | WORKS | Fully functional |
| `docker compose up -d redis` | ✅ Redis cache | ❌ Not configured | MISSING | No redis service |
| **Frontend** |
| Next.js Dashboard | ✅ Real-time visualization | ⚠️ Separate repo | PARTIAL | In intelligence-dashboard-frontend repo |
| Dashboard API endpoint | ✅ `/api/monitoring/sources` | ✅ Implemented | WORKS | Fully functional |
| Dashboard portfolio data | ✅ Real portfolio value (Rp 13.86M) | ✅ Implemented | WORKS | Fully functional |
| **Hermes Integration** |
| Hermes cron jobs | ✅ 7 active jobs | ✅ Implemented | WORKS | Fully operational |
| Hermes script integration | ✅ Scripts in profile directory | ✅ Implemented | WORKS | All scripts in `~/.hermes/profiles/social-politic-lab/scripts/` |
| Telegram delivery | ✅ Automated reporting | ✅ Implemented | WORKS | Jobs deliver to "origin" |
| **Data Storage** |
| ArangoDB collections | ✅ Stock data, news articles | ✅ Implemented | WORKS | 4+ collections active |
| Qdrant vectors | ✅ News embeddings | ❌ Not implemented | MISSING | No Qdrant integration yet |
| Prof Jiang KB | ✅ 130 chunks, 3 categories | ✅ Implemented | WORKS | Fully operational via MCP |
| SQLite staging | ✅ Development staging DB | ❌ Not implemented | MISSING | No SQLite usage |
| **Processing Pipeline** |
| RSS feed collection | ✅ 25+ feeds | ❌ Not implemented | MISSING | Rust collector stubbed |
| HTML cleaning | ✅ Article sanitization | ❌ Not implemented | MISSING | No cleaner module |
| LLM labeling | ✅ Prof Jiang framework | ❌ Not implemented | MISSING | No labeler module |
| Vector embedding | ✅ TEI + Qdrant | ❌ Not implemented | MISSING | No embedder module |
| **Market Data** |
| Yahoo Finance integration | ✅ Live market data | ⚠️ Hardcoded | BROKEN | Placeholder data only |
| Commodity prices | ✅ LME Nickel, Coal, CPO | ⚠️ Hardcoded | BROKEN | Placeholder data only |
| IDX stock prices | ✅ BMRI, BBRI, INCO, etc. | ✅ Implemented | WORKS | Real data via Rust backend |
| BPS inflation data | ✅ Indonesian government stats | ✅ Implemented | WORKS | Real data (3.18%) |
| **Social Intelligence** |
| HackerNews collection | ✅ Tech news | ❌ Not implemented | MISSING | Planned but not started |
| Reddit collection | ✅ Social sentiment | ❌ Not implemented | MISSING | Planned but not started |
| YouTube analysis | ✅ Video content | ❌ Not implemented | MISSING | Planned but not started |
| **Knowledge Base** |
| PDF/EPUB ingestion | 📋 Planned | ❌ Not started | PLANNED | Post-v1.0 feature |
| RAG pipeline | 📋 Planned | ❌ Not started | PLANNED | Post-v1.0 feature |

## Summary Statistics

### Working Features: 14/42 (33%)
- IDX Analyst (5-persona debate) ✅
- Economic data collection ✅  
- Unlimited Indonesian news ✅
- Hermes cron job integration ✅
- Dashboard API ✅
- ArangoDB storage ✅
- TEI embeddings ✅ (infrastructure)
- Health checks ✅

### Partially Working: 6/42 (14%)
- Rust command stubs (collect, clean, label, embed, daemon) ⚠️
- Python BI scraper (needs dependencies) ⚠️
- Docker infrastructure (missing Redis) ⚠️

### Broken: 9/42 (21%)
- Commodity collector (hardcoded data) ❌
- RSS pipeline (not implemented) ❌
- Processing pipeline (not implemented) ❌
- Social media collection (not implemented) ❌

### Missing: 13/42 (31%)
- Redis cache ❌
- Qdrant integration ❌
- SQLite staging ❌
- Knowledge ingestion ❌

## Root Causes

### 1. **Aspirational Architecture**
The repository documents a complete architecture but only parts are implemented. README describes features that don't exist in code.

### 2. **Python-Rust Split Issues**
- Python scripts for data collection but Rust for processing
- No unified data flow between languages
- Different storage patterns (ArangoDB vs placeholder JSON)

### 3. **Infrastructure Complexity**
- Docker compose has partial services
- Missing Redis for caching
- No production deployment configuration

### 4. **Data Flow Breaks**
- Commodity data hardcoded, not from yfinance
- No RSS feed processing pipeline
- No vector embedding pipeline

## Recommendations

### Immediate (Today)
1. **Fix README dishonesty** - Document actual working features
2. **Install Python dependencies** - Fix BI scraper
3. **Choose one vertical** - Market Data vs RSS vs Social Media

### Week 1
1. **Implement yfinance integration** - Replace hardcoded commodity data
2. **Create honest documentation** - ROADMAP.md, ARCHITECTURE.md
3. **Add .gitignore** - Remove generated files from git

### Week 2-4
1. **Complete one Rust pipeline** - Either RSS or labeling
2. **Integrate Qdrant** - Vector storage for embeddings
3. **Add Redis cache** - Rate limiting and deduplication

### Week 5-8  
1. **Social media pipeline** - HackerNews + Reddit
2. **Production deployment** - Monitoring, backup, security
3. **Knowledge ingestion** - PDF/EPUB RAG

---

**Audit Completed:** 2026-08-22T00:26:26Z  
**Next Action:** Action 2 - Fix Root README  
**Priority:** HIGH - Repository documentation is dishonest