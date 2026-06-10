"""Unified Social Intelligence Collector.

Aggregates data from multiple sources and stores in Qdrant with deduplication.
Integrates with existing news pipeline.

Usage:
    python -m social_intel.collector --query "AI" --depth default
    python -m social_intel.collector --front-page
"""

import argparse
import hashlib
import json
import sys
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

import requests
from sentence_transformers import SentenceTransformer

from . import hackernews, reddit, youtube

# Config
QDRANT_URL = "http://localhost:6333"
COLLECTION_NAME = "social_intelligence"
EMBEDDING_MODEL = "sentence-transformers/all-MiniLM-L6-v2"


def _log(msg: str) -> None:
    sys.stderr.write(f"[Collector] {msg}\n")
    sys.stderr.flush()


class SocialIntelCollector:
    """Unified collector for all social intelligence sources."""
    
    def __init__(
        self,
        qdrant_url: str = QDRANT_URL,
        collection: str = COLLECTION_NAME,
    ):
        self.qdrant_url = qdrant_url
        self.collection = collection
        self.encoder = None
        self._setup_collection()
        
        # Stats
        self.stats = {
            "total_fetched": 0,
            "stored": 0,
            "duplicates_skipped": 0,
            "errors": 0,
        }
    
    def _load_encoder(self):
        """Lazy load encoder to save memory when not storing."""
        if self.encoder is None:
            _log(f"Loading encoder: {EMBEDDING_MODEL}")
            self.encoder = SentenceTransformer(EMBEDDING_MODEL)
    
    def _setup_collection(self):
        """Create Qdrant collection if needed."""
        try:
            config = {
                "vectors": {"size": 384, "distance": "Cosine"},
                "optimizers_config": {"default_segment_number": 2},
            }
            resp = requests.put(
                f"{self.qdrant_url}/collections/{self.collection}",
                json=config,
                timeout=10,
            )
            if resp.status_code in (200, 409):
                _log(f"Collection '{self.collection}' ready")
            else:
                _log(f"Collection setup failed: {resp.status_code}")
        except Exception as e:
            _log(f"Collection setup error: {e}")
    
    def _url_hash(self, url: str) -> str:
        """Generate hash from URL for dedup."""
        return hashlib.sha256(url.encode()).hexdigest()[:16]
    
    def _check_exists(self, url_hash: str) -> bool:
        """Check if item already exists in collection."""
        try:
            point_id = int(url_hash, 16)
            resp = requests.get(
                f"{self.qdrant_url}/collections/{self.collection}/points/{point_id}",
                timeout=5,
            )
            return resp.status_code == 200
        except Exception:
            return False
    
    def _store_item(self, item: Dict[str, Any]) -> bool:
        """Store single item in Qdrant. Returns True on success."""
        self._load_encoder()
        
        url_hash = self._url_hash(item.get("url", ""))
        
        # Check for duplicate
        if self._check_exists(url_hash):
            self.stats["duplicates_skipped"] += 1
            return False
        
        # Generate embedding
        text = f"{item.get('title', '')}. {item.get('description', '')}"
        try:
            embedding = self.encoder.encode(text).tolist()
        except Exception as e:
            _log(f"Encoding error: {e}")
            self.stats["errors"] += 1
            return False
        
        # Prepare point
        point = {
            "id": int(url_hash, 16),
            "vector": embedding,
            "payload": {
                "title": item.get("title", "")[:500],
                "description": item.get("description", "")[:500],
                "url": item.get("url", ""),
                "source": item.get("source", ""),
                "author": item.get("author", ""),
                "score": item.get("score", 0),
                "num_comments": item.get("num_comments", 0),
                "date": item.get("date"),
                "relevance": item.get("relevance", 0),
                "content_type": item.get("content_type", ""),
                "collected_at": item.get("collected_at") or datetime.now(timezone.utc).isoformat(),
                "metadata": item.get("metadata", {}),
            },
        }
        
        try:
            resp = requests.put(
                f"{self.qdrant_url}/collections/{self.collection}/points",
                json={"points": [point]},
                timeout=10,
            )
            if resp.status_code == 200:
                self.stats["stored"] += 1
                return True
            else:
                _log(f"Storage error: {resp.status_code}")
                self.stats["errors"] += 1
                return False
        except Exception as e:
            _log(f"Storage error: {e}")
            self.stats["errors"] += 1
            return False
    
    def collect_hackernews(
        self,
        query: Optional[str] = None,
        depth: str = "default",
        store: bool = True,
    ) -> List[Dict[str, Any]]:
        """Collect from HackerNews."""
        _log(f"Collecting HackerNews (query={query}, depth={depth})")
        
        items = []
        
        if query:
            items = hackernews.search_stories(query, depth=depth)
        else:
            # Get front page and top stories
            items = hackernews.get_front_page(30)
            items.extend(hackernews.get_top_stories("week", 20))
        
        self.stats["total_fetched"] += len(items)
        
        if store:
            for item in items:
                self._store_item(item)
        
        return items
    
    def collect_reddit(
        self,
        query: str,
        depth: str = "default",
        subreddits: Optional[List[str]] = None,
        store: bool = True,
    ) -> List[Dict[str, Any]]:
        """Collect from Reddit."""
        _log(f"Collecting Reddit (query={query}, depth={depth})")
        
        items = reddit.search_reddit(query, depth=depth, subreddits=subreddits)
        self.stats["total_fetched"] += len(items)
        
        if store:
            for item in items:
                self._store_item(item)
        
        return items
    
    def collect_youtube(
        self,
        query: str,
        depth: str = "default",
        store: bool = True,
    ) -> List[Dict[str, Any]]:
        """Collect from YouTube."""
        _log(f"Collecting YouTube (query={query}, depth={depth})")
        
        items = youtube.search_youtube(query, depth=depth)
        self.stats["total_fetched"] += len(items)
        
        if store:
            for item in items:
                self._store_item(item)
        
        return items
    
    def collect_all(
        self,
        query: str,
        depth: str = "default",
        subreddits: Optional[List[str]] = None,
        store: bool = True,
    ) -> Dict[str, List[Dict[str, Any]]]:
        """Collect from all sources in parallel."""
        _log(f"Collecting ALL sources (query={query}, depth={depth})")
        
        results = {}
        
        with ThreadPoolExecutor(max_workers=3) as executor:
            hn_future = executor.submit(
                self.collect_hackernews, query, depth, store
            )
            reddit_future = executor.submit(
                self.collect_reddit, query, depth, subreddits, store
            )
            yt_future = executor.submit(
                self.collect_youtube, query, depth, store
            )
            
            results["hackernews"] = hn_future.result()
            results["reddit"] = reddit_future.result()
            results["youtube"] = yt_future.result()
        
        return results
    
    def print_stats(self):
        """Print collection statistics."""
        print("\n=== Collection Stats ===")
        print(f"Total fetched: {self.stats['total_fetched']}")
        print(f"Stored: {self.stats['stored']}")
        print(f"Duplicates skipped: {self.stats['duplicates_skipped']}")
        print(f"Errors: {self.stats['errors']}")


def main():
    parser = argparse.ArgumentParser(description="Social Intelligence Collector")
    parser.add_argument("--query", "-q", help="Search query")
    parser.add_argument(
        "--depth",
        choices=["quick", "default", "deep"],
        default="default",
        help="Collection depth",
    )
    parser.add_argument(
        "--source",
        choices=["all", "hackernews", "reddit", "youtube"],
        default="all",
        help="Source to collect from",
    )
    parser.add_argument(
        "--subreddits",
        nargs="+",
        help="Subreddits to search (for Reddit)",
    )
    parser.add_argument(
        "--front-page",
        action="store_true",
        help="Get HN front page instead of searching",
    )
    parser.add_argument(
        "--no-store",
        action="store_true",
        help="Don't store in Qdrant, just print",
    )
    parser.add_argument(
        "--collection",
        default=COLLECTION_NAME,
        help="Qdrant collection name",
    )
    
    args = parser.parse_args()
    
    if not args.query and not args.front_page:
        parser.error("--query or --front-page required")
    
    collector = SocialIntelCollector(collection=args.collection)
    store = not args.no_store
    
    if args.front_page:
        items = collector.collect_hackernews(query=None, store=store)
        print(f"\n=== HackerNews Front Page ({len(items)} items) ===")
        for item in items[:10]:
            print(f"  [{item['score']}] {item['title'][:70]}")
    elif args.source == "all":
        results = collector.collect_all(
            args.query,
            depth=args.depth,
            subreddits=args.subreddits,
            store=store,
        )
        for source, items in results.items():
            print(f"\n=== {source.upper()} ({len(items)} items) ===")
            for item in items[:5]:
                score = item.get("score", 0)
                rel = item.get("relevance", 0)
                print(f"  [{score:>5}|{rel:.2f}] {item['title'][:60]}")
    elif args.source == "hackernews":
        items = collector.collect_hackernews(args.query, args.depth, store)
        print(f"\n=== HackerNews ({len(items)} items) ===")
        for item in items[:10]:
            print(f"  [{item['score']}] {item['title'][:70]}")
    elif args.source == "reddit":
        items = collector.collect_reddit(
            args.query, args.depth, args.subreddits, store
        )
        print(f"\n=== Reddit ({len(items)} items) ===")
        for item in items[:10]:
            print(f"  [r/{item.get('subreddit', '?')}] {item['title'][:60]}")
    elif args.source == "youtube":
        items = collector.collect_youtube(args.query, args.depth, store)
        print(f"\n=== YouTube ({len(items)} items) ===")
        for item in items[:10]:
            views = item.get("metadata", {}).get("view_count", 0) or 0
            print(f"  [{views:>10,} views] {item['title'][:50]}")
    
    collector.print_stats()


if __name__ == "__main__":
    main()
