#!/usr/bin/env python3
"""
Backfill ArangoDB articles to Qdrant

Reads articles from ArangoDB news_analysis.articles, generates embeddings via TEI,
and upserts to Qdrant news_articles collection.

Usage:
    python scripts/backfill_arango_to_qdrant.py --dry-run     # Preview only
    python scripts/backfill_arango_to_qdrant.py --execute     # Run backfill
    python scripts/backfill_arango_to_qdrant.py --execute --batch-size 50 --limit 1000
"""

import argparse
import hashlib
import json
import sys
import time
from datetime import datetime
from typing import Optional

import requests
from arango import ArangoClient
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, PointStruct, VectorParams

# Config
ARANGO_URL = "http://localhost:8529"
ARANGO_DB = "news_analysis"
ARANGO_USER = "root"
ARANGO_PASS = "arangodb"
ARANGO_COLLECTION = "articles"

TEI_URL = "http://localhost:8082"

QDRANT_URL = "http://localhost:6333"
QDRANT_COLLECTION = "news_articles"
VECTOR_DIM = 768


class BackfillStats:
    def __init__(self):
        self.total = 0
        self.embedded = 0
        self.skipped_exists = 0
        self.skipped_no_content = 0
        self.errors = 0
        self.start_time = time.time()

    def summary(self) -> str:
        elapsed = time.time() - self.start_time
        rate = self.embedded / elapsed if elapsed > 0 else 0
        return (
            f"\n{'='*50}\n"
            f"Backfill Summary\n"
            f"{'='*50}\n"
            f"Total processed:    {self.total}\n"
            f"Embedded to Qdrant: {self.embedded}\n"
            f"Skipped (exists):   {self.skipped_exists}\n"
            f"Skipped (empty):    {self.skipped_no_content}\n"
            f"Errors:             {self.errors}\n"
            f"Time:               {elapsed:.1f}s\n"
            f"Rate:               {rate:.1f} articles/sec\n"
        )


def get_existing_keys(qdrant: QdrantClient) -> set:
    """Get all existing article keys from Qdrant"""
    existing = set()
    offset = None
    
    while True:
        results, next_offset = qdrant.scroll(
            collection_name=QDRANT_COLLECTION,
            limit=1000,
            offset=offset,
            with_payload=["arango_key"],
        )
        
        for point in results:
            if point.payload and "arango_key" in point.payload:
                existing.add(point.payload["arango_key"])
        
        if next_offset is None:
            break
        offset = next_offset
    
    return existing


def embed_batch(texts: list[str]) -> list[list[float]]:
    """Generate embeddings via TEI"""
    response = requests.post(
        f"{TEI_URL}/embed",
        json={"inputs": texts},
        timeout=120,
    )
    response.raise_for_status()
    return response.json()


def generate_point_id(arango_key: str) -> str:
    """Generate deterministic UUID from arango key"""
    return hashlib.md5(f"arango:{arango_key}".encode()).hexdigest()


def backfill(
    dry_run: bool = True,
    batch_size: int = 32,
    limit: Optional[int] = None,
):
    stats = BackfillStats()
    
    # Connect to ArangoDB
    print(f"🔌 Connecting to ArangoDB at {ARANGO_URL}...")
    arango = ArangoClient(hosts=ARANGO_URL)
    db = arango.db(ARANGO_DB, username=ARANGO_USER, password=ARANGO_PASS)
    collection = db.collection(ARANGO_COLLECTION)
    
    # Connect to Qdrant
    print(f"🔌 Connecting to Qdrant at {QDRANT_URL}...")
    qdrant = QdrantClient(url=QDRANT_URL)
    
    # Ensure collection exists
    collections = [c.name for c in qdrant.get_collections().collections]
    if QDRANT_COLLECTION not in collections:
        print(f"📦 Creating Qdrant collection: {QDRANT_COLLECTION}")
        if not dry_run:
            qdrant.create_collection(
                collection_name=QDRANT_COLLECTION,
                vectors_config=VectorParams(size=VECTOR_DIM, distance=Distance.COSINE),
            )
    
    # Get existing keys to skip duplicates
    print("📋 Fetching existing keys from Qdrant...")
    existing_keys = get_existing_keys(qdrant) if not dry_run else set()
    print(f"   Found {len(existing_keys)} existing articles")
    
    # Query ArangoDB
    query = "FOR a IN articles SORT a.timestamp DESC RETURN a"
    if limit:
        query = f"FOR a IN articles SORT a.timestamp DESC LIMIT {limit} RETURN a"
    
    print(f"📥 Fetching articles from ArangoDB...")
    cursor = db.aql.execute(query, batch_size=batch_size)
    articles = list(cursor)
    print(f"   Found {len(articles)} articles")
    
    # Process in batches
    batch = []
    batch_articles = []
    
    for article in articles:
        stats.total += 1
        arango_key = article["_key"]
        
        # Skip if exists
        if arango_key in existing_keys:
            stats.skipped_exists += 1
            continue
        
        # Skip if no content
        content = article.get("content", "")
        title = article.get("title", "")
        if not content and not title:
            stats.skipped_no_content += 1
            continue
        
        # Prepare text for embedding
        text = f"{title} {content}".strip()
        if len(text) > 8000:  # TEI limit
            text = text[:8000]
        
        batch.append(text)
        batch_articles.append(article)
        
        # Process batch
        if len(batch) >= batch_size:
            process_batch(qdrant, batch, batch_articles, stats, dry_run)
            batch = []
            batch_articles = []
    
    # Process remaining
    if batch:
        process_batch(qdrant, batch, batch_articles, stats, dry_run)
    
    print(stats.summary())


def process_batch(
    qdrant: QdrantClient,
    texts: list[str],
    articles: list[dict],
    stats: BackfillStats,
    dry_run: bool,
):
    """Process a batch of articles"""
    if dry_run:
        print(f"   [DRY RUN] Would embed {len(texts)} articles")
        stats.embedded += len(texts)
        return
    
    try:
        # Generate embeddings
        embeddings = embed_batch(texts)
        
        # Build points
        points = []
        for article, embedding in zip(articles, embeddings):
            point_id = generate_point_id(article["_key"])
            
            payload = {
                "arango_key": article["_key"],
                "title": article.get("title", ""),
                "content": article.get("content", "")[:2000],  # Truncate for storage
                "source": article.get("source", ""),
                "timestamp": article.get("timestamp", ""),
                "language": article.get("language", ""),
                "sentiment": article.get("sentiment", 0),
                "impact": article.get("impact", 0),
                "symbols_mentioned": article.get("symbols_mentioned", []),
                "elite_overproduction_score": article.get("elite_overproduction_score", 0),
                "embedded_at": datetime.utcnow().isoformat(),
            }
            
            points.append(PointStruct(
                id=point_id,
                vector=embedding,
                payload=payload,
            ))
        
        # Upsert to Qdrant
        qdrant.upsert(collection_name=QDRANT_COLLECTION, points=points)
        stats.embedded += len(points)
        print(f"   ✅ Embedded {len(points)} articles (total: {stats.embedded})")
        
    except Exception as e:
        stats.errors += len(texts)
        print(f"   ❌ Batch error: {e}")


def main():
    parser = argparse.ArgumentParser(description="Backfill ArangoDB to Qdrant")
    parser.add_argument("--dry-run", action="store_true", help="Preview only")
    parser.add_argument("--execute", action="store_true", help="Execute backfill")
    parser.add_argument("--batch-size", type=int, default=32, help="Batch size (default: 32)")
    parser.add_argument("--limit", type=int, help="Limit articles to process")
    
    args = parser.parse_args()
    
    if not args.dry_run and not args.execute:
        print("❌ Must specify --dry-run or --execute")
        sys.exit(1)
    
    backfill(
        dry_run=args.dry_run,
        batch_size=args.batch_size,
        limit=args.limit,
    )


if __name__ == "__main__":
    main()
