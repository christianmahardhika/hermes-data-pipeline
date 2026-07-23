# Hermes Cron Job Scripts

Portfolio analysis and financial data pipeline scripts for Hermes Agent automation.

## Core Commands

### IDX AI Analyst (Rust Binary - Consolidated)

**Single command replaces:** screener-fetch + screener-digest + accumulation digest

```bash
# Full portfolio analysis with RTI Business format
cargo run --release -- idx-analyst --portfolio --full

# Compact format (suitable for Telegram)
cargo run --release -- idx-analyst --portfolio

# Specific stocks with mock data
cargo run --release -- idx-analyst KLBF BBRI --mock --full

# Help
cargo run --release -- idx-analyst --help
```

**Location:** `news-social-intelligence-data-pipeline/src/main.rs`

**Output:** RTI-formatted analysis with 5-persona debate results

---

## Cron Job Configuration (Hermes)

### Replace these 3 jobs with consolidated Rust binary:

**Before (3 separate jobs):**
```bash
hermes cron create \
  --name "IDX Screener Fetch" \
  --schedule "0 9 * * 1-5" \
  --script "screener-fetch-cron.sh"

hermes cron create \
  --name "IDX Screener Digest" \
  --schedule "0 9 * * 1-5" \
  --script "screener-digest-cron.sh"

hermes cron create \
  --name "Accumulation Digest" \
  --schedule "0 15 * * 1-5" \
  --script "sospol_accumulation_digest.py"
```

**After (1 unified job):**
```bash
hermes cron create \
  --name "Enhanced IDX AI Analyst — Portfolio" \
  --schedule "0 9,15 * * 1-5" \
  --prompt "cd ~/projects/hermes-data-pipeline/news-social-intelligence-data-pipeline && cargo run --release -- idx-analyst --portfolio --full"
```

---

## Supporting Python Scripts

These scripts remain for specialized analysis:

- `daily_digest*.py` — Financial intelligence digest generation
- `portfolio_*.py` — Fundamental, sentiment, retirement assessment
- `inco_*.py` — INCO-specific deep-dive analysis
- `stock_alerts_*.py` — BUMN export monitoring
- `*_monitor.py` — Utility monitoring scripts

---

## Integration with Pagupon Finance Profile

All scripts reference this portfolio configuration:
- Location: `~/.hermes/profiles/pagupon-finance/`
- Output directory: `~/.hermes/profiles/pagupon-finance/cron/output/`
- Cron jobs: Scheduled via `hermes cron` command

---

## Performance Notes

- Rust binary (`idx-analyst`) is **significantly faster** than Python equivalents
- Supports real Yahoo Finance data with mock fallback
- 5-persona debate engine with configurable rounds
- Output formats: RTI Business (full) or Telegram (compact)

---

## Development

To rebuild the Rust binary after changes:

```bash
cd news-social-intelligence-data-pipeline
cargo build --release
```

Binary location: `target/release/news-collector`
