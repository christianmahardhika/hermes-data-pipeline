"""Shared HTTP utilities with retry logic and defensive patterns.

All functions follow "never raises" pattern - return empty/None on error.
"""

import sys
import time
import random
from typing import Optional, Dict, Any
from urllib.request import Request, urlopen
from urllib.error import URLError, HTTPError
import json

DEFAULT_TIMEOUT = 15
MAX_RETRIES = 3
RETRY_DELAY = 1.0

USER_AGENTS = [
    "Mozilla/5.0 (compatible; SocialIntel/1.0)",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
]


def _log(msg: str) -> None:
    """Log to stderr for visibility without polluting stdout."""
    sys.stderr.write(f"[HTTP] {msg}\n")
    sys.stderr.flush()


def get_text(
    url: str,
    timeout: int = DEFAULT_TIMEOUT,
    headers: Optional[Dict[str, str]] = None,
    accept: str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    retries: int = MAX_RETRIES,
) -> Optional[str]:
    """Fetch URL and return text content. Never raises.
    
    Args:
        url: URL to fetch
        timeout: Request timeout in seconds
        headers: Additional headers to send
        accept: Accept header value
        retries: Number of retry attempts
        
    Returns:
        Response text or None on any failure
    """
    default_headers = {
        "User-Agent": random.choice(USER_AGENTS),
        "Accept": accept,
        "Accept-Language": "en-US,en;q=0.9,id;q=0.8",
    }
    if headers:
        default_headers.update(headers)
    
    for attempt in range(retries):
        try:
            req = Request(url, headers=default_headers)
            with urlopen(req, timeout=timeout) as resp:
                # Handle different encodings
                charset = resp.headers.get_content_charset() or 'utf-8'
                return resp.read().decode(charset, errors='replace')
        except HTTPError as e:
            _log(f"HTTP {e.code} for {url}")
            if e.code in (429, 503):  # Rate limit or service unavailable
                time.sleep(RETRY_DELAY * (attempt + 1))
                continue
            return None
        except URLError as e:
            _log(f"URL error for {url}: {e.reason}")
            time.sleep(RETRY_DELAY * (attempt + 1))
            continue
        except Exception as e:
            _log(f"Unexpected error for {url}: {e}")
            return None
    
    return None


def get_json(
    url: str,
    timeout: int = DEFAULT_TIMEOUT,
    headers: Optional[Dict[str, str]] = None,
    retries: int = MAX_RETRIES,
) -> Optional[Dict[str, Any]]:
    """Fetch URL and parse JSON response. Never raises.
    
    Args:
        url: URL to fetch
        timeout: Request timeout in seconds
        headers: Additional headers to send
        retries: Number of retry attempts
        
    Returns:
        Parsed JSON dict or None on any failure
    """
    text = get_text(
        url,
        timeout=timeout,
        headers=headers,
        accept="application/json",
        retries=retries,
    )
    if not text:
        return None
    
    try:
        return json.loads(text)
    except json.JSONDecodeError as e:
        _log(f"JSON parse error: {e}")
        return None
