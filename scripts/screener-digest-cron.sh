#!/bin/bash
# Pagupon Finance — Daily Digest Cron Job (09:00 & 15:00 WIB)
# Real file: ~/.hermes/profiles/pagupon-finance/scripts/screener-digest-cron.sh

set -e

SCREENER_BIN="$HOME/.hermes/profiles/pagupon-finance/screener/target/debug/screener"
PROFILE_DIR="$HOME/.hermes/profiles/pagupon-finance"

# Load environment
cd "$PROFILE_DIR"
export $(cat .env 2>/dev/null | xargs) || true

# Run digest with real data (no --mock)
"$SCREENER_BIN" digest 2>&1 | tee -a "$PROFILE_DIR/cron/digest-$(date +%Y%m%d-%H%M%S).log"

exit 0
