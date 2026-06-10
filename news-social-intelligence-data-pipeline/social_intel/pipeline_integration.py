#!/usr/bin/env python3
"""Pipeline Integration — Merge social_intel with existing news pipeline.

Consolidates social_intelligence collection into news-articles collection
with cross-source deduplication.

Usage:
    python pipeline_integration.py --consolidate
    python pipeline_integration.py --stats
"""

import argparse
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List

import requests

# Add scripts to path
sys.path.insert(0, str(Path(__file__).parent))

from social_intel.near_duplicate import NearDuplicateDetector

QDRANT_URL = "http://localhost:6333"

# Source collections
SOURCE_COLLECTIONS = [
    "social_intelligence",          # HN, Reddit, YouTube, X
    "unlimited_indonesian_current", # Indonesian news RSS
    "unlimited_international_current",  # International news RSS
]

# Target collection
TARGET_COLLECTION = "news-articles"

# Dedup threshold
NEAR_DUP_THRESHOLD = 0.92


def get_collection_stats(name: str) -> Dict:
    """Get collection statistics."""
    try:
        resp = requests.get(f"{QDRANT_URL}/collections/{name}", timeout=10)
        if resp.status_code == 200:
            result = resp.json().get("result", {})
            return {
                "name": name,
                "points_count": result.get("points_count", 0),
                "status": result.get("status", "unknown"),
            }
    except Exception as e:
        return {"name": name, "error": str(e)}
    return {"name": name, "error": "not_found"}


def consolidate_to_target(
    source: str,
    target: str = TARGET_COLLECTION,
    batch_size: int = 100,
    dedup: bool = True,
) -> Dict:
    """Consolidate source collection into target with deduplication.
    
    Returns stats about the consolidation.
    """
    stats = {
        "source": source,
        "target": target,
        "processed": 0,
        "stored": 0,
        "duplicates_skipped": 0,
        "errors": 0,
    }
    
    detector = NearDuplicateDetector(collections=[target]) if dedup else None
    
    # Scroll through source
    offset = None
    
    while True:
        try:
            payload = {
                "limit": batch_size,
                "with_payload": True,
                "with_vectors": True,
            }
            if offset:
                payload["offset"] = offset
            
            resp = requests.post(
                f"{QDRANT_URL}/collections/{source}/points/scroll",
                json=payload,
                timeout=30,
            )
            
            if resp.status_code != 200:
                print(f"Error scrolling {source}: {resp.status_code}", file=sys.stderr)
                break
            
            result = resp.json().get("result", {})
            points = result.get("points", [])
            
            if not points:
                break
            
            for point in points:
                stats["processed"] += 1
                
                point_id = point.get("id")
                vector = point.get("vector", [])
                payload_data = point.get("payload", {})
                
                if not vector:
                    continue
                
                # Check for duplicate in target
                if dedup and detector:
                    is_dup, existing = detector.check_duplicate(
                        vector, threshold=NEAR_DUP_THRESHOLD
                    )
                    if is_dup:
                        stats["duplicates_skipped"] += 1
                        continue
                
                # Add source collection tag
                payload_data["original_collection"] = source
                payload_data["consolidated_at"] = datetime.now(timezone.utc).isoformat()
                
                # Store in target
                try:
                    store_resp = requests.put(
                        f"{QDRANT_URL}/collections/{target}/points",
                        json={
                            "points": [{
                                "id": point_id,
                                "vector": vector,
                                "payload": payload_data,
                            }]
                        },
                        timeout=10,
                    )
                    
                    if store_resp.status_code == 200:
                        stats["stored"] += 1
                    else:
                        stats["errors"] += 1
                        
                except Exception as e:
                    stats["errors"] += 1
            
            # Next page
            offset = result.get("next_page_offset")
            if not offset:
                break
                
        except Exception as e:
            print(f"Error: {e}", file=sys.stderr)
            stats["errors"] += 1
            break
    
    return stats


def print_pipeline_stats():
    """Print statistics for all pipeline collections."""
    print("=== News Pipeline Statistics ===\n")
    
    all_collections = SOURCE_COLLECTIONS + [TARGET_COLLECTION, "news_events", "pagupon-kb"]
    
    total_points = 0
    
    for coll in all_collections:
        stats = get_collection_stats(coll)
        if "error" in stats:
            print(f"  {coll}: {stats['error']}")
        else:
            count = stats["points_count"]
            total_points += count
            print(f"  {coll}: {count:,} points")
    
    print(f"\n  Total: {total_points:,} points")


def main():
    parser = argparse.ArgumentParser(description="Pipeline Integration")
    parser.add_argument(
        "--consolidate",
        action="store_true",
        help="Consolidate source collections into target",
    )
    parser.add_argument(
        "--stats",
        action="store_true",
        help="Print pipeline statistics",
    )
    parser.add_argument(
        "--source",
        help="Specific source collection to consolidate",
    )
    parser.add_argument(
        "--no-dedup",
        action="store_true",
        help="Skip deduplication",
    )
    
    args = parser.parse_args()
    
    if args.stats:
        print_pipeline_stats()
        return
    
    if args.consolidate:
        sources = [args.source] if args.source else SOURCE_COLLECTIONS
        
        print("=== Pipeline Consolidation ===\n")
        
        total_stats = {
            "processed": 0,
            "stored": 0,
            "duplicates_skipped": 0,
            "errors": 0,
        }
        
        for source in sources:
            print(f"Consolidating {source} → {TARGET_COLLECTION}...")
            
            stats = consolidate_to_target(
                source,
                dedup=not args.no_dedup,
            )
            
            print(f"  Processed: {stats['processed']}")
            print(f"  Stored: {stats['stored']}")
            print(f"  Duplicates skipped: {stats['duplicates_skipped']}")
            print(f"  Errors: {stats['errors']}")
            print()
            
            for key in total_stats:
                total_stats[key] += stats.get(key, 0)
        
        print("=== Total ===")
        print(f"  Processed: {total_stats['processed']}")
        print(f"  Stored: {total_stats['stored']}")
        print(f"  Duplicates skipped: {total_stats['duplicates_skipped']}")
        print(f"  Errors: {total_stats['errors']}")
        
        return
    
    # Default: show stats
    print_pipeline_stats()


if __name__ == "__main__":
    main()
