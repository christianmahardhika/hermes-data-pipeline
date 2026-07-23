#!/bin/bash
# Enhanced IDX AI Analyst wrapper for Hermes cron integration

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )/tools/enhanced-idx-analyst"
cd "$SCRIPT_DIR"

# Default to portfolio in standard format
MODE="${1:-portfolio}"
FORMAT="${2:-standard}"

python3 idx_ai_analyst_enhanced.py --$MODE --format $FORMAT
