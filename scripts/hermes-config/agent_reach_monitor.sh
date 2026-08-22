#!/bin/bash
# Agent-Reach News Monitor untuk Prof Jiang Analysis
# Auto-collect Indonesian political news every 30 minutes

set -e

# Activate Agent-Reach environment
source ~/.agent-reach-venv/bin/activate

# Setup directories
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BASE_DIR="$HOME/.hermes/profiles/social-politic-lab/data"
NEWS_DIR="$BASE_DIR/news/$TIMESTAMP"
RSS_DIR="$BASE_DIR/rss"

mkdir -p "$NEWS_DIR" "$RSS_DIR"

echo "=== Agent-Reach News Monitor Started at $(date) ==="

# Indonesian Political News Sources
echo "📰 Collecting Indonesian political news..."
curl -s "https://r.jina.ai/https://www.detik.com/tag/politik" > "$NEWS_DIR/detik_politik.md"
curl -s "https://r.jina.ai/https://www.kompas.com/tag/geopolitik" > "$NEWS_DIR/kompas_geopolitik.md"
curl -s "https://r.jina.ai/https://www.republika.co.id/tag/internasional" > "$NEWS_DIR/republika_internasional.md"
curl -s "https://r.jina.ai/https://www.cnnindonesia.com/tag/politik-internasional" > "$NEWS_DIR/cnn_politik_internasional.md"

# International Geopolitical Sources  
echo "🌍 Collecting international geopolitical news..."
curl -s "https://r.jina.ai/https://www.reuters.com/world/" > "$NEWS_DIR/reuters_world.md"
curl -s "https://r.jina.ai/https://apnews.com/hub/politics" > "$NEWS_DIR/ap_politics.md"

# RSS Feed Collection (Agent-Reach built-in support)
echo "📡 Processing RSS feeds..."
# Note: RSS processing akan dihandle oleh Agent-Reach feedparser

# Summary report
TOTAL_FILES=$(find "$NEWS_DIR" -name "*.md" | wc -l)
TOTAL_SIZE=$(du -sh "$NEWS_DIR" | cut -f1)

echo "✅ Collection complete:"
echo "   Files: $TOTAL_FILES articles"  
echo "   Size: $TOTAL_SIZE"
echo "   Location: $NEWS_DIR"

# Optional: Trigger existing MCP knowledge base update
# Uncomment when ready to integrate:
# echo "🔄 Updating knowledge base..."
# hermes mcp social-politic-kb ingest "$NEWS_DIR"/*.md

echo "=== Monitor cycle completed at $(date) ==="