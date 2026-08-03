# News Intelligence Correlation System - Technical Design

## System Architecture

### Core Components

#### 1. Multi-Timeframe News Aggregator (Rust)
```rust
pub struct NewsAggregator {
    pub timeframe_engine: TimeframeEngine,
    pub sentiment_analyzer: SentimentAnalyzer, 
    pub correlation_engine: CorrelationEngine,
}

pub enum Timeframe {
    Week,      // 7 days
    Month,     // 30 days  
    Quarter,   // 3 months
    HalfYear,  // 6 months
}
```

#### 2. Prof Jiang Correlation Engine
```rust
pub struct JiangCorrelationEngine {
    pub knowledge_base: JiangKnowledgeBase, // 130 chunks
    pub pattern_matcher: PatternMatcher,
    pub prediction_engine: PredictionEngine,
    pub risk_assessor: RiskAssessor,
}

pub struct HistoricalPattern {
    pub category: JiangCategory, // Geostrategy/GameTheory/SecretHistory
    pub pattern_id: String,
    pub similarity_score: f64,
    pub portfolio_impact: PortfolioImpact,
}
```

#### 3. Indonesian Sentiment Analysis Engine
```rust
pub struct IndonesianSentimentAnalyzer {
    pub bilingual_processor: BilingualProcessor,
    pub sector_classifier: SectorClassifier,
    pub portfolio_impact_scorer: PortfolioImpactScorer,
}

pub enum SentimentCategory {
    Positive(f64),
    Negative(f64),
    Neutral,
    Mixed(PositiveNegative),
}
```

## Database Schema (ArangoDB)

### News Collections
```javascript
// news_summaries
{
  _key: "summary_7d_2026_08_02",
  timeframe: "7_days",
  start_date: "2026-07-26",
  end_date: "2026-08-02", 
  summary: "...",
  sentiment_score: 0.75,
  key_events: [...],
  portfolio_impact: {...},
  jiang_correlations: [...]
}

// jiang_correlations  
{
  _key: "corr_ukraine_energy_2024",
  news_event_id: "news_123",
  jiang_chunk_id: "chunk_45",
  correlation_type: "geostrategy",
  similarity_score: 0.92,
  historical_precedent: "...",
  predicted_outcomes: [...],
  portfolio_risks: {...}
}
```

### API Endpoints

#### News Summary API
```
GET /api/news/summary/{timeframe}
- timeframe: 7d, 30d, 3m, 6m
- Response: Comprehensive summary with sentiment + correlations

GET /api/news/sentiment/{timeframe}
- Sentiment analysis breakdown by sector/source

GET /api/news/correlations/{timeframe}  
- Prof Jiang historical correlations and predictions
```

#### Real-time Updates
```
WebSocket: ws://localhost:8889/news-intelligence
- Real-time news processing notifications
- Sentiment updates
- New correlation discoveries
```

## Processing Pipeline

### 1. Data Collection Phase
```rust
async fn collect_news_data(&self, timeframe: Timeframe) -> Result<NewsCollection> {
    // 1. Query existing Indonesian news sources
    // 2. Filter by timeframe and relevance
    // 3. Deduplicate and clean
    // 4. Store in structured format
}
```

### 2. Sentiment Analysis Phase  
```rust
async fn analyze_sentiment(&self, news: &NewsCollection) -> SentimentAnalysis {
    // 1. Indonesian-English language detection
    // 2. Sector classification (banking/mining/agriculture)
    // 3. Sentiment scoring with portfolio impact weights
    // 4. Aggregate by timeframe and sector
}
```

### 3. Prof Jiang Correlation Phase
```rust
async fn correlate_with_jiang_framework(&self, news: &NewsCollection) -> CorrelationResults {
    // 1. Extract key geopolitical events from news
    // 2. Query Prof Jiang knowledge base (130 chunks)
    // 3. Pattern matching via embeddings similarity
    // 4. Historical precedent analysis  
    // 5. Prediction generation with confidence scores
}
```

### 4. Portfolio Impact Assessment
```rust
async fn assess_portfolio_impact(&self, correlations: &CorrelationResults) -> PortfolioImpact {
    // 1. Map geopolitical events to Indonesian market sectors
    // 2. Calculate risk scores for BMRI, BBRI, INCO, ANTM, PTBA, TAPG
    // 3. Generate actionable insights
    // 4. Recommend portfolio adjustments
}
```

## Integration Architecture

### Existing System Integration
- **Intelligence System**: Direct integration with port 8888 API
- **Dashboard**: New tabs for multi-timeframe analysis
- **BPS Data**: Economic correlation with sentiment trends
- **Portfolio Engine**: Real-time impact scoring integration

### Performance Optimizations
- **Parallel Processing**: Timeframe analysis in separate threads
- **Caching Strategy**: Redis for frequent correlation queries  
- **Incremental Updates**: Only process new news since last run
- **Prof Jiang Embeddings**: Pre-computed similarity matrices