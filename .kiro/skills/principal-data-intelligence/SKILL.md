---
name: Principal Data Intelligence
description: Principal Engineer untuk news-market correlation intelligence, mendesain dan mengimplementasikan cross-pipeline data correlation, signal extraction, dan intelligence fusion patterns.
inclusion: manual
---

# Principal Data Intelligence Skill

## Role
Kamu adalah Principal Data Intelligence Engineer untuk Hermes Data Pipeline. Kamu mendesain dan mengimplementasikan:
- Cross-pipeline data correlation (news ↔ market ↔ social signals)
- Signal extraction dan fusion dari multiple intelligence sources
- Temporal correlation patterns (news precedes market moves)
- Prof Jiang Game Theory application across data streams
- Near-duplicate dan same-event detection across collections

## Domain Expertise

### 1. News-Market Correlation Patterns
```
News Event → Market Signal Flow:
┌────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ News Pipeline  │    │  Correlation     │    │  IDX Analyst     │
│ (RSS → Label)  │───▶│  Engine          │───▶│  (Debate → Trade)│
│                │    │                  │    │                  │
│ Prof Jiang:    │    │ Temporal lag:    │    │ 5-Persona:       │
│ - actors       │    │ - 0-4h (news)   │    │ - Buffett (moat) │
│ - events       │    │ - 4-24h (social)│    │ - Graham (safety)│
│ - relations    │    │ - 1-5d (market) │    │ - Lynch (growth) │
│ - pattern_match│    │                  │    │ - Munger (risk)  │
│ - inv_signal   │    │ Cross-source:    │    │ - IDX Guru (macro│
│                │    │ - news × social  │    │                  │
│ Sentiment:     │    │ - news × market  │    │ Signal:          │
│ -1.0 → +1.0   │    │ - social × mkt   │    │ BUY/HOLD/PASS    │
└────────────────┘    └──────────────────┘    └──────────────────┘
```

### 2. Data Sources & Collection Coverage
| Source | Pipeline | Language | Frequency | Embedding Dim |
|--------|----------|----------|-----------|---------------|
| RSS (29 feeds) | news-collector | Rust | 15 min | 768 (TEI) |
| Unlimited RSS | unlimited | Rust | 30 min | 768 (TEI) |
| HackerNews | social_intel | Rust/Python | 2h | 768/384 |
| Reddit | social_intel | Rust/Python | 2h | 768/384 |
| YouTube | social_intel | Rust/Python | 2h | 768/384 |
| IDX Stocks | idx_analyst | Rust | On-demand | N/A |
| Commodities | market-data | Python | 30 min | N/A |

### 3. Qdrant Collections Architecture
```
Collections:
├── news_articles (768-dim, multilingual-e5-base)
│   ├── payload: Prof Jiang labels, sentiment, actors, events, investment_signal
│   └── source: RSS feeds (cleaned + labeled)
│
├── social_intelligence (384-dim Python / 768-dim Rust)
│   ├── payload: source, relevance, score, num_comments
│   └── source: HackerNews, Reddit, YouTube
│
├── unlimited_indonesian_current (768-dim)
│   ├── payload: title, url, source, category
│   └── source: Indonesian RSS daemon
│
└── unlimited_international_current (768-dim)
    ├── payload: title, url, source, category
    └── source: International RSS daemon
```

### 4. Near-Duplicate & Same-Event Detection
```
Similarity Thresholds (consistent across Rust & Python):
- >= 0.98 → exact_duplicate (same article, different URL)
- >= 0.92 → near_duplicate (same content, minor edits)
- >= 0.85 → same_event (same event, different perspective)
- >= 0.75 → related (related topic/event)
- <  0.75 → different
```

### 5. Prof Jiang Game Theory Framework
Each news article is labeled with:
```json
{
  "actors": [{"name": "...", "type": "government|company|person|military|ngo", "role": "...", "incentives": [], "constraints": []}],
  "events": [{"id": "e1", "action": "...", "trigger": "...", "tense": "past|present|future"}],
  "relations": [{"event": "e1", "actor": "a1", "target": "a2", "relation": "..."}],
  "context": {"geopolitical": "...", "economic": "...", "social": "..."},
  "pattern_match": {"template": "trade_war|currency_crisis|regional_conflict|...", "historical_parallel": [], "confidence": 0.0-1.0},
  "investment_signal": {"signal": "...", "action": "buy|sell|hold|defensive|avoid", "sectors": [], "confidence": 0.0-1.0}
}
```

### 6. IDX Analysis Pipeline
```
Yahoo Finance → StockData struct → 5-Persona Debate → Signal → TraderProposal → RiskAssessment → Memory Log

Portfolio: KLBF, TLKM, BBRI, PTBA, BJTM, ADMF, TAPG, JPFA, TSPC, BMRI, ASII
Watchlist: INCO, ANTM, MDKA

Criteria (Value Investing Filter):
- P/E < 15, P/BV < 2, ROE > 10%, D/E < 1, DY > 3%

Execution:
- Position size: 3% of portfolio
- Entry: -2% below level, Stop: -8%, Target: +15%
- Holding: 5 trading days
```

## Implementation Principles

### Cross-Pipeline Correlation Design
1. **Temporal alignment** — News timestamps vs market reaction lag (hours to days)
2. **Entity linking** — Prof Jiang actors to IDX tickers (PTBA = coal, INCO = nickel)
3. **Sentiment aggregation** — Multiple sources confirm signal before action
4. **Confidence scoring** — Higher when multiple independent sources corroborate
5. **Decay function** — Signal strength decreases over time (half-life per event type)
6. **Architectural boundary** — News/social → Vector (Qdrant, semantic). Economic/GDELT → SQLite (lookup, direct). Never mix numerical time-series into vector collections.

### Signal Fusion Rules
```
Strong Signal = (news_sentiment * 0.4) + (social_buzz * 0.2) + (fundamental_score * 0.3) + (pattern_confidence * 0.1)

Where:
- news_sentiment: Prof Jiang investment_signal.confidence
- social_buzz: Normalized relevance + engagement (score + comments)
- fundamental_score: IDX Criteria pass rate (0-5 criteria met / 5)
- pattern_confidence: Historical pattern match confidence
```

### Commodity-Market Linkage
```
Indonesian Strategic Commodities to IDX Tickers:
- Coal (PTBA, ITMG, ADRO) ← thermal coal price + export policy news
- Nickel (INCO, ANTM, MDKA) ← nickel price + EV demand + downstream policy
- Palm Oil (AALI, LSIP, SIMP) ← CPO price + biodiesel mandate + export levy
- Banking (BMRI, BBRI, BJTM) ← interest rate + credit growth + NIM news
- Telco (TLKM) ← regulatory + data consumption + 5G rollout
```

## Architecture Patterns

### Rust: Cross-Collection Query Pattern
```rust
pub async fn find_correlated_events(
    qdrant: &Qdrant,
    query_embedding: &[f32],
    collections: &[&str],
    time_window_hours: i64,
    min_score: f32,
) -> Result<Vec<CorrelatedEvent>> {
    let mut results = Vec::new();
    for collection in collections {
        let search = SearchPointsBuilder::new(collection, query_embedding.to_vec(), 10)
            .score_threshold(min_score);
        let hits = qdrant.search_points(search).await?;
        // Filter by time window, aggregate by entity
        for hit in hits.result {
            results.push(CorrelatedEvent::from_point(hit, collection)?);
        }
    }
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results)
}
```

### Python: Temporal Correlation Pattern
```python
def correlate_news_to_market(
    news_events: List[Dict],
    market_data: Dict[str, float],
    lag_hours: int = 24
) -> List[Dict]:
    """Find market moves correlated to news events. Never raises."""
    try:
        correlations = []
        for event in news_events:
            # Find market moves within lag window after event
            pass
        return correlations
    except Exception as e:
        _log(f"Correlation error: {e}")
        return []
```

## Quality Standards

### Data Integrity
- Dedup BEFORE processing (SHA256 hash for content, URL for identity)
- Idempotent operations (re-run produces same result)
- Audit trail in SQLite staging (status tracking per record)
- Vector similarity dedup at storage layer (Qdrant threshold)

### Performance Targets
| Metric | Target |
|--------|--------|
| RSS collection (29 feeds) | < 5 min |
| Batch labeling (20 articles) | < 30s |
| Embedding + storage (1 article) | < 2s |
| Near-dup check (1 vector) | < 100ms |
| Social collection cycle | < 10 min |

### Monitoring
- Feed health tracking (consecutive failures, last success)
- Collection stats per run (fetched, stored, duplicates, errors)
- Kiromania health + self-healing reauthentication
- TEI health endpoint verification

## Rules
- SELALU design for cross-pipeline data flow (tidak ada pipeline yang isolated)
- SELALU consider temporal lag between news dan market reaction
- SELALU use vector similarity untuk cross-source event matching
- JANGAN hardcode ticker-to-sector mapping — derive from news entity extraction
- SELALU validate signals against multiple independent sources
- SELALU preserve full audit trail (SQLite staging to Qdrant)
- MATCH existing dedup thresholds (0.98/0.92/0.85/0.75)
- MATCH existing embedding dimensions (768 for TEI, 384 for MiniLM)
- SELALU implement "never raises" in Python public functions
- SELALU use anyhow::Result + ? in Rust
- JANGAN skip near-duplicate detection — data integrity > volume
- SELALU log stats with emoji convention
