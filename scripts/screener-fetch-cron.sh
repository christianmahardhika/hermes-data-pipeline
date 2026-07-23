#!/bin/bash
# Pagupon Finance — Data Fetch Cron Job (before 09:00 & 15:00 WIB)
# Real file: ~/.hermes/profiles/pagupon-finance/scripts/screener-fetch-cron.sh

set -e

SCREENER_BIN="$HOME/.hermes/profiles/pagupon-finance/screener/target/debug/screener"
PROFILE_DIR="$HOME/.hermes/profiles/pagupon-finance"

# Load environment
cd "$PROFILE_DIR"
export $(cat .env 2>/dev/null | xargs) || true

# Fetch fresh market data for all portfolio stocks
TICKERS="KLBF,TLKM,BBRI,PTBA,BJTM,ADMF,TAPG,JPFA,TSPC,BMRI,ASII,ULTJ,HMSP,MNCN"

"$SCREENER_BIN" fetch --tickers "$TICKERS" 2>&1 | tee -a "$PROFILE_DIR/cron/fetch-$(date +%Y%m%d-%H%M%S).log"

exit 0
