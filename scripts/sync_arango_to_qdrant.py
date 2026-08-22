#!/usr/bin/env python3
"""
Incremental ArangoDB → Qdrant Sync

Designed to run as a cron job. Finds articles in ArangoDB that aren't yet in Qdrant
and embeds them incrementally.

Usage:
    python scripts/sync_arango_to_qdrant.py              # Sync new articles
    python scripts/sync_arango_to_qdrant.py --limit 500  # Limit per run
    python scripts/sync_arango_to_qdrant.py --dry-run    # Preview only
"""

import argparse
import hashlib
import sys
import time
from datetime import datetime
from typing import Optional

import requests
from arango import ArangoClient
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, PointStruct, VectorParams

# Config - can override via env vars
ARANGO_URL = "http://localhost:8529"
ARANGO_DB = "news_analysis"
ARANGO_USER = "root"
ARANGO_PASS = "arangodb"
ARANGO_COLLECTION = "articles"

TEI_URL = "http://localhost:8082"

QDRANT_URL = "http://localhost:6333"
QDRANT_COLLECTION = "news_articles"
VECTOR_DIM = 768

# Default batch size and limit per run
DEFAULT_BATCH_SIZE = 32
DEFAULT_LIMIT = 500  # Process max 500 per cron run to avoid long-running jobs


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


def sync(
    dry_run: bool = False,
    batch_size: int = DEFAULT_BATCH_SIZE,
    limit: int = DEFAULT_LIMIT,
):
    """Incremental sync from ArangoDB to Qdrant"""
    start_time = time.time()
    
    # Connect
    arango = ArangoClient(hosts=ARANGO_URL)
    db = arango.db(ARANGO_DB, username=ARANGO_USER, password=ARANGO_PASS)
    qdrant = QdrantClient(url=QDRANT_URL)
    
    # Ensure Qdrant collection exists
    collections = [c.name for c in qdrant.get_collections().collections]
    if QDRANT_COLLECTION not in collections:
        print(f"📦 Creating Qdrant collection: {QDRANT_COLLECTION}")
        if not dry_run:
            qdrant.create_collection(
                collection_name=QDRANT_COLLECTION,
                vectors_config=VectorParams(size=VECTOR_DIM, distance=Distance.COSINE),
            )
    
    # Get existing keys
    existing_keys = get_existing_keys(qdrant)
    print(f"📊 Qdrant has {len(existing_keys)} articles")
    
    # Query ArangoDB for recent articles not yet embedded
    # Sort by timestamp DESC to prioritize recent articles
    query = f"""
    FOR a IN articles 
    SORT a.timestamp DESC 
    LIMIT {limit * 2}
    RETURN a
    """
    
    cursor = db.aql.execute(query)
    articles = [a for a in cursor if a["_key"] not in existing_keys][:limit]
    
    if not articles:
        print("✅ All articles already synced")
        return
    
    print(f"📥 Found {len(articles)} new articles to embed")
    
    # Process in batches
    embedded = 0
    errors = 0
    
    for i in range(0, len(articles), batch_size):
        batch_articles = articles[i:i + batch_size]
        texts = []
        
        for article in batch_articles:
            title = article.get("title", "")
            content = article.get("content", "")
            text = f"{title} {content}".strip()[:8000]
            texts.append(text)
        
        if dry_run:
            print(f"   [DRY RUN] Would embed {len(texts)} articles")
            embedded += len(texts)
            continue
        
        try:
            embeddings = embed_batch(texts)
            
            points = []
            for article, embedding in zip(batch_articles, embeddings):
                point_id = generate_point_id(article["_key"])
                payload = {
                    "arango_key": article["_key"],
                    "title": article.get("title", ""),
                    "content": article.get("content", "")[:2000],
                    "source": article.get("source", ""),
                    "timestamp": article.get("timestamp", ""),
                    "language": article.get("language", ""),
                    "sentiment": article.get("sentiment", 0),
                    "impact": article.get("impact", 0),
                    "symbols_mentioned": article.get("symbols_mentioned", []),
                    "embedded_at": datetime.utcnow().isoformat(),
                }
                points.append(PointStruct(id=point_id, vector=embedding, payload=payload))
            
            qdrant.upsert(collection_name=QDRANT_COLLECTION, points=points)
            embedded += len(points)
            print(f"   ✅ Embedded {len(points)} (total: {embedded})")
            
        except Exception as e:
            errors += len(texts)
            print(f"   ❌ Batch error: {e}")
    
    elapsed = time.time() - start_time
    print(f"\n{'='*40}")
    print(f"Sync complete: {embedded} embedded, {errors} errors in {elapsed:.1f}s")


def main():
    parser = argparse.ArgumentParser(description="Incremental ArangoDB → Qdrant sync")
    parser.add_argument("--dry-run", action="store_true", help="Preview only")
    parser.add_argument("--batch-size", type=int, default=DEFAULT_BATCH_SIZE)
    parser.add_argument("--limit", type=int, default=DEFAULT_LIMIT, help="Max articles per run")
    
    args = parser.parse_args()
    sync(dry_run=args.dry_run, batch_size=args.batch_size, limit=args.limit)


if __name__ == "__main__":
    main()
