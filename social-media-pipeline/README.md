# Social Media Pipeline

Collects and analyzes social media content for sentiment and trend analysis.

## Data Sources

### X (Twitter)
- Keyword/hashtag monitoring
- Account tracking
- Trending topics

### Reddit
- Subreddit monitoring (r/indonesia, r/finansial, r/stocks)
- Post and comment sentiment
- Trending discussions

## Architecture

```
┌─────────────────┐
│  Social APIs    │
│  (X, Reddit)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Collector     │  Rate-limited API calls
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Analyzer      │  Sentiment, entity extraction
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Embedder      │  TEI multilingual-e5-base
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Qdrant       │  Vector storage + metadata
└─────────────────┘
```

## Features (Planned)

- [ ] X/Twitter keyword collector (xurl CLI)
- [ ] Reddit subreddit collector
- [ ] Sentiment analysis (LLM-based)
- [ ] Entity extraction (companies, people, events)
- [ ] Trend detection
- [ ] Alert on viral content

## Requirements

- Python 3.10+
- xurl CLI (for X/Twitter)
- PRAW (Reddit API)
- TEI embedding service
- Qdrant vector database

## Usage

```bash
# Install deps
pip install -r requirements.txt

# Collect from X
python collector.py x --keywords "IHSG,saham,investasi"

# Collect from Reddit
python collector.py reddit --subreddits "indonesia,finansial"

# Run daemon
python collector.py daemon --interval 15m
```

## Environment Variables

```bash
# X/Twitter (via xurl)
X_AUTH_TOKEN=

# Reddit
REDDIT_CLIENT_ID=
REDDIT_CLIENT_SECRET=
REDDIT_USER_AGENT=

# Services
TEI_URL=http://localhost:8082
QDRANT_URL=http://localhost:6333
```

## Rate Limits

| Platform | Limit | Strategy |
|----------|-------|----------|
| X Free | 1500/15min | Batch, cache |
| Reddit | 60/min | Queue, backoff |
