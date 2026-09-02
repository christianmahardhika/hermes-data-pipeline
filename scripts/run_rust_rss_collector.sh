#!/bin/bash
# Wrapper to execute Rust RSS Collector via cargo
set -e

CDIR="/home/ctianm/hermes-data-pipeline"
cd "$CDIR"

export ARANGO_URL="http://localhost:8529"
export ARANGO_DATABASE="news_analysis"
export ARANGO_USERNAME="root"
export ARANGO_PASSWORD=""

# Build release binary if not present or run debug fast
./target/debug/rss_collector
