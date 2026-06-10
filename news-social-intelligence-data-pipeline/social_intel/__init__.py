"""Social Intelligence — Multi-source data gathering for social/news intelligence.

Modules:
- hackernews: HackerNews via Algolia API (free, no auth)
- reddit: Reddit via RSS feeds (free, no auth)
- youtube: YouTube via yt-dlp (free, no API key)
- x_twitter: X/Twitter via xurl CLI (requires OAuth2 setup)
- near_duplicate: Cross-source duplicate detection

All modules follow "never raises" pattern and use normalized output format.
"""

from .hackernews import search_stories, get_front_page, get_top_stories
from .reddit import search_reddit, get_subreddit_hot
from .youtube import search_youtube, get_video_info, get_channel_videos
from .x_twitter import search_tweets, get_user_timeline, is_available as x_is_available
from .near_duplicate import NearDuplicateDetector, deduplicate_batch

__all__ = [
    # HackerNews
    'search_stories',
    'get_front_page', 
    'get_top_stories',
    # Reddit
    'search_reddit',
    'get_subreddit_hot',
    # YouTube
    'search_youtube',
    'get_video_info',
    'get_channel_videos',
    # X/Twitter
    'search_tweets',
    'get_user_timeline',
    'x_is_available',
    # Near-duplicate
    'NearDuplicateDetector',
    'deduplicate_batch',
]
