# Intelligence Dashboard Revamp Requirements
# Christian's Indonesian Portfolio Intelligence System

## 🎯 **Project Overview**

**Objective**: Revamp the existing intelligence dashboard into a modern, real-time portfolio correlation system specialized for Indonesian financial markets with Prof Jiang framework integration.

**Current State**: Dash/Plotly dashboard with basic monitoring capabilities
**Target State**: Next.js + TypeScript + Rust backend with real-time WebSocket updates, BPS integration, and advanced correlation analytics

---

## 📊 **Functional Requirements**

### FR-001: Portfolio Correlation Matrix
**Priority**: Critical  
**As a** portfolio manager  
**I want** real-time correlation analysis for my Indonesian stock holdings  
**So that** I can optimize portfolio allocation and manage risk exposure  

**Acceptance Criteria:**
- ✅ Support for INCO, ANTM, PTBA, TAPG, BMRI, BBRI correlation tracking
- ✅ Real-time correlation matrix updates (30-second intervals)
- ✅ Historical correlation trend analysis (1D, 7D, 30D, 90D windows)
- ✅ Cross-commodity correlation (nickel, coal, palm oil, banking sector)
- ✅ Correlation regime change detection and alerts
- ✅ Portfolio concentration risk scoring

### FR-002: BPS Economic Data Integration
**Priority**: Critical  
**As a** financial analyst  
**I want** real-time Indonesian economic indicators from BPS  
**So that** I can correlate macro conditions with portfolio performance  

**Acceptance Criteria:**
- ✅ BPS API integration with 1.9 req/s rate limit compliance
- ✅ Critical variables: Inflation (var_id: 1, 2, 1709)
- ✅ Automated data quality validation
- ✅ Portfolio impact analysis based on macro indicators
- ✅ Banking sector sensitivity to BI rate changes
- ✅ Commodity export correlation with economic cycles

**Technical Constraints:**
- Must use Christian's registered App ID: 71eeba17f4f1e4b6ef253886c04ec49e
- Rate limiting: Maximum 1.9 requests per second
- Fallback mechanisms for API failures
- Data caching for offline resilience

### FR-003: Intelligence API Integration (localhost:8888)
**Priority**: High  
**As a** intelligence system user  
**I want** seamless integration with existing intelligence services  
**So that** I can leverage Prof Jiang framework analysis and news intelligence  

**Acceptance Criteria:**
- ✅ Full integration with localhost:8888 intelligence API
- ✅ Prof Jiang geopolitical risk scoring
- ✅ News-to-portfolio correlation analysis
- ✅ Agent-Reach news monitoring integration
- ✅ Geostrategy, game theory, and secret history framework scoring
- ✅ Indonesian political stability indicators

### FR-004: Real-time WebSocket Updates
**Priority**: High  
**As a** active trader  
**I want** real-time data updates without page refreshes  
**So that** I can react quickly to market changes and correlation shifts  

**Acceptance Criteria:**
- ✅ WebSocket connection for live price feeds
- ✅ Real-time correlation matrix updates
- ✅ Instant alert notifications
- ✅ Connection health monitoring and auto-reconnection
- ✅ Fallback to HTTP polling if WebSocket fails
- ✅ Client-side state synchronization

### FR-005: Silent Monitoring Mode
**Priority**: Medium  
**As a** system administrator  
**I want** error-only alert system with silent monitoring  
**So that** I can maintain system oversight without notification fatigue  

**Acceptance Criteria:**
- ✅ Error-only notifications (no routine status updates)
- ✅ Critical threshold breach alerts only
- ✅ System health degradation warnings
- ✅ Portfolio risk limit violations
- ✅ BPS API connectivity failures
- ✅ Configurable alert severity levels

---

## 🔧 **Technical Requirements**

### TR-001: Frontend Technology Stack
**Framework**: Next.js 14+ with App Router  
**Language**: TypeScript (strict mode)  
**UI Library**: Ant Design + TailwindCSS  
**State Management**: Zustand for global state  
**Charts**: Recharts for correlation matrices and trend analysis  
**WebSocket**: Socket.io-client for real-time updates  

**Performance Targets:**
- Initial page load: < 2 seconds
- Real-time update latency: < 100ms
- Mobile responsive: 100% compatibility
- Bundle size: < 2MB total
- Memory usage: < 100MB

### TR-002: Backend Technology Stack
**Primary**: Rust with Axum web framework  
**Database**: ArangoDB (existing) for graph correlations  
**Cache**: Redis for real-time data caching  
**WebSocket**: tokio-tungstenite for real-time communication  
**HTTP Client**: reqwest for BPS API integration  
**Monitoring**: Prometheus metrics + Grafana dashboards  

**Performance Targets:**
- API response time: < 50ms p95
- WebSocket message throughput: > 1000 msgs/sec
- Database query time: < 10ms p95
- Memory usage: < 512MB
- CPU usage: < 50% average

### TR-003: Testing Framework Integration (Vibium)
**Repository**: https://github.com/VibiumDev/vibium  
**Purpose**: Automated testing for financial data accuracy  

**Test Coverage Requirements:**
- ✅ BPS API integration tests with rate limiting validation
- ✅ Correlation matrix calculation accuracy tests
- ✅ WebSocket connection stability tests
- ✅ Portfolio risk calculation validation tests
- ✅ Prof Jiang framework scoring tests
- ✅ End-to-end dashboard workflow tests

**Test Data:**
- Historical Indonesian stock price data (2023-2024)
- Mock BPS economic indicators
- Simulated news events with known market impact
- Edge cases: API failures, network timeouts, data corruption

---

## 📈 **Non-Functional Requirements**

### NFR-001: Reliability & Availability
- **Uptime**: 99.5% availability target
- **Error Recovery**: Automatic retry with exponential backoff
- **Failover**: Graceful degradation when dependencies fail
- **Data Integrity**: Validation checksums for all market data
- **Backup**: Real-time data replication to backup systems

### NFR-002: Security & Compliance
- **API Security**: Rate limiting, request validation, error handling
- **Data Privacy**: No PII storage, encrypted data transmission
- **Access Control**: Role-based dashboard access controls
- **Audit Trail**: Complete logging of all trading signal generations
- **Compliance**: Indonesian financial data regulations adherence

### NFR-003: Scalability & Performance
- **Concurrent Users**: Support 10+ simultaneous dashboard users
- **Data Volume**: Handle 100,000+ correlation data points
- **Real-time Processing**: Process 1000+ market updates per minute
- **Storage Growth**: Plan for 1GB+ historical correlation data
- **Network Bandwidth**: Optimize for <1Mbps per user connection

### NFR-004: Maintainability & Operations
- **Monitoring**: Comprehensive health checks and alerting
- **Logging**: Structured logging with correlation IDs
- **Configuration**: Environment-based config management
- **Documentation**: Complete API documentation and runbooks
- **Deployment**: Docker containerization with CI/CD pipeline

---

## 🎖️ **Integration Requirements**

### INT-001: Existing System Compatibility
**Must Preserve:**
- ArangoDB news intelligence data
- Agent-Reach monitoring system (/home/ctianm/.hermes/profiles/social-politic-lab/cron/output/)
- BPS production service (bps_production_service.py)
- Existing Prof Jiang framework analysis

**Migration Strategy:**
- Phased rollout with parallel systems
- Data validation between old and new systems
- Rollback capability to previous dashboard
- User training and documentation updates

### INT-002: External API Dependencies
**BPS Government API:**
- Base URL: https://webapi.bps.go.id/v1/api
- Authentication: App ID based
- Rate Limits: 1.9 requests/second maximum
- Data Quality: Built-in validation and error handling

**Intelligence API (localhost:8888):**
- Health endpoint monitoring
- WebSocket upgrade capability
- Prof Jiang analysis integration
- News correlation scoring

**Market Data Sources:**
- Yahoo Finance Indonesia API
- Jakarta Stock Exchange feeds (backup)
- Local broker APIs (tertiary backup)

---

## 🚨 **Risk Analysis & Mitigation**

### Risk-001: BPS API Rate Limiting
**Risk Level**: High  
**Impact**: Critical data collection failures  
**Mitigation**: 
- Conservative rate limiting (1.9 req/s)
- Request queuing with priority system
- Fallback to cached data during outages
- Multiple API key rotation strategy

### Risk-002: Real-time Data Accuracy
**Risk Level**: High  
**Impact**: Incorrect trading signals and portfolio decisions  
**Mitigation**: 
- Multi-source data validation
- Anomaly detection algorithms
- Historical data consistency checks
- User alerts for data quality issues

### Risk-003: WebSocket Connection Stability  
**Risk Level**: Medium  
**Impact**: Stale dashboard data during market hours  
**Mitigation**: 
- Automatic reconnection with exponential backoff
- HTTP polling fallback mechanism
- Connection health monitoring
- Client-side data staleness indicators

### Risk-004: Indonesian Market Hours Coverage
**Risk Level**: Medium  
**Impact**: Limited real-time updates during off-hours  
**Mitigation**: 
- After-hours data collection for global correlations
- Pre-market analysis and preparation
- Weekend geopolitical monitoring
- Holiday schedule automation

---

## 📋 **Success Criteria**

### Operational Success Metrics
- **System Uptime**: >99.5% during Indonesian market hours
- **Data Latency**: Real-time updates within 30 seconds
- **Alert Accuracy**: >95% relevance rate for critical alerts
- **User Satisfaction**: Positive feedback on dashboard usability

### Financial Success Metrics
- **Portfolio Optimization**: >10% improvement in risk-adjusted returns
- **Correlation Accuracy**: Matrix updates reflect regime changes within 2 minutes
- **Risk Management**: Early warning system prevents >2% portfolio drawdowns
- **Decision Support**: Prof Jiang analysis correlation with market volatility >0.7

### Technical Success Metrics
- **Performance**: <2 second page load times consistently
- **Test Coverage**: >90% code coverage with Vibium framework
- **Documentation**: Complete API docs and operational runbooks
- **Monitoring**: 100% critical component health monitoring

---

## 📅 **Delivery Timeline**

### Phase 1: Foundation (Week 1-2)
- ✅ Rust backend with Axum framework setup
- ✅ BPS API integration with rate limiting
- ✅ ArangoDB correlation data migration
- ✅ Basic WebSocket infrastructure

### Phase 2: Core Dashboard (Week 3-4)
- ✅ Next.js dashboard with TypeScript
- ✅ Real-time correlation matrix display
- ✅ Portfolio overview and risk metrics
- ✅ Intelligence API integration (localhost:8888)

### Phase 3: Advanced Features (Week 5-6)
- ✅ Prof Jiang framework visualization
- ✅ Silent monitoring and alert system
- ✅ Vibium testing framework integration
- ✅ Mobile responsive optimizations

### Phase 4: Testing & Deployment (Week 7-8)
- ✅ End-to-end testing with historical data
- ✅ Performance optimization and load testing
- ✅ Production deployment and monitoring setup
- ✅ User training and documentation completion

**Total Duration**: 8 weeks  
**Resource Requirements**: 1 full-stack developer + Christian for domain expertise  
**Budget Estimate**: Development focused (no external API costs beyond BPS)

---

## 📝 **Notes & Assumptions**

1. **Data Sources**: Assuming continued access to BPS API and existing intelligence system
2. **Infrastructure**: Current ArangoDB and Redis setup sufficient for requirements  
3. **Network**: Stable internet connection for real-time data feeds
4. **Compliance**: Indonesian financial regulations remain stable during development
5. **User Base**: Primary user is Christian, with potential for team expansion
6. **Technology Stack**: Rust expertise available for backend development
7. **Integration**: Existing Prof Jiang framework analysis remains accessible

**Dependencies**: 
- BPS API stability and continued free access
- Intelligence system (localhost:8888) operational continuity  
- ArangoDB performance under increased correlation data volume
- Agent-Reach news monitoring system maintenance

**Out of Scope**:
- Trading execution capabilities (analysis and alerts only)
- Real-time trade settlement and portfolio accounting
- Multi-user authentication and role management (single-user system)
- Advanced portfolio optimization beyond correlation analysis