# Hermes Data Pipeline

**Advanced Indonesian Financial Intelligence & Social Analysis System**

## Overview
This repository contains the complete Hermes Agent data pipeline configuration for Christian Mahardhika's Indonesian intelligence system. The system provides real-time financial data collection, social intelligence analysis, news monitoring, and portfolio correlation capabilities.

## 🏗️ Architecture

### Multi-Tier Intelligence Collection System

**Tier 1: Real-time Portfolio Intelligence (30-minute frequency)**
- Commodity price tracking (INCO, PTBA, TAPG)
- LME Nickel, Coal, CPO market data
- Rust-powered high-performance collection

**Tier 2: Social Intelligence Collection (2-hour frequency)**
- HackerNews, Reddit, YouTube analysis
- Social sentiment correlation
- Domain categorization (Tech, Social, Politics, Business)

**Tier 3: Correlation Analysis (4-hour frequency)**
- Social-economic correlation analysis
- Notion portfolio integration
- Prof Jiang Predictive History Framework integration

### Daily Operations
- **Tech Curation:** Daily tech intelligence at 8:00
- **News Monitoring:** Daily news analysis at midnight
- **BI Currency:** Bank Indonesia rates every 8 hours
- **Infrastructure Monitoring:** Agent-Reach every 30 minutes

## 📁 Repository Structure

```
hermes-data-pipeline/
├── crates/                    # Rust components
│   ├── hermes-economic/      # Economic data collection
│   ├── hermes-social/        # Social intelligence
│   └── hermes-common/        # Shared utilities
├── scripts/                  # Pipeline scripts
│   ├── hermes-config/       # Hermes Agent scripts
│   ├── hermes_pipeline_deploy.sh  # Deployment script
│   └── sync_arango_to_qdrant.py   # Database sync
├── documentation/           # System documentation
│   └── current-setup/      # Current configuration
└── news-social-intelligence-data-pipeline/  # Data outputs
```

## 🚀 Quick Start

### 1. Deployment
```bash
# Clone repository
git clone https://github.com/christianmahardhika/hermes-data-pipeline.git
cd hermes-data-pipeline

# Run deployment script
chmod +x scripts/hermes_pipeline_deploy.sh
./scripts/hermes_pipeline_deploy.sh
```

### 2. Prerequisites
- **Hermes Agent** (latest version)
- **Python 3.11+** with required packages
- **Rust** (for compiled components)
- **ArangoDB** (for data storage)
- **Git** (for version control)

### 3. Python Dependencies
```bash
pip install beautifulsoup4 lxml requests pandas \
    transformers torch numpy scikit-learn python-arango
```

## 📊 Current Configuration

### Active Cron Jobs
1. **Advanced Portfolio Intelligence System** - `*/30 * * * *`
2. **Advanced Social Intelligence Collection** - `0 */2 * * *`
3. **Social-Economic Correlation Analysis** - `15 */4 * * *`
4. **Daily Curated Tech Collection** - `0 8 * * *`
5. **arangodb-news-monitor** - `0 0 * * *`
6. **BI Currency Rate Collection** - `0 */8 * * *`
7. **Agent-Reach Fixed Monitor** - `*/30 * * * *`

### Data Storage
- **ArangoDB:** Stock data, news articles, social sentiment
- **Local Files:** Commodity data, tech curation summaries
- **Prof Jiang KB:** Geostrategy, game theory, secret history

### Integration Points
- **Frontend Dashboard:** Next.js at localhost:3002
- **Rust Backend:** Intelligence system at localhost:8888
- **Notion Portfolio:** Investment tracking integration
- **Telegram Delivery:** Automated reporting to designated topics

## 🔧 Management Commands

### View Pipeline Status
```bash
hermes cron list
python3 scripts/hermes-config/check_pipeline_health.py
```

### Manual Operations
```bash
# Run specific job
hermes cron run --job-id JOB_ID

# Update job configuration
hermes cron edit --job-id JOB_ID --schedule "0 9 * * *"

# Check health
python3 scripts/hermes-config/check_pipeline_health.py
```

### Maintenance
```bash
# Weekly maintenance
./scripts/hermes-config/pipeline_maintenance.sh

# Backup configuration
tar -czf backup_$(date +%Y%m%d).tar.gz scripts/hermes-config/
```

## 🛠️ Development

### Building Rust Components
```bash
cd crates/hermes-economic
cargo build --release

cd ../hermes-social
cargo build --release
```

### Testing
```bash
# Run Rust tests
cargo test --all

# Test Python scripts
python3 -m pytest scripts/tests/
```

### Adding New Data Sources
1. Add collector implementation in appropriate crate
2. Create Hermes cron job configuration
3. Update health check script
4. Add to documentation

## 📈 Performance & Optimization

### Current Performance
- **Python → Rust Migration:** 10-100x performance improvement
- **Real-time Processing:** 30-minute intervals for commodity data
- **Social Intelligence:** 2-hour intervals with multi-source collection

### Cost Optimization
- **LLM Usage:** Only essential jobs use LLM models
- **No-Agent Mode:** 6 out of 7 jobs use `no_agent: true`
- **Local Processing:** Rust binaries for high-performance tasks

## 🚨 Troubleshooting

### Common Issues

1. **BI Currency Scraper Error**
   ```bash
   # Install missing dependencies
   pip install beautifulsoup4 lxml
   ```

2. **Cron Job Not Running**
   ```bash
   # Check job status
   hermes cron list
   
   # Verify script permissions
   chmod +x scripts/hermes-config/*.py
   ```

3. **ArangoDB Connection Issues**
   ```bash
   # Verify database is running
   systemctl status arangodb3
   
   # Test connection
   arangosh --server.endpoint tcp://localhost:8529
   ```

4. **Rust Binary Execution Error**
   ```bash
   # Use shell wrapper for cron compatibility
   chmod +x scripts/hermes-config/run_commodity_collector.sh
   ```

### Health Check
Run comprehensive health check:
```bash
python3 scripts/hermes-config/check_pipeline_health.py
```

## 📋 Monitoring & Logging

### Log Files
- **Location:** `~/.hermes/profiles/social-politic-lab/logs/`
- **Retention:** 7 days automatic cleanup

### Backup Strategy
- **Daily:** Script configuration backups
- **Weekly:** Database exports
- **Monthly:** Complete system backup

### Alerting
- **Cron Job Failures:** Immediate notification via Telegram
- **Resource Issues:** Disk space and memory monitoring
- **Data Quality:** Anomaly detection in collected data

## 🔄 Continuous Integration

### GitHub Actions
- **Testing:** Automated testing on push
- **Deployment:** Staging environment deployment
- **Documentation:** Automatic documentation updates

### Deployment Pipeline
1. **Development:** Feature branches with testing
2. **Staging:** Integration testing environment
3. **Production:** Automated deployment with rollback

## 📚 Documentation

### Current Documentation
- **System Architecture:** `documentation/current-setup/hermes_data_pipeline_configuration.md`
- **Deployment Guide:** `scripts/hermes_pipeline_deploy.sh`
- **Script Documentation:** Individual script headers

### Generating Documentation
```bash
# Generate current configuration
python3 scripts/generate_documentation.py

# Update README
./scripts/update_readme.sh
```

## 🤝 Contributing

### Development Workflow
1. Fork the repository
2. Create feature branch: `git checkout -b feature/your-feature`
3. Commit changes: `git commit -m "Add your feature"`
4. Push to branch: `git push origin feature/your-feature`
5. Create Pull Request

### Code Standards
- **Rust:** Follow Rust style guide, clippy checks
- **Python:** PEP 8 compliance, type hints
- **Documentation:** Keep README and comments updated

## 📞 Support & Contact

### Primary Contact
- **Owner:** Christian Mahardhika
- **Profile:** social-politic-lab
- **System:** Indonesian Financial Intelligence Pipeline

### Issue Reporting
- **GitHub Issues:** https://github.com/christianmahardhika/hermes-data-pipeline/issues
- **Telegram:** Designated home topic only

### Emergency Procedures
1. **System Failure:** Restore from latest backup
2. **Data Corruption:** Run data validation scripts
3. **Security Incident:** Isolate system and review logs

## 📊 Performance Metrics

### Current Statistics
- **Portfolio Value:** Rp 13.86M (real-time tracking)
- **Data Sources:** 6 active sources with health monitoring
- **Collection Frequency:** 30-minute to daily intervals
- **Processing Speed:** Sub-second response for most operations

### Success Metrics
- **Uptime:** 99.9% target for critical components
- **Data Accuracy:** >95% accuracy in financial data
- **Processing Time:** <5 seconds for intelligence reports
- **Resource Usage:** <2GB RAM, <10GB disk space

## 🔮 Roadmap

### Short-term (1-3 months)
- [ ] Enhanced machine learning for sentiment analysis
- [ ] Real-time alerting for market anomalies
- [ ] Mobile dashboard application
- [ ] Additional Indonesian data sources

### Medium-term (3-6 months)
- [ ] Advanced predictive analytics
- [ ] Multi-language support
- [ ] API for third-party integration
- [ ] Automated report generation

### Long-term (6-12 months)
- [ ] Blockchain integration for data provenance
- [ ] AI-powered investment recommendations
- [ ] Global market expansion
- [ ] Enterprise deployment options

---

**Repository:** https://github.com/christianmahardhika/hermes-data-pipeline  
**Last Updated:** 2026-08-22  
**Version:** 1.0.0  
**License:** MIT