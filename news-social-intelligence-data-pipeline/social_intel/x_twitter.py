"""X/Twitter scraper using xurl CLI (requires OAuth2 setup).

Uses the official xurl CLI for X API v2 access.
Requires: xurl auth setup (user must do this manually)

Output format matches our normalized article schema.

IMPORTANT: User must complete OAuth2 setup manually:
1. Create app at https://developer.x.com/en/portal/dashboard
2. xurl auth apps add my-app --client-id YOUR_ID --client-secret YOUR_SECRET
3. xurl auth oauth2 --app my-app
4. xurl auth default my-app
"""

import json
import subprocess
import sys
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

from .lib import combined_relevance

XURL_TIMEOUT = 30

# Depth-aware limits
DEPTH_LIMITS = {
    "quick": 10,
    "default": 25,
    "deep": 50,
}


def _log(msg: str) -> None:
    sys.stderr.write(f"[X/Twitter] {msg}\n")
    sys.stderr.flush()


def _run_xurl(args: List[str], timeout: int = XURL_TIMEOUT) -> Optional[Dict]:
    """Run xurl command and return parsed JSON. Never raises."""
    cmd = ["xurl"] + args
    
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        if result.returncode == 0 and result.stdout:
            return json.loads(result.stdout)
        else:
            if "No apps registered" in result.stdout or "No apps registered" in result.stderr:
                _log("Auth not configured - user must run xurl auth setup")
            else:
                _log(f"xurl error: {result.stderr[:200] if result.stderr else result.stdout[:200]}")
            return None
    except subprocess.TimeoutExpired:
        _log(f"xurl timeout after {timeout}s")
        return None
    except FileNotFoundError:
        _log("xurl not found - install with: curl -fsSL https://raw.githubusercontent.com/xdevplatform/xurl/main/install.sh | bash")
        return None
    except json.JSONDecodeError as e:
        _log(f"JSON parse error: {e}")
        return None
    except Exception as e:
        _log(f"xurl error: {e}")
        return None


def _check_auth() -> bool:
    """Check if xurl auth is configured. Never raises."""
    result = _run_xurl(["auth", "status"])
    if result is None:
        return False
    # If we got valid JSON back, auth is likely configured
    return True


def _parse_tweet(tweet: Dict[str, Any], query: str = "") -> Optional[Dict[str, Any]]:
    """Parse X API tweet into normalized article dict. Never raises."""
    try:
        tweet_id = tweet.get("id", "")
        text = tweet.get("text", "")
        
        if not tweet_id or not text:
            return None
        
        # Calculate relevance
        relevance = combined_relevance(query, text, "") if query else 0.0
        
        # Parse created_at (ISO format from X API v2)
        created_at = tweet.get("created_at")
        date_str = None
        created_utc = None
        if created_at:
            try:
                dt = datetime.fromisoformat(created_at.replace("Z", "+00:00"))
                date_str = dt.date().isoformat()
                created_utc = dt.timestamp()
            except Exception:
                pass
        
        # Extract author info
        author = tweet.get("author_id", "")
        
        # Build URL
        url = f"https://x.com/i/status/{tweet_id}"
        
        # Get metrics
        public_metrics = tweet.get("public_metrics", {})
        
        return {
            "id": tweet_id,
            "title": text[:100] + ("..." if len(text) > 100 else ""),
            "url": url,
            "description": text,
            "source": "X/Twitter",
            "author": author,
            "score": public_metrics.get("like_count", 0),
            "num_comments": public_metrics.get("reply_count", 0),
            "created_utc": created_utc,
            "date": date_str,
            "relevance": round(relevance, 3),
            "why_relevant": "xurl search",
            "collected_at": datetime.now(timezone.utc).isoformat(),
            "content_type": "x_twitter",
            "metadata": {
                "tweet_id": tweet_id,
                "retweet_count": public_metrics.get("retweet_count", 0),
                "like_count": public_metrics.get("like_count", 0),
                "reply_count": public_metrics.get("reply_count", 0),
                "quote_count": public_metrics.get("quote_count", 0),
                "impression_count": public_metrics.get("impression_count", 0),
            },
        }
    except Exception as e:
        _log(f"Parse error: {e}")
        return None


def is_available() -> bool:
    """Check if X/Twitter scraping is available (xurl installed + auth configured)."""
    try:
        result = subprocess.run(
            ["xurl", "auth", "status"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        return "No apps registered" not in result.stdout and result.returncode == 0
    except Exception:
        return False


def search_tweets(
    query: str,
    depth: str = "default",
) -> List[Dict[str, Any]]:
    """Search X/Twitter posts. Never raises.
    
    Args:
        query: Search query string (supports X search operators)
        depth: 'quick', 'default', or 'deep' - controls result limit
        
    Returns:
        List of normalized article dicts, empty on failure or if auth not configured
    """
    if not is_available():
        _log("X/Twitter not available - xurl auth not configured")
        return []
    
    limit = DEPTH_LIMITS.get(depth, DEPTH_LIMITS["default"])
    
    _log(f"Searching: {query} (depth={depth}, limit={limit})")
    
    data = _run_xurl(["search", query, "-n", str(limit)])
    
    if not data:
        return []
    
    # X API v2 returns data in {"data": [...]} format
    tweets = data.get("data", [])
    if isinstance(data, list):
        tweets = data
    
    results = []
    for tweet in tweets:
        parsed = _parse_tweet(tweet, query)
        if parsed:
            results.append(parsed)
    
    _log(f"Found {len(results)} results")
    return results


def get_user_timeline(
    username: str,
    limit: int = 20,
) -> List[Dict[str, Any]]:
    """Get recent tweets from a user. Never raises.
    
    Args:
        username: X/Twitter username (with or without @)
        limit: Max tweets to return
        
    Returns:
        List of normalized article dicts, empty on failure
    """
    if not is_available():
        _log("X/Twitter not available - xurl auth not configured")
        return []
    
    username = username.lstrip("@")
    _log(f"Getting timeline for @{username}")
    
    # First get user info
    user_data = _run_xurl(["user", username])
    if not user_data:
        return []
    
    # Get user's tweets via raw API
    user_id = user_data.get("data", {}).get("id")
    if not user_id:
        return []
    
    data = _run_xurl([f"/2/users/{user_id}/tweets", "-n", str(limit)])
    
    if not data:
        return []
    
    tweets = data.get("data", [])
    results = []
    
    for tweet in tweets:
        parsed = _parse_tweet(tweet)
        if parsed:
            parsed["author"] = username
            results.append(parsed)
    
    _log(f"Found {len(results)} tweets")
    return results


def get_mentions(limit: int = 20) -> List[Dict[str, Any]]:
    """Get mentions of the authenticated user. Never raises.
    
    Args:
        limit: Max mentions to return
        
    Returns:
        List of normalized article dicts, empty on failure
    """
    if not is_available():
        _log("X/Twitter not available - xurl auth not configured")
        return []
    
    _log(f"Getting mentions (limit={limit})")
    
    data = _run_xurl(["mentions", "-n", str(limit)])
    
    if not data:
        return []
    
    tweets = data.get("data", [])
    results = []
    
    for tweet in tweets:
        parsed = _parse_tweet(tweet)
        if parsed:
            results.append(parsed)
    
    _log(f"Found {len(results)} mentions")
    return results


# Search operators cheatsheet
SEARCH_OPERATORS = """
X/Twitter Search Operators:
- from:username - tweets from user
- to:username - replies to user
- @username - mentions of user
- #hashtag - hashtag search
- "exact phrase" - exact match
- lang:en - language filter
- -filter:retweets - exclude retweets
- filter:media - only with media
- min_retweets:N - minimum retweets
- min_faves:N - minimum likes
- since:YYYY-MM-DD - after date
- until:YYYY-MM-DD - before date
"""


if __name__ == "__main__":
    print("=== X/Twitter Status ===")
    if is_available():
        print("✅ xurl auth configured")
        
        print("\n=== Search: AI ===")
        results = search_tweets("AI", depth="quick")
        for item in results[:3]:
            print(f"  [{item['score']}] {item['title'][:60]}")
    else:
        print("❌ xurl auth not configured")
        print("\nTo set up:")
        print("1. Create app at https://developer.x.com/en/portal/dashboard")
        print("2. xurl auth apps add my-app --client-id YOUR_ID --client-secret YOUR_SECRET")
        print("3. xurl auth oauth2 --app my-app")
        print("4. xurl auth default my-app")
