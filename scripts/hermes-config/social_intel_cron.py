#!/usr/bin/env python3
"""Social Intelligence Cron Job — Periodic Collection.

Collects from all available sources and stores in Qdrant.
Designed to run every 30 minutes via Hermes cron.

Usage:
    python social_intel_cron.py
    python social_intel_cron.py --topics "AI,Indonesia,startup"
"""

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

# Add scripts to path
sys.path.insert(0, str(Path(__file__).parent))

from social_intel import (
    hackernews, reddit, youtube, x_twitter,
    NearDuplicateDetector,
)
from social_intel.collector import SocialIntelCollector

# Default topics to monitor
DEFAULT_TOPICS = [
    # HackerNews: tech + business
    "AI artificial intelligence",
    "machine learning LLM",
    "startup funding venture capital",
    "business strategy economics",
    # YouTube: business, politics, tech, podcasts
    "tech news podcast",
    "business finance podcast",
    "geopolitics world news",
    # Reddit: global news, conspiracy fact-based
    "world news geopolitics",
    "conspiracy theory evidence",
]

# Subreddits by topic
TOPIC_SUBREDDITS = {
    # Tech & AI
    "AI": ["MachineLearning", "artificial", "LocalLLaMA", "singularity"],
    "tech": ["technology", "programming", "Futurology", "technews"],
    # Business & Finance  
    "business": ["business", "Economics", "stocks", "investing", "wallstreetbets"],
    "startup": ["startups", "Entrepreneur", "venturecapital"],
    # Global News & Geopolitics
    "news": ["worldnews", "geopolitics", "anime_titties", "neutralnews"],
    "politics": ["worldpolitics", "GlobalTalk", "InternationalNews"],
    # Conspiracy (fact-based, evidence-focused)
    "conspiracy": ["conspiracy", "HighStrangeness", "actualconspiracies", "UnresolvedMysteries"],
    # Indonesia
    "Indonesia": ["indonesia", "finansial"],
}


def collect_all_sources(
    topics: list,
    depth: str = "quick",
    store: bool = True,
) -> dict:
    """Collect from all available sources for given topics."""
    
    collector = SocialIntelCollector()
    
    results = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "topics": topics,
        "sources": {},
        "stats": {},
    }
    
    for topic in topics:
        topic_results = {
            "hackernews": [],
            "reddit": [],
            "youtube": [],
            "x_twitter": [],
        }
        
        # HackerNews
        try:
            hn_items = collector.collect_hackernews(topic, depth=depth, store=store)
            topic_results["hackernews"] = len(hn_items)
            print(f"[HN] {topic}: {len(hn_items)} items", file=sys.stderr)
        except Exception as e:
            print(f"[HN] {topic} error: {e}", file=sys.stderr)
        
        # Reddit
        try:
            # Get relevant subreddits for topic
            subs = []
            for key, sub_list in TOPIC_SUBREDDITS.items():
                if key.lower() in topic.lower():
                    subs.extend(sub_list)
            
            reddit_items = collector.collect_reddit(
                topic, depth=depth, subreddits=subs or None, store=store
            )
            topic_results["reddit"] = len(reddit_items)
            print(f"[Reddit] {topic}: {len(reddit_items)} items", file=sys.stderr)
        except Exception as e:
            print(f"[Reddit] {topic} error: {e}", file=sys.stderr)
        
        # YouTube (less frequent - only first topic to save time)
        if topic == topics[0]:
            try:
                yt_items = collector.collect_youtube(topic, depth=depth, store=store)
                topic_results["youtube"] = len(yt_items)
                print(f"[YouTube] {topic}: {len(yt_items)} items", file=sys.stderr)
            except Exception as e:
                print(f"[YouTube] {topic} error: {e}", file=sys.stderr)
        
        # X/Twitter (if available)
        if x_twitter.is_available():
            try:
                x_items = x_twitter.search_tweets(topic, depth=depth)
                # Store manually since collector doesn't have X yet
                for item in x_items:
                    collector._store_item(item)
                topic_results["x_twitter"] = len(x_items)
                print(f"[X] {topic}: {len(x_items)} items", file=sys.stderr)
            except Exception as e:
                print(f"[X] {topic} error: {e}", file=sys.stderr)
        
        results["sources"][topic] = topic_results
    
    results["stats"] = collector.stats
    
    return results


def main():
    parser = argparse.ArgumentParser(description="Social Intelligence Cron")
    parser.add_argument(
        "--topics",
        help="Comma-separated topics to monitor",
    )
    parser.add_argument(
        "--depth",
        choices=["quick", "default", "deep"],
        default="quick",
        help="Collection depth",
    )
    parser.add_argument(
        "--no-store",
        action="store_true",
        help="Don't store in Qdrant",
    )
    
    args = parser.parse_args()
    
    topics = args.topics.split(",") if args.topics else DEFAULT_TOPICS
    
    print(f"=== Social Intelligence Collection ===", file=sys.stderr)
    print(f"Topics: {topics}", file=sys.stderr)
    print(f"Depth: {args.depth}", file=sys.stderr)
    print(f"X/Twitter: {'Available' if x_twitter.is_available() else 'Not configured'}", file=sys.stderr)
    print("", file=sys.stderr)
    
    results = collect_all_sources(
        topics=topics,
        depth=args.depth,
        store=not args.no_store,
    )
    
    # Summary output
    print("\n=== Collection Summary ===")
    print(f"Timestamp: {results['timestamp']}")
    print(f"Total fetched: {results['stats'].get('total_fetched', 0)}")
    print(f"Stored: {results['stats'].get('stored', 0)}")
    print(f"Duplicates skipped: {results['stats'].get('duplicates_skipped', 0)}")
    
    print("\nPer-topic breakdown:")
    for topic, sources in results["sources"].items():
        total = sum(v if isinstance(v, int) else 0 for v in sources.values())
        print(f"  {topic}: {total} items")
        for src, count in sources.items():
            if count:
                print(f"    - {src}: {count}")


if __name__ == "__main__":
    main()
