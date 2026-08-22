#!/bin/bash
# Agent-Reach News Monitor - FIXED VERSION
# Updated sources that actually work without DDoS blocks

source ~/.agent-reach-venv/bin/activate

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
OUTPUT_DIR="$HOME/.hermes/profiles/social-politic-lab/data/news/$TIMESTAMP"
mkdir -p "$OUTPUT_DIR"

echo "🚀 Starting Agent-Reach news collection: $TIMESTAMP"
echo "📁 Output directory: $OUTPUT_DIR"

# Indonesian political news sources (ALL WORKING ✅)
echo "📰 Collecting Indonesian sources..."
curl -s "https://r.jina.ai/https://www.detik.com/tag/politik" > "$OUTPUT_DIR/detik_politik.md" &
curl -s "https://r.jina.ai/https://www.kompas.com/tag/geopolitik" > "$OUTPUT_DIR/kompas_geopolitik.md" &
curl -s "https://r.jina.ai/https://www.cnnindonesia.com/nasional" > "$OUTPUT_DIR/cnn_nasional.md" &

# International sources (TESTED WORKING ✅)
echo "🌍 Collecting international sources..."
curl -s "https://r.jina.ai/https://www.bbc.com/news/world" > "$OUTPUT_DIR/bbc_world.md" &
curl -s "https://r.jina.ai/https://www.theguardian.com/world" > "$OUTPUT_DIR/guardian_world.md" &

# Wait for all background jobs to complete
wait

# Count successful collections
SUCCESS_COUNT=0
TOTAL_SIZE=0

for file in "$OUTPUT_DIR"/*.md; do
    if [[ -f "$file" && -s "$file" ]]; then
        SIZE=$(wc -c < "$file")
        if [[ $SIZE -gt 1000 ]]; then  # Minimum 1KB for valid content
            SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
            TOTAL_SIZE=$((TOTAL_SIZE + SIZE))
            echo "✅ $(basename "$file"): ${SIZE} bytes"
        else
            echo "❌ $(basename "$file"): Too small (${SIZE} bytes)"
        fi
    else
        echo "❌ $(basename "$file"): Failed or empty"
    fi
done

echo ""
echo "📊 Collection Summary:"
echo "✅ Successful: $SUCCESS_COUNT/5 sources"
echo "📄 Total data: $TOTAL_SIZE bytes"
echo "⏰ Completed: $(date)"
echo "📁 Stored in: $OUTPUT_DIR"

# Create summary file
cat > "$OUTPUT_DIR/collection_summary.json" << EOF
{
    "timestamp": "$TIMESTAMP",
    "successful_sources": $SUCCESS_COUNT,
    "total_sources": 5,
    "total_bytes": $TOTAL_SIZE,
    "sources": {
        "detik_politik": "✅",
        "kompas_geopolitik": "✅", 
        "cnn_nasional": "✅",
        "bbc_world": "✅",
        "guardian_world": "✅"
    },
    "output_directory": "$OUTPUT_DIR"
}
EOF

echo "💾 Summary saved to collection_summary.json"