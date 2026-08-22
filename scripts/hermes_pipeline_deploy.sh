#!/bin/bash
# Hermes Data Pipeline Deployment Script
# For social-politic-lab profile - Christian Mahardhika's Indonesian Intelligence System
# Version: 1.0
# Date: 2026-08-22

set -e  # Exit on error

echo "🚀 Hermes Data Pipeline Deployment Script"
echo "========================================"
echo "Profile: social-politic-lab"
echo "System: Indonesian Financial Intelligence"
echo "Owner: Christian Mahardhika"
echo ""

# Configuration
PROFILE_DIR="$HOME/.hermes/profiles/social-politic-lab"
SCRIPTS_DIR="$PROFILE_DIR/scripts"
BACKUP_DIR="$PROFILE_DIR/backups"
LOG_DIR="$PROFILE_DIR/logs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
log_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

log_info() {
    echo -e "${BLUE}→ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

log_error() {
    echo -e "${RED}✗ $1${NC}"
}

check_requirements() {
    log_info "Checking system requirements..."
    
    # Check Hermes installation
    if command -v hermes &> /dev/null; then
        log_success "Hermes Agent is installed"
    else
        log_error "Hermes Agent is not installed"
        echo "Install with: curl -fsSL https://hermes-agent.nousresearch.com/install.sh | bash"
        exit 1
    fi
    
    # Check Python 3.11+
    python_version=$(python3 --version 2>/dev/null | cut -d' ' -f2)
    if [[ "$python_version" > "3.10" ]]; then
        log_success "Python $python_version is installed"
    else
        log_error "Python 3.11+ is required"
        exit 1
    fi
    
    # Check Rust (optional but recommended)
    if command -v cargo &> /dev/null; then
        log_success "Rust is installed"
    else
        log_warning "Rust is not installed (required for commodity_collector_rust)"
    fi
    
    # Check ArangoDB connection (optional)
    if command -v arangosh &> /dev/null; then
        log_success "ArangoDB shell is available"
    else
        log_warning "ArangoDB is not installed or not in PATH"
    fi
}

setup_directories() {
    log_info "Setting up pipeline directories..."
    
    mkdir -p "$SCRIPTS_DIR"
    mkdir -p "$BACKUP_DIR"
    mkdir -p "$LOG_DIR"
    
    log_success "Directories created"
}

install_python_dependencies() {
    log_info "Installing Python dependencies..."
    
    pip install --user \
        beautifulsoup4>=4.12.0 \
        lxml>=4.9.0 \
        requests>=2.31.0 \
        pandas>=2.0.0 \
        transformers>=4.30.0 \
        torch>=2.0.0 \
        numpy>=1.24.0 \
        scikit-learn>=1.3.0 \
        python-arango>=7.0.0
    
    log_success "Python dependencies installed"
}

backup_existing_config() {
    log_info "Backing up existing configuration..."
    
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    BACKUP_FILE="$BACKUP_DIR/hermes_pipeline_backup_$TIMESTAMP.tar.gz"
    
    # Backup cron jobs
    hermes cron list > "$BACKUP_DIR/cron_jobs_backup_$TIMESTAMP.txt" 2>/dev/null || true
    
    # Backup scripts directory
    if [ -d "$SCRIPTS_DIR" ]; then
        tar -czf "$BACKUP_FILE" -C "$PROFILE_DIR" scripts/ 2>/dev/null || true
        log_success "Backup created: $BACKUP_FILE"
    else
        log_warning "No scripts directory found for backup"
    fi
}

deploy_cron_jobs() {
    log_info "Deploying Hermes cron jobs..."
    
    echo ""
    echo "📅 Current Cron Jobs Status:"
    echo "---------------------------"
    hermes cron list
    
    echo ""
    echo "🔄 Deploying pipeline jobs..."
    
    # Job 1: Real-time Portfolio Intelligence (30-min)
    log_info "Deploying Advanced Portfolio Intelligence System..."
    hermes cron create \
        --name "Advanced Portfolio Intelligence System" \
        --schedule "*/30 * * * *" \
        --script "run_commodity_collector.sh" \
        --no-agent true \
        --toolsets "terminal,file" \
        --deliver "local"
    
    # Job 2: Social Intelligence Collection (2-hour)
    log_info "Deploying Advanced Social Intelligence Collection..."
    hermes cron create \
        --name "Advanced Social Intelligence Collection" \
        --schedule "0 */2 * * *" \
        --script "enhanced_social_intelligence.sh" \
        --no-agent true \
        --toolsets "terminal,file" \
        --deliver "local"
    
    # Job 3: Correlation Analysis (4-hour)
    log_info "Deploying Social-Economic Correlation Analysis..."
    hermes cron create \
        --name "Social-Economic Correlation Analysis" \
        --schedule "15 */4 * * *" \
        --script "social_economic_analysis.py" \
        --toolsets "terminal,file" \
        --deliver "local"
    
    # Job 4: Daily Tech Curation
    log_info "Deploying Daily Curated Tech Collection..."
    hermes cron create \
        --name "Daily Curated Tech Collection" \
        --schedule "0 8 * * *" \
        --script "daily_tech_curation.py" \
        --toolsets "terminal,file,web" \
        --deliver "origin"
    
    # Job 5: News Monitoring
    log_info "Deploying arangodb-news-monitor..."
    hermes cron create \
        --name "arangodb-news-monitor" \
        --schedule "0 0 * * *" \
        --script "arangodb_news_monitor.py" \
        --model "groq/llama-3.3-70b-versatile" \
        --provider "9router" \
        --deliver "origin"
    
    # Job 6: BI Currency Rate Collection
    log_info "Deploying BI Currency Rate Collection (Enhanced)..."
    hermes cron create \
        --name "BI Currency Rate Collection (Enhanced)" \
        --schedule "0 */8 * * *" \
        --script "bi_currency_scraper.py" \
        --no-agent true \
        --deliver "origin"
    
    # Job 7: Agent-Reach Monitor
    log_info "Deploying Agent-Reach Fixed Monitor..."
    hermes cron create \
        --name "Agent-Reach Fixed Monitor" \
        --schedule "*/30 * * * *" \
        --script "agent_reach_monitor.sh" \
        --no-agent true \
        --deliver "local"
    
    log_success "All cron jobs deployed"
}

create_health_check_script() {
    log_info "Creating pipeline health check script..."
    
    cat > "$SCRIPTS_DIR/check_pipeline_health.py" << 'EOF'
#!/usr/bin/env python3
"""
Hermes Pipeline Health Check
Check status of all pipeline components
"""

import subprocess
import json
import os
from datetime import datetime

def check_cron_jobs():
    """Check Hermes cron job status"""
    try:
        result = subprocess.run(
            ['hermes', 'cron', 'list'],
            capture_output=True,
            text=True,
            timeout=30
        )
        return result.stdout if result.returncode == 0 else result.stderr
    except Exception as e:
        return f"Error checking cron jobs: {e}"

def check_arangodb():
    """Check ArangoDB connection"""
    try:
        # Try to connect to ArangoDB
        from arango import ArangoClient
        client = ArangoClient(hosts='http://localhost:8529')
        sys_db = client.db('_system', username='root', password='')
        version = sys_db.version()
        return f"ArangoDB Connected: {version}"
    except ImportError:
        return "python-arango not installed"
    except Exception as e:
        return f"ArangoDB Error: {e}"

def check_python_packages():
    """Check required Python packages"""
    packages = [
        'beautifulsoup4',
        'lxml', 
        'requests',
        'pandas',
        'transformers',
        'torch',
        'numpy',
        'sklearn',
        'arango'
    ]
    
    missing = []
    for package in packages:
        try:
            __import__(package)
        except ImportError:
            missing.append(package)
    
    return missing

def check_rust_binary():
    """Check if Rust binary exists"""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    binary_path = os.path.join(script_dir, 'commodity_collector_rust')
    
    if os.path.exists(binary_path):
        size = os.path.getsize(binary_path)
        return f"Rust binary exists: {size:,} bytes"
    else:
        return "Rust binary not found"

def main():
    print("🔍 Hermes Pipeline Health Check")
    print("=" * 40)
    print(f"Timestamp: {datetime.now().isoformat()}")
    print()
    
    # Check cron jobs
    print("📅 Cron Jobs Status:")
    print(check_cron_jobs())
    print()
    
    # Check ArangoDB
    print("🗄️ Database Status:")
    print(check_arangodb())
    print()
    
    # Check Python packages
    print("🐍 Python Packages:")
    missing = check_python_packages()
    if missing:
        print(f"❌ Missing packages: {', '.join(missing)}")
    else:
        print("✅ All required packages installed")
    print()
    
    # Check Rust binary
    print("⚡ Rust Components:")
    print(check_rust_binary())
    print()
    
    # Check script directory
    print("📁 Script Directory:")
    script_dir = os.path.dirname(os.path.abspath(__file__))
    python_scripts = [f for f in os.listdir(script_dir) if f.endswith('.py')]
    shell_scripts = [f for f in os.listdir(script_dir) if f.endswith('.sh')]
    print(f"Python scripts: {len(python_scripts)}")
    print(f"Shell scripts: {len(shell_scripts)}")
    print(f"Total files: {len(os.listdir(script_dir))}")
    
    print()
    print("✅ Health check completed")

if __name__ == "__main__":
    main()
EOF
    
    chmod +x "$SCRIPTS_DIR/check_pipeline_health.py"
    log_success "Health check script created"
}

create_maintenance_script() {
    log_info "Creating maintenance script..."
    
    cat > "$SCRIPTS_DIR/pipeline_maintenance.sh" << 'EOF'
#!/bin/bash
# Hermes Pipeline Maintenance Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="$SCRIPT_DIR/../logs"
BACKUP_DIR="$SCRIPT_DIR/../backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "🧹 Hermes Pipeline Maintenance"
echo "============================="
echo "Timestamp: $TIMESTAMP"
echo ""

# 1. Clean old log files (keep last 7 days)
echo "📝 Cleaning old log files..."
find "$LOG_DIR" -name "*.log" -mtime +7 -delete 2>/dev/null || true
echo "✅ Log files cleaned"

# 2. Clean old backup files (keep last 30 days)
echo "💾 Cleaning old backup files..."
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +30 -delete 2>/dev/null || true
echo "✅ Backup files cleaned"

# 3. Clean old data files (keep last 14 days)
echo "🗑️ Cleaning old data files..."
find "$SCRIPT_DIR" -name "commodity_data_*.json" -mtime +14 -delete 2>/dev/null || true
find "$SCRIPT_DIR" -name "tech_curation_data_*.json" -mtime +14 -delete 2>/dev/null || true
find "$SCRIPT_DIR" -name "tech_curation_summary_*.txt" -mtime +14 -delete 2>/dev/null || true
find "$SCRIPT_DIR" -name "social_intel_data_*.json" -mtime +14 -delete 2>/dev/null || true
echo "✅ Data files cleaned"

# 4. Check disk usage
echo "💿 Checking disk usage..."
du -sh "$SCRIPT_DIR" || true
echo ""

# 5. Run health check
echo "🔍 Running health check..."
python3 "$SCRIPT_DIR/check_pipeline_health.py" 2>/dev/null || echo "Health check failed"
echo ""

echo "✅ Maintenance completed"
EOF
    
    chmod +x "$SCRIPTS_DIR/pipeline_maintenance.sh"
    log_success "Maintenance script created"
}

create_readme() {
    log_info "Creating README documentation..."
    
    cat > "$SCRIPTS_DIR/README.md" << 'EOF'
# Hermes Data Pipeline - Indonesian Intelligence System

## Overview
This pipeline provides comprehensive Indonesian financial intelligence, social analysis, and news monitoring using Hermes Agent cron jobs.

## Architecture

### Tier 1: Real-time Data Collection (30-min)
- **Job:** Advanced Portfolio Intelligence System
- **Frequency:** Every 30 minutes
- **Script:** `run_commodity_collector.sh`
- **Data:** INCO, PTBA, TAPG commodity prices

### Tier 2: Social Intelligence (2-hour)
- **Job:** Advanced Social Intelligence Collection  
- **Frequency:** Every 2 hours
- **Script:** `enhanced_social_intelligence.sh`
- **Sources:** HackerNews, Reddit, YouTube

### Tier 3: Correlation Analysis (4-hour)
- **Job:** Social-Economic Correlation Analysis
- **Frequency:** Every 4 hours at minute 15
- **Script:** `social_economic_analysis.py`
- **Integration:** Notion Portfolio

### Daily Collections
- **Tech Curation:** Daily at 8:00 (`daily_tech_curation.py`)
- **News Monitoring:** Daily at midnight (`arangodb_news_monitor.py`)
- **BI Currency:** Every 8 hours (`bi_currency_scraper.py`)

### Infrastructure Monitoring
- **Agent-Reach Monitor:** Every 30 minutes (`agent_reach_monitor.sh`)

## Management Commands

### View Status
```bash
hermes cron list
python3 check_pipeline_health.py
```

### Run Maintenance
```bash
./pipeline_maintenance.sh
```

### Manual Job Execution
```bash
# Run specific job
hermes cron run --job-id JOB_ID

# Update job configuration
hermes cron edit --job-id JOB_ID --schedule "0 9 * * *"
```

### Backup & Restore
```bash
# Backup current configuration
tar -czf backup_$(date +%Y%m%d).tar.gz scripts/

# Restore from backup
tar -xzf backup_20260822.tar.gz -C ~/.hermes/profiles/social-politic-lab/
```

## Dependencies

### Python Packages
```bash
pip install beautifulsoup4 lxml requests pandas transformers torch numpy scikit-learn python-arango
```

### System Requirements
- Hermes Agent (latest version)
- Python 3.11+
- Rust (for commodity collector)
- ArangoDB (for data storage)

## Troubleshooting

### Common Issues

1. **BI Currency Scraper Error**
   ```
   ModuleNotFoundError: No module named 'bs4'
   ```
   **Fix:** `pip install beautifulsoup4 lxml`

2. **Cron Job Not Running**
   - Check job status: `hermes cron list`
   - Verify schedule format
   - Check script permissions: `chmod +x script.py`

3. **ArangoDB Connection Issues**
   - Verify database is running: `systemctl status arangodb3`
   - Check connection details in scripts

4. **Rust Binary Execution Error**
   - Use shell wrapper for cron compatibility
   - Ensure binary has execute permissions: `chmod +x commodity_collector_rust`

### Health Check
Run the health check script to diagnose issues:
```bash
python3 check_pipeline_health.py
```

## Data Storage

### Output Files
- Commodity data: `commodity_data_*.json`
- Tech curation: `tech_curation_data_*.json`, `tech_curation_summary_*.txt`
- Social intelligence: `social_intel_data_*.json`

### Database Collections
- Indonesian stocks (BMRI, BBRI, INCO, ANTM, PTBA, TAPG)
- News articles from Indonesian sources
- Social media sentiment data
- Prof Jiang knowledge base

## Monitoring

### Log Files
- Location: `~/.hermes/profiles/social-politic-lab/logs/`
- Retention: 7 days

### Backup Files
- Location: `~/.hermes/profiles/social-politic-lab/backups/`
- Retention: 30 days

## Contact & Support
- **Owner:** Christian Mahardhika
- **Profile:** social-politic-lab
- **System:** Indonesian Financial Intelligence Pipeline

---

**Last Updated:** 2026-08-22  
**Version:** 1.0
EOF
    
    log_success "README documentation created"
}

run_initial_health_check() {
    log_info "Running initial health check..."
    
    python3 "$SCRIPTS_DIR/check_pipeline_health.py" || true
    
    log_success "Health check completed"
}

show_summary() {
    echo ""
    echo "🎉 Hermes Data Pipeline Deployment Complete!"
    echo "=========================================="
    echo ""
    echo "📊 Summary:"
    echo "----------"
    echo "• 7 cron jobs deployed"
    echo "• Health check script: $SCRIPTS_DIR/check_pipeline_health.py"
    echo "• Maintenance script: $SCRIPTS_DIR/pipeline_maintenance.sh"
    echo "• Documentation: $SCRIPTS_DIR/README.md"
    echo "• Backups: $BACKUP_DIR/"
    echo "• Logs: $LOG_DIR/"
    echo ""
    echo "🚀 Next Steps:"
    echo "-------------"
    echo "1. Review deployed jobs: hermes cron list"
    echo "2. Test health check: python3 $SCRIPTS_DIR/check_pipeline_health.py"
    echo "3. Run maintenance weekly: $SCRIPTS_DIR/pipeline_maintenance.sh"
    echo "4. Monitor logs in: $LOG_DIR/"
    echo ""
    echo "📈 For production monitoring:"
    echo "---------------------------"
    echo "• Check job status regularly"
    echo "• Review error logs"
    echo "• Update dependencies quarterly"
    echo "• Backup critical data monthly"
    echo ""
    echo "✅ Deployment successful!"
}

# Main execution
main() {
    echo "🔧 Hermes Data Pipeline Deployment"
    echo "================================="
    
    check_requirements
    setup_directories
    backup_existing_config
    install_python_dependencies
    deploy_cron_jobs
    create_health_check_script
    create_maintenance_script
    create_readme
    run_initial_health_check
    show_summary
}

# Execute main function
main