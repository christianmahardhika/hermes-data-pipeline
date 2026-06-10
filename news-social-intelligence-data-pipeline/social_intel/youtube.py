"""YouTube metadata scraper using yt-dlp (no API key required).

Uses yt-dlp binary to extract video metadata without downloading.
Requires: pip install yt-dlp (or system package)

Output format matches our normalized article schema.
"""

import json
import subprocess
import sys
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

from .lib import combined_relevance

YTDLP_TIMEOUT = 30

# Depth-aware limits
DEPTH_LIMITS = {
    "quick": 5,
    "default": 10,
    "deep": 25,
}


def _log(msg: str) -> None:
    sys.stderr.write(f"[YouTube] {msg}\n")
    sys.stderr.flush()


def _run_ytdlp(args: List[str], timeout: int = YTDLP_TIMEOUT) -> Optional[str]:
    """Run yt-dlp command and return stdout. Never raises."""
    cmd = ["yt-dlp", "--no-warnings", "-q"] + args
    
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        if result.returncode == 0:
            return result.stdout
        else:
            _log(f"yt-dlp error: {result.stderr[:200]}")
            return None
    except subprocess.TimeoutExpired:
        _log(f"yt-dlp timeout after {timeout}s")
        return None
    except FileNotFoundError:
        _log("yt-dlp not found - install with: pip install yt-dlp")
        return None
    except Exception as e:
        _log(f"yt-dlp error: {e}")
        return None


def _parse_video(data: Dict[str, Any], query: str = "") -> Optional[Dict[str, Any]]:
    """Parse yt-dlp JSON into normalized article dict. Never raises."""
    try:
        video_id = data.get("id", "")
        title = data.get("title", "")
        
        if not video_id or not title:
            return None
        
        description = data.get("description", "") or ""
        description = description[:500]
        
        # Calculate relevance
        relevance = combined_relevance(query, title, description) if query else 0.0
        
        # Parse upload date (YYYYMMDD format)
        upload_date = data.get("upload_date")
        date_str = None
        if upload_date and len(upload_date) == 8:
            try:
                date_str = f"{upload_date[:4]}-{upload_date[4:6]}-{upload_date[6:8]}"
            except Exception:
                pass
        
        return {
            "id": video_id,
            "title": title,
            "url": data.get("webpage_url") or f"https://www.youtube.com/watch?v={video_id}",
            "description": description,
            "source": "YouTube",
            "author": data.get("uploader") or data.get("channel") or "",
            "score": data.get("view_count") or 0,
            "num_comments": data.get("comment_count") or 0,
            "created_utc": data.get("timestamp"),
            "date": date_str,
            "relevance": round(relevance, 3),
            "why_relevant": "yt-dlp search",
            "collected_at": datetime.now(timezone.utc).isoformat(),
            "content_type": "youtube",
            "metadata": {
                "channel_id": data.get("channel_id"),
                "channel": data.get("channel"),
                "duration": data.get("duration"),
                "view_count": data.get("view_count"),
                "like_count": data.get("like_count"),
                "categories": data.get("categories", []),
                "tags": data.get("tags", [])[:10],
            },
        }
    except Exception as e:
        _log(f"Parse error: {e}")
        return None


def search_youtube(
    query: str,
    depth: str = "default",
) -> List[Dict[str, Any]]:
    """Search YouTube videos. Never raises.
    
    Args:
        query: Search query string
        depth: 'quick', 'default', or 'deep' - controls result limit
        
    Returns:
        List of normalized article dicts, empty on failure
    """
    limit = DEPTH_LIMITS.get(depth, DEPTH_LIMITS["default"])
    
    _log(f"Searching: {query} (depth={depth}, limit={limit})")
    
    # yt-dlp search syntax
    search_url = f"ytsearch{limit}:{query}"
    
    output = _run_ytdlp([
        "--dump-json",
        "--flat-playlist",
        search_url,
    ])
    
    if not output:
        return []
    
    results = []
    for line in output.strip().split("\n"):
        if not line:
            continue
        try:
            data = json.loads(line)
            parsed = _parse_video(data, query)
            if parsed:
                results.append(parsed)
        except json.JSONDecodeError:
            continue
    
    _log(f"Found {len(results)} results")
    return results


def get_video_info(url: str) -> Optional[Dict[str, Any]]:
    """Get metadata for a specific video. Never raises.
    
    Args:
        url: YouTube video URL
        
    Returns:
        Normalized article dict or None on failure
    """
    _log(f"Getting info: {url}")
    
    output = _run_ytdlp([
        "--dump-json",
        "--no-playlist",
        url,
    ])
    
    if not output:
        return None
    
    try:
        data = json.loads(output)
        return _parse_video(data)
    except json.JSONDecodeError:
        return None


def get_channel_videos(
    channel_url: str,
    limit: int = 10,
) -> List[Dict[str, Any]]:
    """Get recent videos from a channel. Never raises.
    
    Args:
        channel_url: YouTube channel URL (/@handle or /channel/ID)
        limit: Max videos to return
        
    Returns:
        List of normalized article dicts
    """
    _log(f"Getting channel videos: {channel_url}")
    
    # Add /videos to get uploads
    if not channel_url.endswith("/videos"):
        channel_url = channel_url.rstrip("/") + "/videos"
    
    output = _run_ytdlp([
        "--dump-json",
        "--flat-playlist",
        "--playlist-end", str(limit),
        channel_url,
    ], timeout=60)  # Channels can take longer
    
    if not output:
        return []
    
    results = []
    for line in output.strip().split("\n"):
        if not line:
            continue
        try:
            data = json.loads(line)
            parsed = _parse_video(data)
            if parsed:
                results.append(parsed)
        except json.JSONDecodeError:
            continue
    
    _log(f"Found {len(results)} videos")
    return results


if __name__ == "__main__":
    print("=== Search: Python tutorial ===")
    results = search_youtube("Python tutorial", depth="quick")
    for item in results:
        views = item["metadata"].get("view_count", 0)
        print(f"  [{views:,} views] {item['title'][:50]}")
