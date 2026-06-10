"""HackerNews scraper using Algolia API (free, no auth required).

Algolia provides full-text search over HN posts and comments.
API docs: https://hn.algolia.com/api

Output format matches our normalized article schema for consistency.
"""

import sys
from concurrent.futures import ThreadPoolExecutor, TimeoutError as FuturesTimeoutError
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional
from urllib.parse import quote_plus

from .lib import get_json, combined_relevance

ALGOLIA_BASE = "https://hn.algolia.com/api/v1"

# Depth-aware limits (matching last30days pattern)
DEPTH_LIMITS = {
    "quick": 10,
    "default": 25,
    "deep": 50,
}

MAX_WORKERS = 4
FETCH_TIMEOUT = 15


def _log(msg: str) -> None:
    sys.stderr.write(f"[HackerNews] {msg}\n")
    sys.stderr.flush()


def _timestamp_to_date(ts: Optional[int]) -> Optional[str]:
    """Convert Unix timestamp to YYYY-MM-DD."""
    if not ts:
        return None
    try:
        dt = datetime.fromtimestamp(ts, tz=timezone.utc)
        return dt.date().isoformat()
    except (ValueError, OSError):
        return None


def _parse_hit(hit: Dict[str, Any], query: str = "") -> Optional[Dict[str, Any]]:
    """Parse Algolia hit into normalized article dict. Never raises."""
    try:
        object_id = hit.get("objectID", "")
        title = hit.get("title") or hit.get("story_title") or ""
        url = hit.get("url") or f"https://news.ycombinator.com/item?id={object_id}"
        
        if not title:
            return None
        
        # Extract text content
        story_text = hit.get("story_text") or ""
        comment_text = hit.get("comment_text") or ""
        description = story_text or comment_text or ""
        
        # Clean HTML from description
        import re
        description = re.sub(r'<[^>]+>', ' ', description)
        description = re.sub(r'\s+', ' ', description).strip()[:500]
        
        created_at = hit.get("created_at_i")
        
        relevance = combined_relevance(query, title, description) if query else 0.0
        
        return {
            "id": object_id,
            "title": title,
            "url": url,
            "description": description,
            "source": "HackerNews",
            "author": hit.get("author", ""),
            "score": hit.get("points") or 0,
            "num_comments": hit.get("num_comments") or 0,
            "created_utc": created_at,
            "date": _timestamp_to_date(created_at),
            "relevance": round(relevance, 3),
            "why_relevant": "Algolia search",
            "collected_at": datetime.now(timezone.utc).isoformat(),
            "content_type": "hackernews",
            "metadata": {
                "hn_id": object_id,
                "tags": hit.get("_tags", []),
            },
        }
    except Exception as e:
        _log(f"Parse error: {e}")
        return None


def search_stories(
    query: str,
    depth: str = "default",
    tags: Optional[str] = None,
) -> List[Dict[str, Any]]:
    """Search HackerNews stories via Algolia. Never raises.
    
    Args:
        query: Search query string
        depth: 'quick', 'default', or 'deep' - controls result limit
        tags: Optional Algolia tags filter (e.g., 'story', 'comment', 'show_hn')
        
    Returns:
        List of normalized article dicts, empty on failure
    """
    limit = DEPTH_LIMITS.get(depth, DEPTH_LIMITS["default"])
    
    # Build URL
    q = quote_plus(query)
    url = f"{ALGOLIA_BASE}/search?query={q}&hitsPerPage={limit}"
    if tags:
        url += f"&tags={tags}"
    
    _log(f"Searching: {query} (depth={depth}, limit={limit})")
    
    data = get_json(url, timeout=FETCH_TIMEOUT)
    if not data:
        _log("No response from Algolia")
        return []
    
    hits = data.get("hits", [])
    results = []
    
    for hit in hits:
        parsed = _parse_hit(hit, query)
        if parsed:
            results.append(parsed)
    
    _log(f"Found {len(results)} results")
    return results


def get_front_page(limit: int = 30) -> List[Dict[str, Any]]:
    """Get current HN front page stories. Never raises.
    
    Args:
        limit: Max stories to return
        
    Returns:
        List of normalized article dicts, empty on failure
    """
    url = f"{ALGOLIA_BASE}/search?tags=front_page&hitsPerPage={limit}"
    
    _log(f"Fetching front page (limit={limit})")
    
    data = get_json(url, timeout=FETCH_TIMEOUT)
    if not data:
        return []
    
    hits = data.get("hits", [])
    results = []
    
    for hit in hits:
        parsed = _parse_hit(hit)
        if parsed:
            results.append(parsed)
    
    _log(f"Got {len(results)} front page stories")
    return results


def get_top_stories(
    period: str = "week",
    limit: int = 30,
) -> List[Dict[str, Any]]:
    """Get top stories by points for a time period. Never raises.
    
    Args:
        period: 'day', 'week', or 'month'
        limit: Max stories to return
        
    Returns:
        List of normalized article dicts sorted by score, empty on failure
    """
    # Calculate timestamp for period
    now = datetime.now(timezone.utc)
    if period == "day":
        seconds_ago = 86400
    elif period == "month":
        seconds_ago = 86400 * 30
    else:  # week
        seconds_ago = 86400 * 7
    
    created_after = int(now.timestamp()) - seconds_ago
    
    url = (
        f"{ALGOLIA_BASE}/search?tags=story"
        f"&numericFilters=created_at_i>{created_after}"
        f"&hitsPerPage={limit}"
    )
    
    _log(f"Fetching top stories (period={period}, limit={limit})")
    
    data = get_json(url, timeout=FETCH_TIMEOUT)
    if not data:
        return []
    
    hits = data.get("hits", [])
    results = []
    
    for hit in hits:
        parsed = _parse_hit(hit)
        if parsed:
            results.append(parsed)
    
    # Sort by score descending
    results.sort(key=lambda x: x.get("score", 0), reverse=True)
    
    _log(f"Got {len(results)} top stories")
    return results


if __name__ == "__main__":
    # Quick test
    import json
    
    print("=== Front Page ===")
    fp = get_front_page(5)
    for item in fp:
        print(f"  [{item['score']}] {item['title'][:60]}")
    
    print("\n=== Search: AI ===")
    results = search_stories("AI", depth="quick")
    for item in results[:5]:
        print(f"  [{item['relevance']:.2f}] {item['title'][:60]}")
