"""Near-duplicate detection across multiple sources.

Uses embedding similarity to find articles about the same topic/event
from different sources. Helps merge and deduplicate cross-platform content.
"""

import sys
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional, Set, Tuple

import numpy as np
import requests

QDRANT_URL = "http://localhost:6333"

# Thresholds for similarity detection
EXACT_DUPLICATE_THRESHOLD = 0.98    # Nearly identical text
NEAR_DUPLICATE_THRESHOLD = 0.92     # Same content, minor differences  
SAME_EVENT_THRESHOLD = 0.85         # Same topic/event, different wording
RELATED_THRESHOLD = 0.75            # Related content


def _log(msg: str) -> None:
    sys.stderr.write(f"[NearDup] {msg}\n")
    sys.stderr.flush()


class NearDuplicateDetector:
    """Detect near-duplicates across collections using vector similarity."""
    
    def __init__(
        self,
        qdrant_url: str = QDRANT_URL,
        collections: Optional[List[str]] = None,
    ):
        self.qdrant_url = qdrant_url
        self.collections = collections or [
            "social_intelligence",
            "unlimited_indonesian_current",
            "news-articles",
        ]
    
    def find_similar(
        self,
        vector: List[float],
        threshold: float = SAME_EVENT_THRESHOLD,
        limit: int = 10,
        exclude_ids: Optional[Set[int]] = None,
    ) -> List[Dict[str, Any]]:
        """Find similar items across all collections. Never raises.
        
        Args:
            vector: Query embedding vector
            threshold: Minimum similarity score
            limit: Max results per collection
            exclude_ids: Point IDs to exclude
            
        Returns:
            List of similar items with scores
        """
        all_similar = []
        exclude_ids = exclude_ids or set()
        
        for collection in self.collections:
            try:
                resp = requests.post(
                    f"{self.qdrant_url}/collections/{collection}/points/search",
                    json={
                        "vector": vector,
                        "limit": limit,
                        "with_payload": True,
                        "score_threshold": threshold,
                    },
                    timeout=10,
                )
                
                if resp.status_code != 200:
                    continue
                
                results = resp.json().get("result", [])
                
                for item in results:
                    point_id = item.get("id")
                    if point_id in exclude_ids:
                        continue
                    
                    all_similar.append({
                        "id": point_id,
                        "score": item.get("score", 0),
                        "collection": collection,
                        "payload": item.get("payload", {}),
                    })
                    
            except Exception as e:
                _log(f"Error searching {collection}: {e}")
        
        # Sort by score descending
        all_similar.sort(key=lambda x: x["score"], reverse=True)
        return all_similar
    
    def classify_similarity(self, score: float) -> str:
        """Classify similarity score into category."""
        if score >= EXACT_DUPLICATE_THRESHOLD:
            return "exact_duplicate"
        elif score >= NEAR_DUPLICATE_THRESHOLD:
            return "near_duplicate"
        elif score >= SAME_EVENT_THRESHOLD:
            return "same_event"
        elif score >= RELATED_THRESHOLD:
            return "related"
        else:
            return "different"
    
    def check_duplicate(
        self,
        vector: List[float],
        threshold: float = NEAR_DUPLICATE_THRESHOLD,
    ) -> Tuple[bool, Optional[Dict]]:
        """Check if item is a duplicate. Returns (is_dup, existing_item).
        
        Args:
            vector: Embedding of item to check
            threshold: Similarity threshold for duplicate
            
        Returns:
            (True, existing_item) if duplicate found, (False, None) otherwise
        """
        similar = self.find_similar(vector, threshold=threshold, limit=1)
        
        if similar and similar[0]["score"] >= threshold:
            return True, similar[0]
        
        return False, None
    
    def find_cross_source_matches(
        self,
        collection: str,
        limit: int = 100,
        threshold: float = SAME_EVENT_THRESHOLD,
    ) -> List[Dict[str, Any]]:
        """Find items in one collection that match items in other collections.
        
        Useful for finding the same news story across different platforms.
        
        Args:
            collection: Source collection to check
            limit: Max items to check from source
            threshold: Similarity threshold
            
        Returns:
            List of cross-source match groups
        """
        _log(f"Finding cross-source matches for {collection}")
        
        # Get items from source collection
        try:
            resp = requests.post(
                f"{self.qdrant_url}/collections/{collection}/points/scroll",
                json={
                    "limit": limit,
                    "with_payload": True,
                    "with_vectors": True,
                },
                timeout=30,
            )
            
            if resp.status_code != 200:
                _log(f"Failed to scroll {collection}")
                return []
            
            points = resp.json().get("result", {}).get("points", [])
            
        except Exception as e:
            _log(f"Error scrolling {collection}: {e}")
            return []
        
        # Other collections to search
        other_collections = [c for c in self.collections if c != collection]
        
        matches = []
        
        for point in points:
            vector = point.get("vector", [])
            if not vector:
                continue
            
            point_id = point.get("id")
            payload = point.get("payload", {})
            
            # Search other collections
            cross_matches = []
            
            for other_coll in other_collections:
                try:
                    resp = requests.post(
                        f"{self.qdrant_url}/collections/{other_coll}/points/search",
                        json={
                            "vector": vector,
                            "limit": 3,
                            "with_payload": True,
                            "score_threshold": threshold,
                        },
                        timeout=10,
                    )
                    
                    if resp.status_code == 200:
                        results = resp.json().get("result", [])
                        for r in results:
                            cross_matches.append({
                                "collection": other_coll,
                                "id": r.get("id"),
                                "score": r.get("score"),
                                "title": r.get("payload", {}).get("title", ""),
                                "source": r.get("payload", {}).get("source", ""),
                            })
                            
                except Exception:
                    continue
            
            if cross_matches:
                matches.append({
                    "source_collection": collection,
                    "source_id": point_id,
                    "source_title": payload.get("title", ""),
                    "source_source": payload.get("source", ""),
                    "cross_matches": cross_matches,
                })
        
        _log(f"Found {len(matches)} items with cross-source matches")
        return matches
    
    def merge_duplicates(
        self,
        items: List[Dict[str, Any]],
        vectors: List[List[float]],
        threshold: float = NEAR_DUPLICATE_THRESHOLD,
    ) -> List[Dict[str, Any]]:
        """Merge duplicate items, keeping the best version.
        
        Args:
            items: List of items to deduplicate
            vectors: Corresponding embedding vectors
            threshold: Similarity threshold for merging
            
        Returns:
            Deduplicated list with merged metadata
        """
        if len(items) != len(vectors):
            _log("Items and vectors length mismatch")
            return items
        
        n = len(items)
        merged = [False] * n
        result = []
        
        for i in range(n):
            if merged[i]:
                continue
            
            # Find all duplicates of item i
            group = [i]
            vec_i = np.array(vectors[i])
            
            for j in range(i + 1, n):
                if merged[j]:
                    continue
                
                vec_j = np.array(vectors[j])
                similarity = np.dot(vec_i, vec_j) / (
                    np.linalg.norm(vec_i) * np.linalg.norm(vec_j)
                )
                
                if similarity >= threshold:
                    group.append(j)
                    merged[j] = True
            
            # Merge group into single item
            if len(group) == 1:
                result.append(items[i])
            else:
                # Pick best item (highest score or most complete)
                best_idx = max(group, key=lambda idx: (
                    items[idx].get("score", 0),
                    len(items[idx].get("description", "")),
                ))
                
                merged_item = items[best_idx].copy()
                
                # Aggregate sources
                sources = set()
                for idx in group:
                    src = items[idx].get("source", "")
                    if src:
                        sources.add(src)
                
                merged_item["merged_sources"] = list(sources)
                merged_item["merge_count"] = len(group)
                
                result.append(merged_item)
        
        _log(f"Merged {n} items into {len(result)}")
        return result


def deduplicate_batch(
    items: List[Dict[str, Any]],
    encoder,
    threshold: float = NEAR_DUPLICATE_THRESHOLD,
) -> List[Dict[str, Any]]:
    """Convenience function to deduplicate a batch of items.
    
    Args:
        items: Items to deduplicate
        encoder: SentenceTransformer encoder
        threshold: Similarity threshold
        
    Returns:
        Deduplicated items
    """
    if not items:
        return []
    
    # Generate embeddings
    texts = [
        f"{item.get('title', '')}. {item.get('description', '')}"
        for item in items
    ]
    
    try:
        vectors = encoder.encode(texts).tolist()
    except Exception as e:
        _log(f"Encoding error: {e}")
        return items
    
    detector = NearDuplicateDetector()
    return detector.merge_duplicates(items, vectors, threshold)


if __name__ == "__main__":
    print("=== Near-Duplicate Detection Test ===")
    
    detector = NearDuplicateDetector()
    
    print("\n=== Cross-Source Matches (social_intelligence) ===")
    matches = detector.find_cross_source_matches(
        "social_intelligence",
        limit=20,
        threshold=0.80,
    )
    
    for match in matches[:5]:
        print(f"\n{match['source_source']}: {match['source_title'][:50]}")
        for cross in match["cross_matches"][:2]:
            print(f"  → [{cross['score']:.2f}] {cross['source']}: {cross['title'][:40]}")
