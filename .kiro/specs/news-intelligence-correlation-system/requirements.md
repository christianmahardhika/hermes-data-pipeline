# News Intelligence Correlation System - Requirements Specification

## Project Overview
Advanced multi-timeframe news intelligence system with sentiment analysis and Prof Jiang Predictive History Framework correlations for Christian's Indonesian Portfolio Intelligence.

## Core Requirements

### 1. Multi-Timeframe News Summary Engine
- **7-day summary**: Recent tactical developments
- **30-day summary**: Short-term trend analysis  
- **3-month summary**: Strategic pattern identification
- **6-month summary**: Long-term geopolitical correlation

### 2. Advanced Sentiment Analysis
- **Indonesian-English hybrid processing**: Handle mixed language content
- **Sector-specific sentiment**: Banking, Mining, Agriculture alignment
- **Portfolio impact scoring**: Direct correlation to BMRI, BBRI, INCO, ANTM, PTBA, TAPG
- **Geopolitical sentiment tracking**: US-China-Indonesia triangulation

### 3. Prof Jiang Framework Integration
- **130-chunk knowledge base**: Geostrategy (72), Game Theory (36), Secret History (22)
- **Predictive correlation engine**: News events → Historical patterns
- **Strategic pattern matching**: Current events vs historical precedents
- **Risk assessment scoring**: Portfolio vulnerability analysis

### 4. Technical Architecture
- **Language**: Rust (performance priority)
- **Database**: Existing ArangoDB graph integration
- **API**: RESTful + WebSocket real-time updates
- **Rate limiting**: Respect BPS 1.9 req/s compliance
- **Integration**: Existing intelligence system (port 8888)

## Data Sources
- **Indonesian News**: Kompas, Detik, Tempo, CNN Indonesia, Antara, Liputan6, Okezone, Republika
- **International Intelligence**: Existing 28+ sources pipeline
- **Prof Jiang KB**: social-politic-kb collection (130 chunks)
- **Portfolio Data**: Real-time Indonesian stock correlations

## Success Criteria
1. **Multi-timeframe summaries** generated and correlated
2. **Sentiment analysis** with portfolio impact scoring
3. **Prof Jiang correlations** with predictive insights
4. **Real-time dashboard integration** with existing system
5. **Enterprise validation** (TDD, QA, DevSecOps) until GREEN

## Performance Requirements
- **Processing**: <5s for 7-day, <30s for 6-month summaries
- **Accuracy**: >85% sentiment classification accuracy
- **Correlation**: Prof Jiang pattern matching >90% precision
- **Availability**: 99.9% uptime integration with existing system

## Integration Points
- **Existing Intelligence System**: localhost:8888
- **Dashboard Frontend**: port 3002 integration
- **BPS Economic Data**: App ID 71eeba17f4f1e4b6ef253886c04ec49e
- **Portfolio Engine**: Real-time stock correlation matrix