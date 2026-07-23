# REQ-001: News Source Resilience & Expansion

**Status**: Draft
**Priority**: P0-P1
**Author**: Principal Data Intelligence
**Date**: 2026-07-23
**Context**: Learnings from WorldMonitor architecture study + RSS feed health audit (31% failure rate discovered)

---

## Problem Statement

Hermes news pipeline has critical resilience gaps:
1. **31% feed failure rate** discovered during audit (9/29 feeds dead, silent failures)
2. **No automated health monitoring** — failures only discovered via manual audit
3. **No circuit breaker** — dead feeds waste 60s timeout per collection cycle
4. **Limited Indonesian coverage** — after removing dead feeds, only 11 Indonesian sources remain
5. **No economic data** — missing BI rate, FRED, CPI that directly impact IDX tickers
6. **User-Agent blocking** — generic bot UA triggers 403 from CDN-protected sites

---

## Requirements

### R1: Feed Health CI Automation
**Priority**: P0

**Description**: Automated daily check of all RSS feed URLs with alerting on degradation.

**Acceptance Criteria**:
- [ ] GitHub Action workflow that curls every feed URL and records HTTP status
- [ ] Fail (or alert) if feed success rate drops below 80%
- [ ] Report shows: feed name, status code, response time, last-success date
- [ ] Runs daily on cron + on-demand manual trigger
- [ ] Results logged to `feed_health` table OR artifact

**Inspiration**: WorldMonitor `feed-validation.yml` — daily cron feed reachability check.

---

### R2: Circuit Breaker for Feed Collection
**Priority**: P1

**Description**: Skip feeds with repeated failures instead of wasting timeout on every collection cycle.

**Acceptance Criteria**:
- [ ] If `consecutive_failures >= 5`, skip feed for 60 minutes (circuit OPEN)
- [ ] After 60 minutes, try ONE request (circuit HALF-OPEN)
- [ ] If success → reset failures, circuit CLOSED
- [ ] If fail → extend skip to 120 minutes, increment backoff
- [ ] Log circuit state transitions: `info!("🔌 Circuit OPEN for {}: {} consecutive failures", name, count)`
- [ ] Health endpoint (`cargo run -- health`) reports circuit-open feeds
- [ ] Max backoff cap: 6 hours

**Implementation notes**:
- State stored in `feed_health` table (add `circuit_open_until TEXT` column)
- Check at start of `collect_all()` before attempting fetch
- Circuit state survives daemon restarts (persisted in SQLite)

**Inspiration**: WorldMonitor client-side circuit breakers per data domain.

---

### R3: RSS Feed Expansion (Phase 1)
**Priority**: P1

**Description**: Add new verified RSS feeds to increase Indonesian and economic news coverage.

**Acceptance Criteria**:
- [ ] Each new feed URL verified accessible (HTTP 200) before adding
- [ ] Each new feed verified to return valid RSS/Atom XML
- [ ] Each feed has `fallback_urls` configured if alternative endpoints exist
- [ ] Feeds added with category tag for filtering

**Candidate feeds to validate and add**:

| Source | URL | Category | Value |
|--------|-----|----------|-------|
| Jakarta Globe | `https://jakartaglobe.id/feed` | ID Business (EN) | English Indonesia business news |
| Katadata | `https://katadata.co.id/rss` | ID Economy | Data-driven economic journalism |
| Reuters Business | `https://feedx.net/rss/reuters-business.xml` | Intl Business | Wire service, pre-market moves |
| Investing.com | `https://www.investing.com/rss/news.rss` | Markets | Market alerts, commodities |
| CNBC ID Market | `https://www.cnbcindonesia.com/market/rss` | ID Market | Subcategory: market-specific |
| CNBC ID Tech | `https://www.cnbcindonesia.com/tech/rss` | ID Tech | Subcategory: tech/digital |
| DW Indonesia | `https://rss.dw.com/xml/rss-id-all` | ID Intl | International news in Bahasa |

- [ ] Final feed count after expansion: >= 30
- [ ] All existing tests still pass after addition
- [ ] First collection cycle with new feeds runs successfully

---

### R4: Seed-Meta Freshness Tracking
**Priority**: P2

**Description**: Track per-source freshness metadata for observability and staleness detection.

**Acceptance Criteria**:
- [ ] New table `source_freshness` with: `source_name TEXT PK, last_article_at TEXT, article_count_24h INTEGER, avg_articles_per_day REAL, updated_at TEXT`
- [ ] Updated after each successful collection cycle
- [ ] `cargo run -- health` reports stale sources (no new articles in 24h+ when avg > 0)
- [ ] Daemon logs freshness summary at end of each cycle

**Inspiration**: WorldMonitor `seed-meta:<key>` pattern — every data write also tracks `{ fetchedAt, recordCount }`.

---

### R5: Feed Category Tagging
**Priority**: P2

**Description**: Add category metadata to FeedConfig for filtering and priority-based processing.

**Acceptance Criteria**:
- [ ] `FeedConfig` struct gets `category: FeedCategory` enum field
- [ ] Categories: `Indonesian`, `InternationalBusiness`, `InternationalGeneral`, `AsiaPacific`, `Market`, `Tech`
- [ ] Collection stats reported per-category: "Indonesian: 8/11 success, Intl: 6/6 success"
- [ ] Future: enable category-based scheduling (Indonesian every 15min, international every 30min)

---

### R6: AI Summary Field in Prof Jiang Output
**Priority**: P2

**Description**: Add a condensed summary field to Prof Jiang labeling output for smaller payloads and better semantic search.

**Acceptance Criteria**:
- [ ] Prof Jiang prompt updated to include `"summary"` field (max 100 words)
- [ ] Summary captures: who did what, market impact, signal
- [ ] Stored in `labeled` table as additional column
- [ ] Used as TEI embedding input instead of full content (better signal-to-noise)
- [ ] Backward-compatible: old labeled records with NULL summary still work
- [ ] Reduces average embedding input from ~500 chars to ~100-150 chars

**Inspiration**: WorldMonitor AI-synthesizes 500+ feeds into briefs — not full-text.

---

### R7: Economic Data Collector (Phase 2 - Design Only)
**Priority**: P3 (design now, implement later)

**Description**: New pipeline module for structured economic indicators that directly correlate to IDX tickers.

**Acceptance Criteria** (design):
- [ ] Architecture doc specifying: data sources, schema, storage, correlation rules
- [ ] `EconomicIndicator` struct defined: `{ source, indicator, value, unit, timestamp, previous_value, change_pct }`
- [ ] Mapping document: which indicators impact which tickers
- [ ] Separate Qdrant collection or SQLite table (TBD — numerical data may not need vector embedding)

**Data sources** (to be validated):
- FRED API (US macro: rates, CPI, GDP)
- Bank Indonesia (BI rate, JIBOR, inflation, forex reserves)
- Yahoo Finance (commodity futures: coal, nickel, palm oil, gold, crude)

**Correlation rules** (initial):
```
BI rate ↑ → Banking sector (BMRI, BBRI, BJTM) negative short-term, positive medium-term
Coal price ↑ → PTBA, ITMG positive
Nickel price ↑ → INCO, ANTM, MDKA positive
CPO price ↑ → AALI, LSIP positive
USD/IDR ↑ (IDR weakening) → Import-heavy negative, Export positive
```

---

## Non-Functional Requirements

### Performance
- Collection cycle (all feeds) must complete in < 5 minutes
- Adding 7 new feeds should not increase cycle time by > 30s
- Circuit breaker skip should save ~60s per dead feed per cycle

### Reliability
- Feed success rate target: >= 80% (with circuit breakers, effective rate should be ~95% of *live* feeds)
- No single feed failure should block pipeline progress
- Daemon must continue operating even if 50% of feeds are unreachable

### Observability
- Every collection cycle logs: total feeds, success count, skipped (circuit open), failed, new articles found
- Health endpoint provides machine-readable status for monitoring
- Freshness tracking enables proactive detection of "silent deaths" (feed returns 200 but content stale)

---

## Dependencies

| Requirement | Depends On | Blocked By |
|-------------|-----------|------------|
| R1 (CI) | GitHub Actions access | None |
| R2 (Circuit breaker) | R5 migration (add column) | None |
| R3 (Feed expansion) | Feed URL validation | None |
| R4 (Freshness) | R5 migration | None |
| R5 (Categories) | None | None |
| R6 (AI summary) | Kiromania/LLM available | None |
| R7 (Economic data) | Design approval | R1-R4 proven stable |

---

## Implementation Order

```
Week 1: R5 (categories) → R2 (circuit breaker) → R3 (feed expansion)
Week 2: R1 (CI automation) → R4 (freshness tracking)
Week 3: R6 (AI summary)
Future: R7 (economic data design + implementation)
```

---

## Success Metrics

| Metric | Before | Target |
|--------|--------|--------|
| Feed success rate | 69% (20/29) | >= 95% of live feeds |
| Feed count | 25 (after cleanup) | >= 30 |
| Collection cycle time | ~3 min | < 3 min (circuit breaker saves time) |
| Silent death detection | Manual (months) | Automated (< 24h) |
| Avg articles ingested per cycle | ~150 | ~200+ |

---

## References

- WorldMonitor architecture study (Learning 39)
- RSS Feed Health Audit (Learnings 36-38)
- Intelligence Patterns steering (data source expansion roadmap)
- Hermes existing: `src/collectors/mod.rs`, `src/storage/mod.rs` (feed_health table)
