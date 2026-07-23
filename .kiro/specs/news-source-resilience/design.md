# News Source Resilience & Intelligence Expansion Design

## Overview

The Hermes news pipeline's RSS collector (`collectors/mod.rs`) lacks resilience mechanisms that cause degraded intelligence coverage and wasted compute when feeds become unreachable. The `collect_all()` function currently iterates all feeds unconditionally, spending up to 60 seconds per dead feed on timeouts. This fix introduces a circuit breaker state machine persisted in SQLite (staging), source freshness tracking to detect "silent deaths", per-category collection stats, and a GitHub Actions CI workflow for proactive feed health monitoring.

Additionally, this design incorporates the **ArangoDB migration** — production already uses ArangoDB as the primary intelligence store (replacing Qdrant). The pipeline flow becomes: RSS → SQLite (staging) → Clean → Label (Prof Jiang) → **ArangoDB** (documents + graph + vector search).

The fix is additive — it wraps the existing `fetch_feed()` call path with circuit state checks and enriches observability without altering the core fetch/retry/fallback logic.

## Storage Architecture: ArangoDB (Primary)

### Why ArangoDB over Qdrant

ArangoDB is already running in production (`/home/ctianm/.hermes/profiles/social-politic-lab/`). The migration rationale:

| Capability | Qdrant (legacy) | ArangoDB (current) |
|-----------|----------------|-------------------|
| Vector search | Native KNN | ArangoSearch APPROX_NEAR |
| Document storage | Payload-on-vector only | Full document collections |
| Graph traversal | N/A | Native (actor→event→ticker) |
| Multi-model query | Separate tool per concern | Single AQL across doc + graph + vector |
| Entity linking | Manual string match | Graph edges (typed relationships) |
| Prof Jiang output | Flat JSON payload | Graph nodes + edges (actors, events, relations) |
| Economic indicators | Not suitable | Time-series documents in collection |

### ArangoDB Collections Design

```
ArangoDB Database: hermes_intelligence
├── DOCUMENT COLLECTIONS
│   ├── articles               # News articles (cleaned + labeled)
│   │   └── { _key, title, content, url, source, published_at, sentiment, news_type, embedding, ... }
│   ├── social_posts           # Social media intelligence
│   │   └── { _key, title, url, source, score, relevance, collected_at, embedding, ... }
│   ├── economic_indicators    # Time-series numerical data (FRED, BI, commodities)
│   │   └── { _key, source, indicator, value, unit, timestamp, change_pct, ... }
│   ├── actors                 # Prof Jiang extracted actors
│   │   └── { _key, name, type, role, incentives, constraints, ... }
│   ├── events                 # Prof Jiang extracted events
│   │   └── { _key, action, trigger, tense, significance, ... }
│   ├── feed_health            # Feed monitoring (mirrors SQLite staging)
│   │   └── { _key, feed_name, consecutive_failures, circuit_state, ... }
│   └── analysis_results       # IDX Analyst debate results + proposals
│       └── { _key, ticker, signal, confidence, proposal, risk, ... }
│
├── EDGE COLLECTIONS (Graph)
│   ├── mentions               # article → actor (who is mentioned)
│   ├── triggers               # event → article (which article triggered event)
│   ├── impacts                # event → ticker (market impact relationship)
│   ├── correlates_with        # article ↔ article (same_event / related)
│   ├── actor_relations        # actor → actor (ally, opponent, authority)
│   └── signal_source          # economic_indicator → ticker (commodity → stock)
│
├── VIEWS (ArangoSearch)
│   ├── articles_vector_view   # Vector search on article embeddings
│   │   └── analyzers: [identity], fields: [embedding (768-dim)]
│   ├── social_vector_view     # Vector search on social post embeddings  
│   │   └── analyzers: [identity], fields: [embedding (768-dim)]
│   └── fulltext_view          # Full-text search across articles + social
│       └── analyzers: [text_en, text_id], fields: [title, content]
│
└── GRAPHS
    └── intelligence_graph     # Named graph connecting all edge collections
        ├── vertex: articles, actors, events, economic_indicators, analysis_results
        └── edges: mentions, triggers, impacts, correlates_with, actor_relations, signal_source
```

### AQL Query Examples (Intelligence Fusion)

**Cross-source correlation** (find news corroborating a market signal):
```aql
// Find articles mentioning PTBA actors within 24h of coal price increase
LET coal_spike = (
  FOR ind IN economic_indicators
    FILTER ind.indicator == "COAL_PRICE" AND ind.change_pct > 3.0
    FILTER ind.timestamp > DATE_SUBTRACT(DATE_NOW(), 24, "hours")
    RETURN ind
)
FOR spike IN coal_spike
  FOR article IN articles
    FILTER article.published_at > DATE_SUBTRACT(spike.timestamp, 4, "hours")
    FILTER article.published_at < DATE_ADD(spike.timestamp, 24, "hours")
    FILTER CONTAINS(article.actors_text, "PTBA") OR CONTAINS(article.actors_text, "coal")
    RETURN { article: article.title, coal_change: spike.change_pct, lag_hours: DATE_DIFF(spike.timestamp, article.published_at, "hours") }
```

**Graph traversal** (find all actors related to a sector event):
```aql
// Traverse from event to all connected actors and their impact on tickers
FOR event IN events
  FILTER event.action LIKE "%export ban%"
  FOR v, e, p IN 1..3 OUTBOUND event triggers, mentions, impacts
    RETURN { path: p.vertices[*].name, edge_types: p.edges[*]._id, impact: v }
```

**Vector similarity** (near-duplicate detection across collections):
```aql
// Find similar articles to a given embedding using ArangoSearch
FOR doc IN articles_vector_view
  SEARCH ANALYZER(
    APPROX_NEAR(doc.embedding, @query_vector, @threshold),
    "identity"
  )
  SORT BM25(doc) DESC
  LIMIT 10
  RETURN { _key: doc._key, title: doc.title, score: BM25(doc) }
```

### Intelligence Fusion via Graph (Fixes Review Issue #1)

The debate engine receives external signals via graph traversal:

```
// When IDX Analyst runs for PTBA:
// 1. Query: what recent events impact PTBA?
FOR v, e IN 1..2 INBOUND "tickers/PTBA" impacts, signal_source
  FILTER v.timestamp > DATE_SUBTRACT(DATE_NOW(), 7, "days")
  RETURN { type: e._type, source: v, strength: e.strength }

// 2. Feed into debate engine as ExternalSignal
// ExternalSignal { source: "coal_price_spike", direction: "positive", confidence: 0.8 }
// → Boosts IDX Guru persona bull_weight for PTBA
```

This resolves the intelligence fusion gap identified in reviews — economic data and event data flow into the debate engine via graph edges with `strength` and `direction` attributes.

### Migration: SQLite Staging → ArangoDB

The pipeline maintains SQLite as **local staging** (collect phase) but ArangoDB is the **intelligence store**:

```
RSS Feeds → SQLite (staging: raw_feeds, cleaned, labeled)
                    ↓ (after labeling)
            ArangoDB (permanent: articles + actors + events + edges)
                    ↓
            ArangoSearch (vector views for similarity queries)
```

SQLite remains for:
- Fast local staging during collection (no network dependency)
- Feed health tracking (circuit breaker state)
- Source freshness metadata
- Crash recovery (staged items can be re-processed)

ArangoDB handles:
- Permanent intelligence storage
- Graph relationships (Prof Jiang actors/events/relations)
- Vector similarity search (near-dup, correlation)
- Economic indicators time-series
- Cross-pipeline queries (news x market x social)

### Infrastructure Update (docker-compose.yml)

```yaml
services:
  arangodb:
    image: arangodb:3.12
    ports:
      - "8529:8529"
    volumes:
      - arangodb_data:/var/lib/arangodb3
    environment:
      - ARANGO_ROOT_PASSWORD=${ARANGO_ROOT_PASSWORD:-hermes}
    restart: unless-stopped

  tei:
    image: ghcr.io/huggingface/text-embeddings-inference:cpu-1.5
    ports:
      - "8082:80"
    volumes:
      - tei_cache:/data
    environment:
      - MODEL_ID=intfloat/multilingual-e5-base
    command: --model-id intfloat/multilingual-e5-base
    restart: unless-stopped
```

TEI stays (for embedding generation). Qdrant removed. ArangoDB replaces it for storage + search.

## Glossary

- **Bug_Condition (C)**: A feed with `consecutive_failures >= 5` is fetched unconditionally, wasting timeout budget; OR a feed returning HTTP 200 with stale content goes undetected; OR feed health has no CI automation; OR stats lack category granularity
- **Property (P)**: Feeds with excessive failures are circuit-broken (skipped), stale feeds are flagged, CI validates reachability daily, and stats report per-category
- **Preservation**: Existing fetch→retry→fallback flow, feed_health success/failure recording, aggregate CollectStats, and Telegram alerting must remain unchanged
- **Circuit State**: Enum `{Closed, Open, HalfOpen}` — tracks whether a feed should be attempted, skipped, or probed
- **FeedCategory**: Enum classifying feeds for per-category stats reporting
- **Source Freshness**: Metadata tracking last article timestamp and article velocity per source to detect silent content death
- **ArangoDB**: Multi-model database (document + graph + vector search) — primary intelligence store replacing Qdrant
- **AQL**: ArangoDB Query Language — enables cross-collection queries spanning documents, graphs, and vector search in single statement
- **Intelligence Graph**: Named graph in ArangoDB connecting articles → actors → events → tickers via typed edges
- **ExternalSignal**: Data from economic/GDELT/ACLED sources that modifies persona confidence in the IDX Analyst debate engine
- **collect_all()**: The function in `src/collectors/mod.rs` that iterates all feeds and stores raw XML
- **feed_health table**: SQLite table tracking per-feed consecutive_failures, last_success, last_error

## Bug Details

### Bug Condition

The bug manifests when the collector enters `collect_all()` and iterates over feeds that have been consistently failing (consecutive_failures >= 5). The system wastes 60 seconds per dead feed per cycle because there is no circuit breaker to skip known-dead feeds. Additionally, feeds that return HTTP 200 but produce no new articles go undetected because there is no freshness tracking.

**Formal Specification:**
```
FUNCTION isBugCondition(input)
  INPUT: input of type (FeedConfig, FeedHealthState)
  OUTPUT: boolean
  
  LET feed = input.0
  LET health = input.1
  
  RETURN (health.consecutive_failures >= 5 AND system_attempts_fetch(feed))
         OR (health.last_http_status == 200 AND health.hours_since_last_article > 24 
             AND health.avg_articles_per_day > 0 AND NOT system_flags_stale(feed))
         OR (NO ci_workflow_exists_for_feed_validation())
         OR (stats_report_lacks_category_breakdown())
END FUNCTION
```

### Examples

- **Circuit Breaker**: Feed "Bloomberg" has `consecutive_failures = 12`. Current behavior: system spends 60s attempting fetch. Expected: skip feed, log circuit OPEN, retry after backoff period
- **Silent Death**: Feed "Kontan" returns HTTP 200 with XML from 3 days ago. Current behavior: reported as healthy. Expected: flagged as "stale" in health check
- **CI Detection**: Feed "Sindonews" returns 403 for 5 days straight. Current behavior: only discovered via manual audit. Expected: daily CI catches it and alerts when success rate < 80%
- **Category Stats**: After collection, user sees "Collected: 20 success, 5 errors". Expected: "Indonesian: 8/11 success | InternationalBusiness: 4/6 success | AsiaPacific: 3/4 success | skipped: 3 (circuit open)"

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- Successful feeds are fetched, parsed, and stored via `insert_raw()` with status "pending"
- `record_feed_success()` resets `consecutive_failures` to 0 on successful fetch
- `record_feed_failure()` increments `consecutive_failures` and records `last_error`
- Fallback URLs are attempted in order when primary URL fails
- Per-request exponential backoff retry via `backoff` crate operates as before
- `SelfHealingMonitor` sends Telegram alerts for feeds with 10+ consecutive failures
- Aggregate `CollectStats` (total success + errors) continues to be reported
- `cargo run -- health` endpoint continues to function

**Scope:**
All inputs where `consecutive_failures < 5` (circuit CLOSED) are completely unaffected — the existing fetch→retry→fallback→record flow proceeds identically. The circuit breaker is purely additive gating logic at the top of the per-feed loop iteration.

## Hypothesized Root Cause

Based on the bug description, the root causes are architectural omissions:

1. **No Circuit State Machine**: `collect_all()` has no gating logic — it unconditionally calls `fetch_feed()` for every feed in `self.feeds`. There is no state machine to track OPEN/HALF-OPEN/CLOSED transitions or skip feeds that are known to be unreachable.

2. **No Freshness Metadata**: The `feed_health` table only tracks failure/success counts and timestamps. There is no tracking of when the *last new article* was seen, making it impossible to detect feeds that return HTTP 200 with stale/unchanged content.

3. **No CI Automation**: There is no GitHub Actions workflow that proactively validates feed reachability. All feed health discovery is reactive (fails during collection) rather than proactive (daily health check independent of collection).

4. **No Category Taxonomy**: `FeedConfig` has `name`, `url`, and `fallback_urls` but no `category` field. Without categories, stats cannot be broken down by source type (Indonesian, International, etc.).

5. **No Backoff Escalation for Circuit**: The existing `ExponentialBackoff` applies per-request retry within a single `fetch_url()` call. There is no *across-cycle* backoff that progressively increases skip duration (60min → 120min → 240min → 6h cap) for persistently failing feeds.

## Correctness Properties

Property 1: Bug Condition - Circuit Breaker Skips Dead Feeds

_For any_ feed where `consecutive_failures >= 5` and `circuit_open_until > now()`, the fixed `collect_all()` function SHALL skip that feed without attempting any HTTP request, log the skip with circuit state info, and include it in the "skipped" count of collection stats.

**Validates: Requirements 2.1, 2.5**

Property 2: Bug Condition - Circuit Half-Open Probe

_For any_ feed where `circuit_open_until <= now()` and `consecutive_failures >= 5` (circuit transitions to HALF-OPEN), the fixed `collect_all()` function SHALL attempt exactly ONE probe request. If the probe succeeds, circuit resets to CLOSED. If it fails, circuit reopens with doubled skip duration (capped at 6 hours).

**Validates: Requirements 2.1, 2.5**

Property 3: Bug Condition - Freshness Staleness Detection

_For any_ feed that returns HTTP 200 but has produced no new articles for 24+ hours (and `avg_articles_per_day > 0`), the health check SHALL flag that source as "stale" and report it in `cargo run -- health` output.

**Validates: Requirements 2.2**

Property 4: Bug Condition - CI Feed Validation

_For any_ configured feed URL, the CI workflow SHALL perform an HTTP HEAD/GET request, record the status code and response time, and alert (fail the workflow) if overall feed success rate drops below 80%.

**Validates: Requirements 2.3**

Property 5: Bug Condition - Per-Category Stats

_For any_ collection cycle completing `collect_all()`, the stats output SHALL include per-category breakdown (success/failure/skipped counts for each `FeedCategory` variant).

**Validates: Requirements 2.4**

Property 6: Preservation - Existing Fetch Flow Unchanged

_For any_ feed where `consecutive_failures < 5` (circuit CLOSED), the fixed `collect_all()` function SHALL produce exactly the same behavior as the original: fetch with retry, try fallbacks, record success/failure in feed_health, store raw XML — with no observable difference.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**

## Fix Implementation

### Changes Required

Assuming our root cause analysis is correct:

**File**: `src/collectors/mod.rs`

**Struct Changes**:
1. **Add `FeedCategory` enum**: `Indonesian`, `InternationalBusiness`, `InternationalGeneral`, `AsiaPacific`, `Market`, `Tech`
2. **Add `category` field to `FeedConfig`**: `pub category: FeedCategory`
3. **Add `CircuitState` enum**: `Closed`, `Open { until: DateTime<Utc>, backoff_secs: i64 }`, `HalfOpen`
4. **Extend `CollectStats`**: Add `skipped: usize`, `per_category: HashMap<FeedCategory, CategoryStats>`

**Function Changes**:
5. **Add `should_attempt_feed()`**: Query `feed_health` for circuit state, return `(bool, CircuitState)`
6. **Modify `collect_all()` loop**: Before `fetch_feed()`, check circuit state. If OPEN → skip. If HALF-OPEN → probe with single attempt.
7. **Add `transition_circuit()`**: Handle state transitions (CLOSED→OPEN, HALF-OPEN→CLOSED, HALF-OPEN→OPEN with doubled backoff)
8. **Add `update_circuit_state()`**: Persist `circuit_open_until` and `backoff_secs` to feed_health table

---

**File**: `src/storage/mod.rs`

**Schema Migration**:
1. **Alter `feed_health` table**: Add columns `circuit_open_until TEXT`, `backoff_secs INTEGER DEFAULT 3600`, `category TEXT`
2. **Create `source_freshness` table**: `source_name TEXT PK, last_article_at TEXT, article_count_24h INTEGER, avg_articles_per_day REAL, updated_at TEXT`
3. **Add `get_circuit_state()`**: Query circuit_open_until for a given feed
4. **Add `set_circuit_open()`**: Set circuit_open_until = now + backoff_secs
5. **Add `reset_circuit()`**: Set circuit_open_until = NULL, backoff_secs = 3600
6. **Add `update_source_freshness()`**: Upsert freshness metadata after successful collection
7. **Add `get_stale_sources()`**: Query sources with no articles in 24h but avg > 0

---

**File**: `src/health/mod.rs`

**Function Changes**:
1. **Add circuit-open feeds to health report**: Query feeds where `circuit_open_until > now`
2. **Add stale source report**: Query `source_freshness` for sources with `last_article_at` > 24h ago
3. **Add per-category summary**: Aggregate feed_health stats by category

---

**New File**: `.github/workflows/feed-health.yml`

**CI Workflow**:
1. Daily cron (06:00 UTC) + manual trigger
2. For each configured feed URL: HTTP GET with 30s timeout
3. Record: feed name, status code, response time, last-success date
4. Calculate overall success rate
5. Fail workflow if success rate < 80%
6. Upload results as artifact

### Schema Migration SQL

```sql
-- Add circuit breaker columns to feed_health
ALTER TABLE feed_health ADD COLUMN circuit_open_until TEXT;
ALTER TABLE feed_health ADD COLUMN backoff_secs INTEGER DEFAULT 3600;
ALTER TABLE feed_health ADD COLUMN category TEXT;

-- Source freshness tracking
CREATE TABLE IF NOT EXISTS source_freshness (
    source_name TEXT PRIMARY KEY,
    last_article_at TEXT,
    article_count_24h INTEGER DEFAULT 0,
    avg_articles_per_day REAL DEFAULT 0.0,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_freshness_stale 
ON source_freshness(last_article_at);
```

### Circuit Breaker State Machine (Pseudocode)

```
FUNCTION determine_circuit_state(feed_name, db) -> CircuitState
  LET health = db.get_feed_health(feed_name)
  
  IF health.consecutive_failures < 5 THEN
    RETURN CircuitState::Closed
  
  IF health.circuit_open_until IS NULL THEN
    // First time hitting threshold - open circuit
    db.set_circuit_open(feed_name, now() + 3600s, 3600)
    RETURN CircuitState::Open { until: now() + 3600s }
  
  IF now() >= health.circuit_open_until THEN
    RETURN CircuitState::HalfOpen
  ELSE
    RETURN CircuitState::Open { until: health.circuit_open_until }

FUNCTION handle_half_open_result(feed_name, success, db)
  IF success THEN
    db.reset_circuit(feed_name)  // -> CLOSED
    info!("🔌 Circuit CLOSED for {}: probe succeeded", feed_name)
  ELSE
    LET new_backoff = min(health.backoff_secs * 2, 21600)  // cap at 6h
    db.set_circuit_open(feed_name, now() + new_backoff, new_backoff)
    info!("🔌 Circuit OPEN for {}: probe failed, next retry in {}s", feed_name, new_backoff)
```

### FeedCategory Enum (Design Reference)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedCategory {
    Indonesian,
    InternationalBusiness,
    InternationalGeneral,
    AsiaPacific,
    Market,
    Tech,
}

impl std::fmt::Display for FeedCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Indonesian => write!(f, "Indonesian"),
            Self::InternationalBusiness => write!(f, "Intl Business"),
            Self::InternationalGeneral => write!(f, "Intl General"),
            Self::AsiaPacific => write!(f, "Asia Pacific"),
            Self::Market => write!(f, "Market"),
            Self::Tech => write!(f, "Tech"),
        }
    }
}
```

### Modified collect_all() Flow (Pseudocode)

```
FUNCTION collect_all(self, db) -> Result<CollectStats>
  LET mut stats = CollectStats::default()
  
  FOR feed IN self.feeds DO
    LET circuit = determine_circuit_state(feed.name, db)
    
    MATCH circuit
      CircuitState::Open { until } =>
        stats.skipped += 1
        stats.per_category[feed.category].skipped += 1
        info!("⏭️ Skipping {} (circuit OPEN until {})", feed.name, until)
        CONTINUE
      
      CircuitState::HalfOpen =>
        info!("🔌 Circuit HALF-OPEN for {}: probing...", feed.name)
        // Fall through to fetch — but only ONE attempt
      
      CircuitState::Closed =>
        // Normal fetch path (unchanged)
    
    MATCH self.fetch_feed(feed).await
      Ok(raw_content) =>
        db.insert_raw(raw_feed)?
        db.record_feed_success(feed.name)?
        IF circuit == HalfOpen THEN
          handle_half_open_result(feed.name, true, db)
        stats.success += 1
        stats.per_category[feed.category].success += 1
      
      Err(e) =>
        db.record_feed_failure(feed.name, e)?
        IF circuit == HalfOpen THEN
          handle_half_open_result(feed.name, false, db)
        ELSE IF health.consecutive_failures >= 5 THEN
          // Threshold just crossed — open circuit
          transition_circuit(feed.name, CircuitState::Open, db)
        stats.errors += 1
        stats.per_category[feed.category].errors += 1
  
  RETURN Ok(stats)
```

## Testing Strategy

### Validation Approach

The testing strategy follows a two-phase approach: first, surface counterexamples that demonstrate the bug on unfixed code, then verify the fix works correctly and preserves existing behavior.

### Exploratory Bug Condition Checking

**Goal**: Surface counterexamples that demonstrate the bug BEFORE implementing the fix. Confirm or refute the root cause analysis. If we refute, we will need to re-hypothesize.

**Test Plan**: Write tests that call `collect_all()` with a mock feed list containing feeds with `consecutive_failures >= 5` in the database. Observe that the unfixed code attempts to fetch ALL feeds (including dead ones) and wastes timeout. Run these on the UNFIXED code to observe failures.

**Test Cases**:
1. **Dead Feed Still Fetched**: Set feed_health.consecutive_failures = 10 for "Bloomberg", call collect_all(), observe it still attempts HTTP request (will timeout — 60s wasted on unfixed code)
2. **No Stale Detection**: Insert feed_health with last_success = 3 days ago but consecutive_failures = 0 (HTTP 200, stale content), call health check, observe no "stale" warning (will pass silently on unfixed code)
3. **No Category Breakdown**: Call collect_all(), observe CollectStats only has `success` and `errors` fields — no per-category data (will show only aggregate on unfixed code)
4. **No CI Workflow**: Check `.github/workflows/` for feed health validation — none exists (will find nothing on unfixed code)

**Expected Counterexamples**:
- `collect_all()` spends 60s on feeds with 10+ failures instead of skipping
- Health check reports no stale feeds even when content is days old
- Possible causes: no circuit state machine, no freshness table, no category enum

### Fix Checking

**Goal**: Verify that for all inputs where the bug condition holds, the fixed function produces the expected behavior.

**Pseudocode:**
```
FOR ALL feed WHERE consecutive_failures >= 5 AND circuit_open_until > now() DO
  result := collect_all_fixed(feeds_including(feed))
  ASSERT feed was NOT fetched (no HTTP request)
  ASSERT result.skipped includes feed
  ASSERT circuit state logged
END FOR

FOR ALL feed WHERE circuit_open_until <= now() AND consecutive_failures >= 5 DO
  result := collect_all_fixed(feeds_including(feed))
  ASSERT exactly ONE probe request was made
  IF probe_success THEN
    ASSERT circuit state = CLOSED
    ASSERT consecutive_failures = 0
  ELSE
    ASSERT circuit state = OPEN
    ASSERT new backoff = min(old_backoff * 2, 21600)
END FOR
```

### Preservation Checking

**Goal**: Verify that for all inputs where the bug condition does NOT hold (consecutive_failures < 5), the fixed function produces the same result as the original function.

**Pseudocode:**
```
FOR ALL feed WHERE consecutive_failures < 5 DO
  ASSERT collect_all_original(feed) = collect_all_fixed(feed)
  // Same HTTP requests made
  // Same feed_health updates (success/failure)
  // Same raw_feeds insertions
  // Same CollectStats.success and .errors counts
END FOR
```

**Testing Approach**: Property-based testing is recommended for preservation checking because:
- It generates many test cases automatically across the input domain (various failure counts 0-4)
- It catches edge cases (consecutive_failures = 4 → must NOT trigger circuit breaker)
- It provides strong guarantees that behavior is unchanged for all non-buggy inputs

**Test Plan**: Observe behavior on UNFIXED code first for feeds with < 5 failures, then write property-based tests capturing that exact behavior persists after the fix.

**Test Cases**:
1. **Fetch Preserved for Healthy Feeds**: Feeds with consecutive_failures = 0..4 are fetched normally with full retry/fallback
2. **Success Recording Preserved**: On successful fetch, record_feed_success() still called, resetting failures to 0
3. **Failure Recording Preserved**: On failed fetch, record_feed_failure() still increments counter and records error
4. **Fallback URL Preserved**: When primary fails and fallback_urls exist, fallbacks still attempted in order
5. **Aggregate Stats Preserved**: CollectStats.success + .errors still accurate (in addition to new category stats)
6. **Telegram Alerts Preserved**: SelfHealingMonitor still sends alerts for feeds with 10+ failures

### Unit Tests

- Test `determine_circuit_state()` with various consecutive_failure counts and circuit_open_until values
- Test `FeedCategory::from_str()` and Display impl for all variants
- Test `handle_half_open_result()` for both success and failure paths
- Test backoff doubling with 6-hour cap (3600 → 7200 → 14400 → 21600 → 21600)
- Test schema migration applies cleanly on existing database
- Test `get_stale_sources()` returns correct feeds based on freshness threshold

### Property-Based Tests

- Generate random `consecutive_failures` values (0..100) and verify circuit state transitions are correct
- Generate random `backoff_secs` values and verify cap at 21600 (6 hours) is never exceeded
- Generate random feed configurations and verify feeds with failures < 5 are ALWAYS fetched (preservation)
- Generate random timestamps for `circuit_open_until` and verify HALF-OPEN transitions at exact boundary

### Integration Tests

- Test full `collect_all()` cycle with mix of healthy, circuit-open, and half-open feeds against mock HTTP server
- Test circuit state persistence across simulated daemon restarts (write state, re-read, verify)
- Test freshness tracking updates after successful collection cycle
- Test per-category stats aggregation with feeds from multiple categories
- Test GitHub Actions workflow YAML is valid (lint with `actionlint`)

## Data Source Expansion Design

### Overview

In addition to the resilience fixes, this spec covers expanding data sources from 25 to 30+ feeds (Phase 1) and designing the architecture for structured economic data ingestion (Phase 2). This directly addresses the coverage gap caused by removing 5 dead feeds.

### Architecture Patterns

Two patterns exist for adding data sources:

**Pattern A: RSS Feed (zero-effort)** — Add `FeedConfig` entry to `collectors/mod.rs`. Feed immediately enters existing pipeline: collect → clean → label → embed. No new modules needed.

**Pattern B: Custom API Collector** — Create new module under `src/` with dedicated client, models, and storage. For data that isn't RSS (REST APIs, WebSocket, structured numerical data).

### Phase 1: RSS Feed Expansion

**Goal**: Recover coverage lost from dead feed removal. Add 5-7 verified RSS feeds.

**New Feeds to Add** (Pattern A):

| Feed | URL | Category | Fallback |
|------|-----|----------|----------|
| Jakarta Globe | `https://jakartaglobe.id/feed` | Indonesian | None |
| Katadata | `https://katadata.co.id/rss` | Indonesian | None |
| Reuters Business | `https://feedx.net/rss/reuters-business.xml` | InternationalBusiness | None |
| Investing.com | `https://www.investing.com/rss/news.rss` | Market | None |
| CNBC ID Market | `https://www.cnbcindonesia.com/market/rss` | Indonesian | Main CNBC ID feed |
| CNBC ID Tech | `https://www.cnbcindonesia.com/tech/rss` | Tech | Main CNBC ID feed |
| DW Indonesia | `https://rss.dw.com/xml/rss-id-all` | Indonesian | None |

**Validation Criteria** (each feed must pass ALL before adding):
1. HTTP GET returns 200 with timeout < 10s
2. Response body is valid RSS 2.0 or Atom XML
3. Feed contains >= 5 articles from last 24h
4. Content is not duplicate of existing source (check titles)
5. Feed has been stable for >= 7 days (not just a one-time check)

**Implementation** (in `collectors/mod.rs`):
```rust
// Add to feeds vec in RssCollector::new()
FeedConfig { name: "Jakarta Globe".into(), url: "https://jakartaglobe.id/feed".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
FeedConfig { name: "Katadata".into(), url: "https://katadata.co.id/rss".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
FeedConfig { name: "Reuters Business".into(), url: "https://feedx.net/rss/reuters-business.xml".into(), fallback_urls: vec![], category: FeedCategory::InternationalBusiness },
FeedConfig { name: "Investing.com".into(), url: "https://www.investing.com/rss/news.rss".into(), fallback_urls: vec![], category: FeedCategory::Market },
FeedConfig { name: "CNBC ID Market".into(), url: "https://www.cnbcindonesia.com/market/rss".into(), fallback_urls: vec!["https://www.cnbcindonesia.com/rss".into()], category: FeedCategory::Indonesian },
FeedConfig { name: "CNBC ID Tech".into(), url: "https://www.cnbcindonesia.com/tech/rss".into(), fallback_urls: vec!["https://www.cnbcindonesia.com/rss".into()], category: FeedCategory::Tech },
FeedConfig { name: "DW Indonesia".into(), url: "https://rss.dw.com/xml/rss-id-all".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
```

**Expected Impact**:
- Feed count: 25 → 32
- Additional articles per cycle: +200-400
- Additional SQLite storage: ~400KB/day
- Additional Qdrant vectors: ~200/day
- Collection cycle time increase: < 30s (7 feeds x ~3s avg)

### Phase 2: Economic Data Module (In-Scope — Full Implementation)

**Goal**: Bridge between news sentiment and hard economic data. Structured numerical indicators that directly correlate to IDX tickers.

**Module Structure**:
```
src/economic/
├── mod.rs                   # EconomicCollector struct + CLI integration
├── fred.rs                  # FRED API client (US macro indicators)
├── bank_indonesia.rs        # BI API client (Indonesian rates/inflation)
├── yahoo_commodities.rs     # Yahoo Finance commodity futures (replace static Python collector)
├── coingecko.rs             # CoinGecko crypto market data
├── models.rs                # EconomicIndicator, EconomicSource enums
└── storage.rs               # SQLite table + queries for indicators
```

**Data Model**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicIndicator {
    pub source: EconomicSource,       // FRED, BankIndonesia, YahooFinance, CoinGecko
    pub indicator: String,            // "FED_FUNDS_RATE", "BI_RATE", "COAL_PRICE"
    pub value: f64,
    pub unit: String,                 // "percent", "IDR/USD", "USD/tonne"
    pub timestamp: DateTime<Utc>,
    pub previous_value: Option<f64>,
    pub change_pct: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EconomicSource {
    Fred,
    BankIndonesia,
    YahooFinance,
    CoinGecko,
}
```

**Storage**: SQLite table (NOT Qdrant — numerical data doesn't need vector embedding):
```sql
CREATE TABLE IF NOT EXISTS economic_indicators (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL,
    indicator TEXT NOT NULL,
    value REAL NOT NULL,
    unit TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    previous_value REAL,
    change_pct REAL,
    fetched_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source, indicator, timestamp)
);

CREATE INDEX IF NOT EXISTS idx_econ_source_indicator 
ON economic_indicators(source, indicator, timestamp DESC);
```

#### Tier 2A: FRED API (Federal Reserve Economic Data)

**Endpoint**: `https://api.stlouisfed.org/fred/series/observations`
**Auth**: Free API key (register at fred.stlouisfed.org)
**Rate Limit**: 120 requests/minute

**Indicators to collect**:

| Series ID | Indicator | Frequency | IDX Impact |
|-----------|-----------|-----------|-----------|
| FEDFUNDS | Fed Funds Rate | Monthly | All ID equities (capital flow) |
| CPIAUCSL | US CPI | Monthly | Global risk sentiment |
| UNRATE | US Unemployment | Monthly | Global demand proxy |
| GDP | US GDP Growth | Quarterly | Global growth indicator |
| DGS10 | 10-Year Treasury Yield | Daily | Emerging market flow pressure |
| DTWEXBGS | Trade-Weighted USD | Daily | IDR pressure indicator |

**Implementation** (`src/economic/fred.rs`):
```rust
pub struct FredClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl FredClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("FRED_API_KEY")
            .context("FRED_API_KEY not set")?;
        // ...
    }

    pub async fn fetch_series(&self, series_id: &str, limit: u32) -> Result<Vec<EconomicIndicator>> {
        let url = format!(
            "{}/fred/series/observations?series_id={}&api_key={}&file_type=json&sort_order=desc&limit={}",
            self.base_url, series_id, self.api_key, limit
        );
        // ...
    }
}
```

#### Tier 2B: Bank Indonesia API

**Endpoint**: `https://dataapi.bi.go.id/` (official BI data portal)
**Auth**: Free, no API key (public data)
**Fallback**: Scrape from `https://www.bi.go.id/id/statistik/informasi-kurs/` for FX rates

**Indicators to collect**:

| Indicator | Source | Frequency | IDX Impact |
|-----------|--------|-----------|-----------|
| BI-Rate (7-Day Repo) | BI API | Monthly (RDG decision) | Banking sector (BMRI, BBRI, BJTM) |
| JIBOR (Jakarta Interbank Offered Rate) | BI API | Daily | Short-term lending cost |
| Inflation (CPI-YoY) | BI API | Monthly | Consumer sector, BI rate trajectory |
| Forex Reserves | BI API | Monthly | IDR stability confidence |
| USD/IDR Exchange Rate | BI API | Daily | Import/export sector balance |
| Foreign Capital Flow (SBN) | BI API | Weekly | Portfolio flow indicator |

#### Tier 2C: Yahoo Finance Commodity Futures (Replace Python collector)

**Endpoint**: `https://query1.finance.yahoo.com/v8/finance/chart/{symbol}`
**Auth**: No API key required
**Rate Limit**: Stagger 150ms between requests

**Symbols to collect** (Indonesian strategic commodities):

| Symbol | Commodity | Unit | IDX Tickers |
|--------|-----------|------|-------------|
| GC=F | Gold | USD/oz | ANTM (gold mining) |
| NI=F | Nickel | USD/tonne | INCO, ANTM, MDKA |
| CL=F | Crude Oil (WTI) | USD/barrel | Energy sector |
| BZ=F | Crude Oil (Brent) | USD/barrel | Energy sector |
| FCPO=F | Palm Oil (CPO) | MYR/tonne | AALI, LSIP, SIMP |
| QC=F | Thermal Coal (Newcastle) | USD/tonne | PTBA, ITMG, ADRO |
| NG=F | Natural Gas | USD/MMBtu | Energy sector |
| HG=F | Copper | USD/lb | Mining general |
| TIN=F | Tin | USD/tonne | TINS |

**Implementation** (`src/economic/yahoo_commodities.rs`):
```rust
pub struct YahooCommodityClient {
    client: Client,
    stagger_ms: u64,  // 150ms between requests
}

impl YahooCommodityClient {
    pub async fn fetch_commodity(&self, symbol: &str) -> Result<EconomicIndicator> {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=5d",
            symbol
        );
        tokio::time::sleep(Duration::from_millis(self.stagger_ms)).await;
        // ... fetch and parse
    }

    pub async fn fetch_all_commodities(&self) -> Result<Vec<EconomicIndicator>> {
        let symbols = ["GC=F", "NI=F", "CL=F", "BZ=F", "FCPO=F", "QC=F", "NG=F", "HG=F", "TIN=F"];
        let mut results = Vec::new();
        for symbol in symbols {
            match self.fetch_commodity(symbol).await {
                Ok(indicator) => results.push(indicator),
                Err(e) => warn!("⚠️ Failed to fetch {}: {}", symbol, e),
            }
        }
        Ok(results)
    }
}
```

#### Tier 2D: CoinGecko (Crypto Market Data)

**Endpoint**: `https://api.coingecko.com/api/v3/`
**Auth**: Free tier (no key, 10-30 calls/minute)
**Rate Limit**: 10 calls/minute on free tier

**Assets to collect**:

| ID | Asset | Why |
|----|-------|-----|
| bitcoin | BTC | Global risk-on/off barometer |
| ethereum | ETH | DeFi/tech sentiment |
| tether | USDT | Stablecoin flow indicator |
| binancecoin | BNB | SEA crypto adoption |
| ripple | XRP | Cross-border payment signal |

**Implementation** (`src/economic/coingecko.rs`):
```rust
pub struct CoinGeckoClient {
    client: Client,
}

impl CoinGeckoClient {
    pub async fn fetch_prices(&self, ids: &[&str]) -> Result<Vec<EconomicIndicator>> {
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true",
            ids.join(",")
        );
        // ... parse response into EconomicIndicator vec
    }
}
```

#### Tier 2E: GDELT (Global Database of Events, Language, and Tone)

**Endpoint**: `https://api.gdeltproject.org/api/v2/doc/doc`
**Auth**: Free, no API key
**Rate Limit**: Generous (no published limit for reasonable use)

**Query Strategy**:
- Filter by: `sourcecountry:ID` (Indonesian sources) OR `theme:ECON_*` OR `theme:ENV_*`
- Time range: last 24 hours
- Fields: title, url, tone, themes, locations, sourceCountry
- Output: JSON

**Implementation** (`src/economic/gdelt.rs`):
```rust
pub struct GdeltClient {
    client: Client,
}

impl GdeltClient {
    pub async fn query_events(&self, query: &str, timespan: &str) -> Result<Vec<GdeltEvent>> {
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=artlist&timespan={}&format=json",
            urlencoding::encode(query), timespan
        );
        // ... parse response
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdeltEvent {
    pub title: String,
    pub url: String,
    pub tone: f64,              // -10 to +10
    pub goldstein_scale: f64,   // -10 to +10 (event impact)
    pub themes: Vec<String>,
    pub source_country: String,
    pub date: DateTime<Utc>,
}
```

#### Tier 2F: EventRegistry (Pre-labeled News Events)

**Endpoint**: `https://eventregistry.org/api/v1/`
**Auth**: Free tier (200 requests/day, 50 results/request)
**Rate Limit**: 1 request/second

**Value**: Events come PRE-LABELED with entities, categories, and sentiment — complements Prof Jiang.

### Phase 3: Geopolitical & Environmental Intelligence (Full Design)

#### Tier 3A: ACLED (Armed Conflict Location & Event Data)

**Endpoint**: `https://acleddata.com/api/acled/`  
**Auth**: Free for research (register for API key)
**Rate Limit**: 500 calls/day

**Query Parameters**:
- `iso=360` (Indonesia country code)
- `event_date_after=YYYY-MM-DD` (last 30 days)
- `fields`: event_type, sub_event_type, actor1, actor2, location, latitude, longitude, fatalities, notes

**Data Model**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictEvent {
    pub event_type: String,        // "Battles", "Violence against civilians", "Protests"
    pub sub_event_type: String,    // "Armed clash", "Mob violence", "Peaceful protest"
    pub actor1: String,
    pub actor2: Option<String>,
    pub location: String,
    pub latitude: f64,
    pub longitude: f64,
    pub fatalities: u32,
    pub notes: String,
    pub date: DateTime<Utc>,
    pub source: String,
}
```

**IDX Correlation**:
- Conflict in Sulawesi/Papua mining regions → INCO, ANTM supply risk
- Protests near industrial zones → manufacturing sector risk
- Separatist activity → country risk premium increase

#### Tier 3B: FIRMS (Fire Information for Resource Management System — NASA)

**Endpoint**: `https://firms.modaps.eosdis.nasa.gov/api/area/csv/{api_key}/VIIRS_SNPP_NRT/`
**Auth**: Free NASA Earthdata API key
**Rate Limit**: Generous for reasonable use

**Query**: Indonesia bounding box (94.5E - 141E, 11S - 6N), last 24h

**Data Model**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireHotspot {
    pub latitude: f64,
    pub longitude: f64,
    pub brightness: f64,       // Fire intensity
    pub confidence: String,    // "low", "nominal", "high"
    pub frp: f64,              // Fire Radiative Power (MW)
    pub acq_date: String,
    pub acq_time: String,
    pub satellite: String,     // "N" (Suomi NPP) or "1" (NOAA-20)
}
```

**IDX Correlation**:
- Fire density in Sumatra/Kalimantan → CPO supply disruption (AALI, LSIP, SIMP)
- Haze events → cross-border tension with Malaysia/Singapore
- Industrial fires near mining → operational disruption risk

#### Tier 3C: IMF Data (International Monetary Fund)

**Endpoint**: `https://dataservices.imf.org/REST/SDMX_JSON.svc/`
**Auth**: Free, no API key
**Rate Limit**: Reasonable use

**Indicators**:
- GDP growth rate (Indonesia + trading partners)
- Current account balance
- Government debt-to-GDP
- FDI inflows
- Trade balance

#### Tier 3D: AIS Maritime (Ship Tracking)

**Protocol**: WebSocket stream from AIS providers
**Auth**: Varies by provider (some free tiers available)
**Value**: Supply chain visibility — track coal/nickel/CPO export vessels

**Concept**:
- Track bulk carriers leaving Indonesian ports (Balikpapan, Samarinda, Belawan)
- Commodity export volume indicator
- Vessel congestion at ports → supply chain bottleneck detection

### Correlation Rules (Complete Matrix)

```
┌──────────────────────┬──────────────────────────────────────────────────────────┐
│ Data Source          │ IDX Ticker Impact                                        │
├──────────────────────┼──────────────────────────────────────────────────────────┤
│ BI Rate ↑            │ BMRI/BBRI/BJTM: -short/+medium (NIM expansion)          │
│ BI Rate ↓            │ Banking: +short, Property: +strong                       │
│ Coal Price ↑         │ PTBA/ITMG/ADRO: +strong                                 │
│ Nickel Price ↑       │ INCO/ANTM/MDKA: +strong                                 │
│ CPO Price ↑          │ AALI/LSIP/SIMP: +strong                                 │
│ Gold Price ↑         │ ANTM: +moderate, Risk-off signal                         │
│ USD/IDR ↑ (weaken)   │ Exporters: +, Importers: -, Banking: neutral             │
│ Fed Rate ↑           │ All ID: -moderate (capital outflow pressure)              │
│ US CPI ↑             │ All ID: -mild (global risk-off)                          │
│ 10Y Treasury ↑       │ All ID: -moderate (EM flow reversal)                     │
│ BTC ↑                │ Risk-on proxy: +mild for growth stocks                    │
│ BTC ↓ sharp          │ Risk-off signal: -mild for speculative                    │
│ GDELT tone < -5      │ Sector-specific: - (depends on actors involved)           │
│ ACLED conflict ↑     │ Mining region: INCO/ANTM -, Country risk: all -mild       │
│ FIRMS fires ↑ (Kali) │ AALI/LSIP: -moderate (supply disruption)                 │
│ FIRMS fires ↑ (Sum)  │ CPO sector: -moderate, haze: cross-border tension         │
│ Forex reserves ↓     │ All ID: -mild (IDR defense capacity weakening)            │
│ Credit growth ↑      │ Banking: +strong (volume), Consumer: +moderate            │
│ SBN foreign flow ↑   │ All ID: +mild (confidence signal)                         │
│ SBN foreign flow ↓   │ All ID: -moderate (capital flight signal)                 │
└──────────────────────┴──────────────────────────────────────────────────────────┘
```

**Collection Schedule**:

| Source | Frequency | Time |
|--------|-----------|------|
| RSS feeds (all) | Every 15 min | 24/7 |
| Yahoo Commodities | Every 30 min | Market hours (Mon-Fri) |
| CoinGecko | Every 60 min | 24/7 |
| FRED | Daily | 07:00 UTC |
| Bank Indonesia | Daily (rates), Monthly (macro) | After BI publish |
| GDELT | Every 2 hours | 24/7 |
| ACLED | Daily | 06:00 UTC |
| FIRMS (NASA) | Every 6 hours | 24/7 |
| EventRegistry | Every 4 hours | 24/7 |

### CLI Integration

Add new commands to `main.rs`:
```
news-collector economic          # Run all economic collectors once
news-collector economic fred     # FRED only
news-collector economic bi       # Bank Indonesia only  
news-collector economic commodity # Yahoo commodity futures only
news-collector economic crypto   # CoinGecko only
news-collector economic daemon   # Run on schedule
```

### Data Size Impact Summary

| Addition | Records/day | SQLite Impact | ArangoDB Impact |
|----------|-------------|---------------|-----------------|
| 7 new RSS feeds | +200-400 articles | +400KB/day (staging) | +200 docs + graph edges/day |
| FRED (US macro) | +10-20 indicators | None (direct to ArangoDB) | +20 docs/day |
| Bank Indonesia | +5-10 indicators | None (direct to ArangoDB) | +10 docs/day |
| Yahoo Commodities | +9 prices/cycle | None (direct to ArangoDB) | +270 docs/day (9 x 30min) |
| CoinGecko | +5 prices/cycle | None (direct to ArangoDB) | +120 docs/day (5 x 60min) |
| GDELT (future) | +500-2000 events | None (direct to ArangoDB) | +2000 docs + edges/day |
| ACLED (future) | +10-50 events | None (direct to ArangoDB) | +50 docs + edges/day |
| FIRMS (future) | +100-500 hotspots | None (direct to ArangoDB) | +500 docs/day |

All additions remain comfortable within single-instance ArangoDB Community Edition. Graph edges grow proportionally to articles x actors x events extracted.
