#!/bin/bash
# Pagupon Finance — Portfolio Intelligence Cron Job
# Schedule: 09:00 & 15:00 WIB (02:00 & 08:00 UTC)
# Job ID: 2be1dce649c1 (Advanced Portfolio Intelligence System)
#
# Unified: screener-fetch + screener-digest merged into Rust pipeline
# Usage: ./portfolio-intelligence-cron.sh [--mock]

set -euo pipefail

PIPELINE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/news-social-intelligence-data-pipeline"

cd "$PIPELINE_DIR"
cargo run --release -- idx-analyst digest "$@"
