"""Relevance scoring utilities for search results.

Token overlap relevance measures how well a result matches a query
without requiring embeddings or ML models.
"""

import re
from typing import Set


def tokenize(text: str) -> Set[str]:
    """Split text into lowercase tokens, removing punctuation.
    
    Args:
        text: Input text to tokenize
        
    Returns:
        Set of lowercase tokens
    """
    if not text:
        return set()
    # Remove punctuation, split on whitespace
    cleaned = re.sub(r'[^\w\s]', ' ', text.lower())
    tokens = set(cleaned.split())
    # Remove very short tokens (likely noise)
    return {t for t in tokens if len(t) > 2}


def token_overlap_relevance(query: str, text: str) -> float:
    """Calculate relevance score based on token overlap.
    
    Higher score = more query tokens appear in the text.
    
    Args:
        query: Search query
        text: Text to score against query
        
    Returns:
        Float between 0.0 and 1.0
    """
    query_tokens = tokenize(query)
    text_tokens = tokenize(text)
    
    if not query_tokens:
        return 0.0
    
    overlap = query_tokens & text_tokens
    return len(overlap) / len(query_tokens)


def combined_relevance(
    query: str,
    title: str,
    description: str = "",
    title_weight: float = 0.7,
) -> float:
    """Calculate weighted relevance from title and description.
    
    Title matches are weighted higher than description matches.
    
    Args:
        query: Search query
        title: Item title
        description: Item description (optional)
        title_weight: Weight for title score (0-1)
        
    Returns:
        Float between 0.0 and 1.0
    """
    title_score = token_overlap_relevance(query, title)
    desc_score = token_overlap_relevance(query, description) if description else 0.0
    
    desc_weight = 1.0 - title_weight
    return (title_score * title_weight) + (desc_score * desc_weight)
