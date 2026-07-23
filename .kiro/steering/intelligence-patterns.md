---
inclusion: manual
---

# Intelligence Patterns & Correlation Engine

## News-Market Correlation Model

### Temporal Lag Model
```
Event Type          | Lag to Market  | Signal Decay Half-Life
────────────────────┼────────────────┼───────────────────────
Policy change       | 0-4 hours      | 48 hours
Earnings release    | 0-1 hours      | 24 hours
Commodity price     | 1-8 hours      | 12 hours
Geopolitical event  | 4-24 hours     | 72 hours
Social sentiment    | 12-48 hours    | 36 hours
Regulatory change   | 1-5 days       | 7 days
```

### Entity-Ticker Mapping (Indonesian Focus)
```
Sector: Coal & Energy
- Actors: Kementerian ESDM, PLN, coal exporters
- Tickers: PTBA, ITMG, ADRO, BUMI
- Commodities: Thermal Coal (Newcastle benchmark)
- Signals: export ban/levy, DMO policy, ICP price

Sector: Nickel & Mining
- Actors: Kementerian ESDM, smelter operators, EV manufacturers
- Tickers: INCO, ANTM, MDKA, NCKL
- Commodities: Nickel (LME), Cobalt
- Signals: downstream policy, EV demand, export ban enforcement

Sector: Banking
- Actors: Bank Indonesia, OJK, systemic banks
- Tickers: BMRI, BBRI, BBCA, BBNI, BJTM
- Signals: BI rate decision, credit growth, NIM, NPL ratios

Sector: Telco & Digital
- Actors: Kominfo, telco operators, tower companies
- Tickers: TLKM, EXCL, ISAT, TOWR
- Signals: spectrum auction, data pricing, 5G rollout, fiber penetration

Sector: Consumer & Plantation
- Actors: Kementerian Perdagangan, CPO producers
- Tickers: AALI, LSIP, SIMP, UNVR, ICBP
- Commodities: CPO (Malaysia benchmark), Crude Oil
- Signals: biodiesel mandate, export levy, food inflation
```

### Signal Strength Calculation
```
signal_strength = base_confidence * temporal_decay * source_multiplier * corroboration_bonus

Where:
  base_confidence    = Prof Jiang pattern_match.confidence (0.0-1.0)
  temporal_decay     = exp(-0.693 * hours_elapsed / half_life)
  source_multiplier  = {official: 1.5, major_media: 1.2, social: 0.8, unverified: 0.5}
  corroboration_bonus = 1.0 + (0.2 * num_independent_sources) capped at 2.0
```

### Cross-Source Corroboration Rules
```
Confidence Level    | Requirements
────────────────────┼─────────────────────────────────
HIGH (act)          | 2+ independent sources + fundamental confirmation
MEDIUM (monitor)    | 1 reliable source + related social buzz
LOW (log only)      | Single source or unverified social
```

## Deduplication Strategy

### Multi-Layer Dedup
```
Layer 1: URL Hash (exact match, O(1))
  → SHA256(url)[:16] as point ID

Layer 2: Content Hash (exact content match)
  → SHA256(title + content) for identical articles from syndication

Layer 3: Vector Similarity (semantic dedup)
  → Qdrant search with score_threshold
  → 0.92+ = skip (near-duplicate)
  → 0.85-0.92 = link as same_event (store both, mark relation)
  → 0.75-0.85 = store independently (related but distinct)
```

### Cross-Collection Dedup
```
Before storing in ANY collection:
1. Check target collection (URL hash)
2. Check related collections (vector similarity)
3. If same_event found across collections → enrich existing, don't duplicate

Collections to cross-check:
- news_articles ↔ social_intelligence
- unlimited_indonesian_current ↔ news_articles
- unlimited_international_current ↔ social_intelligence
```

## Prof Jiang Integration Points

### Pattern Templates & Market Impact
```
Pattern                  | Typical Market Impact | IDX Sectors Affected
─────────────────────────┼──────────────────────┼─────────────────────
trade_war                | -2% to -8% sector    | Mining, Banking, Export
currency_crisis          | -5% to -15% broad    | All, especially import-heavy
regional_conflict        | -1% to -5% broad     | Energy+, Banking-, Tourism-
political_transition     | +/-3% uncertainty     | SOE-heavy sectors
infrastructure_crisis    | -2% to -5% targeted  | Construction, Materials
corporate_scandal        | -10% to -30% stock   | Specific issuer
commodity_supercycle     | +5% to +20% sector   | Mining, Plantation
rate_cut_cycle           | +3% to +8% broad     | Banking, Property, Consumer
```

### Investment Signal Integration
```
Prof Jiang Output → IDX Analyst Input:
1. investment_signal.action = "buy" + sectors = ["banking"]
   → Boost bull_weight for BMRI, BBRI, BJTM in debate engine
2. pattern_match.template = "currency_crisis" + confidence > 0.7
   → Activate defensive mode, reduce position sizes
3. actors contain "Bank Indonesia" + events tense = "future"
   → Flag for IDX Guru persona (macro specialist)
```

## Data Flow Integrity Rules

### Pipeline Ordering Guarantees
```
MUST maintain order:
1. Collect (fetch raw) → status: "pending"
2. Clean (parse + dedup) → status: "processed" | "failed"
3. Label (Prof Jiang) → status: "labeled" | "label_failed"
4. Embed (TEI + Qdrant) → status: "ingested" | "embed_failed"

Re-run safety:
- Each phase only processes items with correct status
- Failed items can be retried (retry_count tracked)
- Never skip phases (labeled requires cleaned, embedded requires labeled)
```

### Embedding Dimension Consistency
```
CRITICAL: Never mix embedding dimensions in same collection
- TEI multilingual-e5-base → 768 dimensions (Rust pipeline default)
- all-MiniLM-L6-v2 → 384 dimensions (Python legacy)
- Future migrations must create new collections, not overwrite

If migrating 384 → 768:
1. Create new collection with 768-dim config
2. Re-embed all content via TEI
3. Switch reads to new collection
4. Archive old collection (don't delete)
```

## Monitoring & Alerting Patterns

### Intelligence Health Metrics
```
Pipeline Health:
- feed_success_rate > 80% (else: source degradation alert)
- labeling_success_rate > 90% (else: LLM service issue)
- embedding_latency_p95 < 5s (else: TEI overload)
- dedup_rate < 30% per run (else: stale feeds, no new content)

Signal Quality:
- avg_confidence > 0.5 for Prof Jiang output
- correlation_hit_rate (signals that preceded market moves)
- false_positive_rate (signals without subsequent market move)
```

## Data Source Expansion Roadmap

### Phase 1: RSS Expansion (zero infrastructure change)
```
New feeds to validate and add to collectors/mod.rs:
- Jakarta Globe (English ID business)
- Katadata (ID economic data journalism)
- Reuters via feedx.net (wire service proxy)
- Investing.com RSS (market alerts)
- CNBC Indonesia subcategory feeds (/market, /tech)
```

### Phase 2: Economic Data Module (new src/economic/)
```
Structured numerical time-series — NOT text, different from news pipeline:
- FRED API (US: Fed Funds Rate, CPI, GDP, unemployment)
- Bank Indonesia API (BI rate, JIBOR, inflation, forex reserves)
- Stored as EconomicIndicator { source, indicator, value, unit, timestamp, change_pct }
- Correlation trigger: BI rate change → banking tickers (BMRI, BBRI, BJTM)
```

### Phase 3: Geopolitical Event Databases (quantitative conflict data)
```
- GDELT (event tone + Goldstein scale) — cross-validates Prof Jiang events
- ACLED (armed conflict with geo-coordinates) — country risk scoring
- FIRMS/NASA (active fires) — palm oil supply chain disruption
- Multi-source confidence: Prof Jiang + GDELT + ACLED = HIGH confidence signal
```

### Source Selection Criteria
```
Must meet ALL:
1. Free or free-tier available (no paid subscriptions for MVP)
2. Stable endpoint (>95% uptime, not frequently restructured)
3. Machine-readable (RSS/Atom/JSON API — no scraping)
4. Relevant to Indonesian market investment decisions
5. Adds signal not already covered by existing sources
```
