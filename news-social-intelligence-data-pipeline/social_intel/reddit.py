"""Reddit scraper using public RSS/Atom feeds (no API key required).

Reddit's .json endpoints return 403 (anti-bot), but RSS feeds still work.
This module uses RSS for discovery, matching the last30days pattern.

Output format matches our normalized article schema.
"""

import sys
import xml.etree.ElementTree as ET
from concurrent.futures import ThreadPoolExecutor, TimeoutError as FuturesTimeoutError
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional
from urllib.parse import quote_plus

from .lib import get_text, combined_relevance

ATOM = "{http://www.w3.org/2005/Atom}"

# Depth-aware limits
DEPTH_LIMITS = {
    "quick": 10,
    "default": 25,
    "deep": 50,
}

# Listing sorts per subreddit
LISTING_SORTS = {
    "quick": ["top"],
    "default": ["top", "hot"],
    "deep": ["top", "hot", "new"],
}

MAX_WORKERS = 4
FEED_TIMEOUT = 15


def _log(msg: str) -> None:
    sys.stderr.write(f"[Reddit] {msg}\n")
    sys.stderr.flush()


def _iso_to_date(value: Optional[str]) -> Optional[str]:
    """Parse ISO-8601 timestamp to YYYY-MM-DD."""
    if not value:
        return None
    try:
        dt = datetime.fromisoformat(value.strip())
        return dt.date().isoformat()
    except (ValueError, TypeError):
        return None


def _iso_to_epoch(value: Optional[str]) -> Optional[float]:
    """Parse ISO-8601 timestamp to Unix epoch."""
    if not value:
        return None
    try:
        dt = datetime.fromisoformat(value.strip())
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return dt.timestamp()
    except (ValueError, TypeError):
        return None


def _subreddit_from(category: str, url: str) -> str:
    """Extract subreddit name from category or URL."""
    if category:
        return category
    parts = url.split("/r/", 1)
    if len(parts) == 2:
        return parts[1].split("/", 1)[0]
    return ""


def _parse_feed(xml_text: str, query: str = "") -> List[Dict[str, Any]]:
    """Parse Atom feed into normalized article dicts. Never raises."""
    if not xml_text:
        return []
    
    try:
        root = ET.fromstring(xml_text)
    except ET.ParseError as e:
        _log(f"Feed parse error: {e}")
        return []
    
    import re
    posts = []
    
    for entry in root.iter(f"{ATOM}entry"):
        try:
            # Extract URL
            link_el = entry.find(f"{ATOM}link")
            url = link_el.get("href", "").strip() if link_el is not None else ""
            if not url or "/comments/" not in url:
                continue
            
            # Extract title
            title_el = entry.find(f"{ATOM}title")
            title = (title_el.text or "").strip() if title_el is not None else ""
            if not title:
                continue
            
            # Extract author
            author = ""
            author_el = entry.find(f"{ATOM}author/{ATOM}name")
            if author_el is not None and author_el.text:
                author = author_el.text.strip().removeprefix("/u/").removeprefix("u/")
            if author in ("[deleted]", "[removed]", ""):
                author = "[deleted]"
            
            # Extract subreddit
            cat_el = entry.find(f"{ATOM}category")
            category = cat_el.get("term", "").strip() if cat_el is not None else ""
            subreddit = _subreddit_from(category, url)
            
            # Extract timestamp
            updated_el = entry.find(f"{ATOM}updated")
            updated = (updated_el.text or "").strip() if updated_el is not None else ""
            
            # Extract content/description
            content_el = entry.find(f"{ATOM}content")
            description = ""
            if content_el is not None and content_el.text:
                description = re.sub(r"<[^>]+>", " ", content_el.text)
                description = re.sub(r"\s+", " ", description).strip()[:500]
            
            # Calculate relevance
            relevance = combined_relevance(query, title, description) if query else 0.0
            
            # Extract post ID from URL
            post_id = ""
            if "/comments/" in url:
                parts = url.split("/comments/")
                if len(parts) > 1:
                    post_id = parts[1].split("/")[0]
            
            posts.append({
                "id": post_id,
                "title": title,
                "url": url,
                "description": description,
                "source": f"Reddit/r/{subreddit}" if subreddit else "Reddit",
                "author": author,
                "subreddit": subreddit,
                "score": 0,  # Not available in RSS, needs enrichment
                "num_comments": 0,  # Not available in RSS
                "created_utc": _iso_to_epoch(updated),
                "date": _iso_to_date(updated),
                "relevance": round(relevance, 3),
                "why_relevant": "Reddit RSS",
                "collected_at": datetime.now(timezone.utc).isoformat(),
                "content_type": "reddit",
                "metadata": {
                    "subreddit": subreddit,
                },
            })
        except Exception:
            continue
    
    return posts


def _build_urls(query: str, depth: str, subreddits: Optional[List[str]]) -> List[str]:
    """Build RSS feed URLs to fetch."""
    q = quote_plus(query)
    urls = [
        f"https://www.reddit.com/search.rss?q={q}&sort=relevance&t=month"
    ]
    
    for raw_sub in (subreddits or []):
        sub = raw_sub.removeprefix("r/").strip()
        if not sub:
            continue
        # Subreddit search
        urls.append(
            f"https://www.reddit.com/r/{sub}/search.rss"
            f"?q={q}&restrict_sr=on&sort=relevance&t=month"
        )
        # Listing feeds
        for sort in LISTING_SORTS.get(depth, LISTING_SORTS["default"]):
            urls.append(f"https://www.reddit.com/r/{sub}/{sort}.rss?t=month")
    
    return urls


def _fetch_feed(url: str, query: str) -> List[Dict[str, Any]]:
    """Fetch and parse one feed. Never raises."""
    try:
        text = get_text(url, timeout=FEED_TIMEOUT, accept="application/atom+xml")
        return _parse_feed(text, query) if text else []
    except Exception as e:
        _log(f"Feed fetch failed for {url}: {e}")
        return []


def search_reddit(
    query: str,
    depth: str = "default",
    subreddits: Optional[List[str]] = None,
) -> List[Dict[str, Any]]:
    """Search Reddit via RSS feeds. Never raises.
    
    Args:
        query: Search query string
        depth: 'quick', 'default', or 'deep' - controls feeds and limits
        subreddits: Optional list of subreddits to search (without r/ prefix)
        
    Returns:
        List of normalized article dicts, deduped by URL
    """
    limit = DEPTH_LIMITS.get(depth, DEPTH_LIMITS["default"])
    urls = _build_urls(query, depth, subreddits)
    
    _log(f"Searching: {query} across {len(urls)} feeds (depth={depth})")
    
    all_posts = []
    seen_urls = set()
    
    # Parallel fetch
    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
        futures = {executor.submit(_fetch_feed, url, query): url for url in urls}
        
        for future in futures:
            try:
                posts = future.result(timeout=FEED_TIMEOUT + 5)
                for post in posts:
                    if post["url"] not in seen_urls:
                        seen_urls.add(post["url"])
                        all_posts.append(post)
            except FuturesTimeoutError:
                _log(f"Timeout for {futures[future]}")
            except Exception as e:
                _log(f"Error: {e}")
    
    # Sort by relevance, limit results
    all_posts.sort(key=lambda x: x.get("relevance", 0), reverse=True)
    results = all_posts[:limit]
    
    _log(f"Found {len(results)} unique results")
    return results


def get_subreddit_hot(
    subreddit: str,
    limit: int = 25,
) -> List[Dict[str, Any]]:
    """Get hot posts from a subreddit. Never raises.
    
    Args:
        subreddit: Subreddit name (without r/ prefix)
        limit: Max posts to return
        
    Returns:
        List of normalized article dicts
    """
    sub = subreddit.removeprefix("r/").strip()
    url = f"https://www.reddit.com/r/{sub}/hot.rss"
    
    _log(f"Fetching r/{sub} hot posts")
    
    posts = _fetch_feed(url, "")
    return posts[:limit]


# Popular subreddits for different topics
TECH_SUBREDDITS = [
    "technology", "programming", "MachineLearning", "artificial",
    "Python", "golang", "rust", "webdev", "devops",
]

FINANCE_SUBREDDITS = [
    "investing", "stocks", "wallstreetbets", "CryptoCurrency",
    "personalfinance", "FinancialPlanning",
]

INDONESIA_SUBREDDITS = [
    "indonesia", "finansial",
]


if __name__ == "__main__":
    print("=== r/technology hot ===")
    hot = get_subreddit_hot("technology", 5)
    for item in hot:
        print(f"  {item['title'][:60]}")
    
    print("\n=== Search: AI ===")
    results = search_reddit("AI", depth="quick", subreddits=["MachineLearning"])
    for item in results[:5]:
        print(f"  [{item['relevance']:.2f}] {item['title'][:60]}")
