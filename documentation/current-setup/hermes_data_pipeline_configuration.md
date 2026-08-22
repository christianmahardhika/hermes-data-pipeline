# Hermes Data Pipeline Configuration
**Current Setup Documentation** - Captured on 2026-08-22
**Profile:** social-politic-lab
**Owner:** Christian Mahardhika
**Focus:** Indonesian Financial Intelligence & Social-Political Analysis

## Overview
This document captures the complete Hermes Agent data pipeline configuration for the advanced Indonesian intelligence system. The system integrates real-time financial data, social intelligence, news monitoring, and portfolio correlation analysis.

## System Architecture

### 1. Multi-Tier Intelligence Collection

**Tier 1: Real-time Portfolio Intelligence (30-minute frequency)**
- **Job ID:** 2be1dce649c1 - "Advanced Portfolio Intelligence System"
- **Schedule:** `*/30 * * * *` (every 30 minutes)
- **Script:** `run_commodity_collector.sh`
- **Delivery:** `local` (save to files)
- **Technology:** Rust-powered commodity collector
- **Data Collected:** INCO, PTBA, TAPG commodity prices (LME Nickel, Coal, CPO)
- **Toolsets:** terminal, file
- **Status:** ✅ Active, last run 2026-08-22T07:00:01

**Tier 2: Social Intelligence Collection (2-hour frequency)**
- **Job ID:** 771315d2a9e4 - "Advanced Social Intelligence Collection"
- **Schedule:** `0 */2 * * *` (every 2 hours)
- **Script:** `enhanced_social_intelligence.sh`
- **Delivery:** `local`
- **Technology:** Rust-enhanced social intelligence
- **Sources:** HackerNews, Reddit, YouTube
- **Toolsets:** terminal, file
- **Status:** ✅ Active, last run 2026-08-22T06:45:07

**Tier 3: Correlation Analysis (4-hour frequency)**
- **Job ID:** c746ece93aa5 - "Social-Economic Correlation Analysis"
- **Schedule:** `15 */4 * * *` (every 4 hours at minute 15)
- **Script:** `social_economic_analysis.py`
- **Delivery:** `local`
- **Integration:** Notion Portfolio Integration
- **Analysis:** Social sentiment vs market data correlation
- **Toolsets:** terminal, file
- **Status:** ✅ Active, last run 2026-08-22T06:47:55

### 2. Daily Curated Collections

**Daily Tech Intelligence**
- **Job ID:** 7fa105e7137c - "Daily Curated Tech Collection"
- **Schedule:** `0 8 * * *` (daily at 8:00)
- **Script:** `daily_tech_curation.py`
- **Delivery:** `origin` (back to this chat)
- **Technology:** Rust-powered tech curation
- **Toolsets:** terminal, file, web
- **Status:** ✅ Active, last run 2026-08-22T06:47:31

**Daily News Monitoring**
- **Job ID:** 5bb5aa437c44 - "arangodb-news-monitor"
- **Schedule:** `0 0 * * *` (daily at midnight)
- **Script:** `arangodb_news_monitor.py`
- **Delivery:** `origin`
- **Model:** groq/llama-3.3-70b-versatile
- **Provider:** 9router
- **Function:** Track article counts, source distribution
- **Status:** ✅ Active, last run 2026-08-22T06:45:07

**BI Currency Rate Collection**
- **Job ID:** 3008c0bc613d - "BI Currency Rate Collection (Enhanced)"
- **Schedule:** `0 */8 * * *` (every 8 hours)
- **Script:** `bi_currency_scraper.py`
- **Delivery:** `origin`
- **Method:** arifwidip's scraping method modernized
- **Status:** ⚠️ Error, last run 2026-08-22T06:45:07 (requires Python package dependencies)

### 3. Infrastructure Monitoring

**Agent-Reach Fixed Monitor**
- **Job ID:** 1f15299ee81a - "Agent-Reach Fixed Monitor"
- **Schedule:** `*/30 * * * *` (every 30 minutes)
- **Script:** `agent_reach_monitor.sh`
- **Delivery:** `local`
- **no_agent:** true
- **Status:** ✅ Active, last run 2026-08-22T07:00:07

## Data Flow Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Real-time       │───▶│ ArangoDB        │───▶│ Frontend        │
│ Commodity Data  │    │ Collections     │    │ Dashboard       │
│ (30-min)        │    │ (stocks, news)  │    │ (Next.js)       │
└─────────────────┘    └─────────────────┘    └────────┬────────┘
          │                                          │
          ▼                                          ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Social          │───▶│ Correlation     │───▶│ Notion          │
│ Intelligence    │    │ Analysis        │    │ Portfolio       │
│ (2-hour)        │    │ (4-hour)        │    │ Integration     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
          │
          ▼
┌─────────────────┐    ┌─────────────────┐
│ Daily Tech      │───▶│ Telegram        │
│ Curation        │    │ Delivery        │
│ (8:00 daily)    │    │                 │
└─────────────────┘    └─────────────────┘
```

## Script Inventory

### Rust Components
1. `commodity_collector_rust` - High-performance commodity data collection (61MB binary)
2. `commodity_collector.py` - Python wrapper for Rust collector (10KB)

### Python Scripts
1. `social_economic_analysis.py` - Social-economic correlation analysis
2. `daily_tech_curation.py` - Daily tech intelligence collection
3. `arangodb_news_monitor.py` - News database monitoring with LLM
4. `bi_currency_scraper.py` - Bank Indonesia currency rate collection
5. `check_db_stats.py` - Database statistics checker
6. `indonesian_news_collector.py` - Indonesian news collection

### Shell Scripts
1. `enhanced_social_intelligence.sh` - Social intelligence collection wrapper
2. `run_commodity_collector.sh` - Commodity collector wrapper
3. `agent_reach_monitor.sh` - Agent-Reach monitoring
4. `news_monitor_fixed.sh` - Fixed news monitoring
5. `social_intel_cron.py` - Social intelligence cron job

## Data Storage Structure

### Output Files
- **Commodity Data:** `commodity_data_*.json` (multiple timestamps)
- **Tech Curation:** `tech_curation_data_*.json` and `tech_curation_summary_*.txt` (daily)
- **Social Intelligence:** Various JSON outputs in `advanced-intelligence-system/`

### Database Collections (ArangoDB)
- Indonesian stock data (BMRI, BBRI, INCO, ANTM, PTBA, TAPG)
- News articles from Indonesian sources (Kompas, Detik, Tempo, etc.)
- Social media sentiment data
- Prof Jiang knowledge base chunks (geostrategy, game theory, secret history)

## Integration Points

### 1. Frontend Dashboard
- **URL:** http://localhost:3002 (Next.js)
- **Tailscale:** http://100.70.96.84:3002
- **Backend:** Rust intelligence system on localhost:8888
- **Features:** Real portfolio tracking (Rp 13.86M), monitoring source health, global/domestic news summaries

### 2. Backend Systems
- **Rust Backend:** localhost:8888 - Serves real Indonesian financial data
- **ArangoDB:** Stock data storage with 4 active collections
- **BPS API:** Indonesian government inflation data (3.18% current)

### 3. Monitoring APIs
- `/api/monitoring/sources` - Tracks 6 data sources health status
- `/api/news-summary` - Global and domestic news summaries with sentiment analysis
- `/api/portfolio` - Real-time portfolio value and stock data

## Performance Characteristics

### Migration Success
- **Python → Rust Migration:** 10-100x performance improvement
- **Real-time Processing:** 30-minute intervals for commodity data
- **Social Intelligence:** 2-hour intervals with multi-source collection

### Cost Optimization
- **LLM Usage:** Only essential jobs use LLM models (news monitoring)
- **No-Agent Mode:** 6 out of 7 jobs use `no_agent: true` for cost efficiency
- **Local Processing:** Rust binaries for high-performance tasks

## Dependencies & Requirements

### Python Packages (for scripts)
```
beautifulsoup4>=4.12.0
lxml>=4.9.0
requests>=2.31.0
pandas>=2.0.0
transformers>=4.30.0
torch>=2.0.0
numpy>=1.24.0
scikit-learn>=1.3.0
python-arango>=7.0.0
```

### System Requirements
- **Rust:** For compiled binary execution
- **ArangoDB:** Running instance with collections
- **Hermes Agent:** Version supporting cron jobs and profiles
- **Python 3.11+:** For script execution

## Current Issues & Recommendations

### Issues
1. **BI Currency Scraper Error:** Requires Python package dependencies (bs4, lxml, etc.)
2. **Binary Execution Conflicts:** Rust binaries need shell wrapper scripts for cron compatibility
3. **Delivery Routing:** Some jobs deliver to `local` only, may need Telegram integration

### Recommendations
1. **Fix BI Scraper:** Install missing Python packages
2. **Consolidate Outputs:** Create unified reporting endpoint
3. **Enhanced Monitoring:** Add health checks for all pipeline components
4. **Backup Strategy:** Implement data backup for critical collections

## Security Considerations

- **Secret Redaction:** Enabled by default in Hermes configuration
- **API Keys:** Stored in `~/.hermes/profiles/social-politic-lab/.env`
- **Local Processing:** Financial data processed locally for privacy
- **Access Control:** Tailscale for secure remote dashboard access

## Recovery & Maintenance

### Cron Job Recovery Commands
```bash
# Check job status
hermes cron list

# Manually run failed job
hermes cron run --job-id 3008c0bc613d

# Update job configuration
hermes cron edit --job-id 3008c0bc613d --script "bi_currency_scraper.py"
```

### Data Backup
- **Commodity Data:** JSON files in scripts directory
- **Tech Curation:** Daily summaries with timestamps
- **ArangoDB:** Regular database exports recommended

## Evolution History

### Key Milestones
1. **June 2026:** Initial news collection system setup
2. **July 2026:** Python → Rust migration for 10-100x performance improvement
3. **August 2026:** Advanced intelligence system deployment with multi-tier architecture
4. **August 2026:** Frontend dashboard development and production deployment

### Current Capabilities
- Real-time Indonesian portfolio tracking (Rp 13.86M)
- Multi-source social intelligence collection
- Commodity price correlation analysis
- Automated news curation and summarization
- Professional enterprise-grade dashboard interface

---

**Last Updated:** 2026-08-22  
**Document Version:** 1.0  
**Maintainer:** Hermes Agent (social-politic-lab profile)