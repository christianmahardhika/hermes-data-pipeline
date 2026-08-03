#!/usr/bin/env python3
"""
Knowledge Base Query Module
Part of hermes-data-pipeline project
"""

from qdrant_client import QdrantClient
import requests
from typing import List, Dict, Optional
import os


class KnowledgeQuery:
    """Query knowledge bases with profile-aware routing."""
    
    def __init__(self, qdrant_url: str = "http://localhost:6333", tei_url: str = "http://localhost:8082"):
        self.client = QdrantClient(url=qdrant_url)
        self.tei_url = tei_url
        
        # Profile to collection mapping (768-dim TEI collections only)
        self.profile_collections = {
            "default": ["pagupon-kb"],
            "pagupon": ["pagupon-kb"], 
            "pagupon-finance": ["pagupon-kb"],
            "pagupon-kampus": ["pagupon-kb"],
            "pondo-ngopi": ["pondo-business-kb"]
        }
    
    def _get_embedding(self, text: str) -> List[float]:
        """Get embedding from TEI service."""
        try:
            response = requests.post(
                f"{self.tei_url}/embed",
                json={"inputs": text},
                headers={"Content-Type": "application/json"}
            )
            response.raise_for_status()
            return response.json()[0]  # TEI returns list of embeddings
        except Exception as e:
            print(f"TEI embedding error: {e}")
            # Fallback to zero vector with correct dimension (768 for multilingual-e5-base)
            return [0.0] * 768
    
    def query(
        self, 
        query_text: str, 
        profile: str = "default",
        limit: int = 5,
        collections: Optional[List[str]] = None
    ) -> List[Dict]:
        """
        Query knowledge base with profile-aware routing.
        
        Args:
            query_text: The search query
            profile: Hermes profile name (default, pagupon, pondo-ngopi)
            limit: Max results per collection
            collections: Override profile routing with specific collections
        
        Returns:
            List of results with text, score, and metadata
        """
        # Determine which collections to query
        if collections:
            target_collections = collections
        else:
            target_collections = self.profile_collections.get(profile, ["investment-books-kb"])
        
        # Generate query embedding using TEI service
        query_vector = self._get_embedding(query_text)
        
        # Query each collection
        all_results = []
        
        for coll_name in target_collections:
            try:
                # Use query_points (Qdrant API v1.18+)
                results = self.client.query_points(
                    collection_name=coll_name,
                    query=query_vector,
                    limit=limit,
                    with_payload=True
                )
                
                for result in results.points:
                    all_results.append({
                        "collection": coll_name,
                        "score": result.score,
                        "text": result.payload.get("text", ""),
                        "ticker": result.payload.get("ticker", ""),
                        "type": result.payload.get("type", "book"),
                        "title": result.payload.get("title", ""),
                        "author": result.payload.get("author", ""),
                        "chapter": result.payload.get("chapter", "")
                    })
            except Exception as e:
                print(f"Warning: Could not query {coll_name}: {e}")
        
        # Sort by score and return top results
        all_results.sort(key=lambda x: x["score"], reverse=True)
        
        return all_results[:limit * len(target_collections)]
    
    def query_investment_books(self, query_text: str, limit: int = 5) -> List[Dict]:
        """Query investment books only (Graham, Lynch, Munger)."""
        return self.query(query_text, collections=["investment-books-kb"], limit=limit)
    
    def query_business(self, query_text: str, limit: int = 5) -> List[Dict]:
        """Query Pondo business knowledge."""
        return self.query(query_text, collections=["pondo-business-kb"], limit=limit)
    
    def query_general(self, query_text: str, profile: str = "default", limit: int = 5) -> List[Dict]:
        """Query with profile routing."""
        return self.query(query_text, profile=profile, limit=limit)


if __name__ == "__main__":
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: python knowledge_query.py <query> [profile] [limit]")
        sys.exit(1)
    
    query = sys.argv[1]
    profile = sys.argv[2] if len(sys.argv) > 2 else "default"  
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else 5
    
    kb = KnowledgeQuery()
    results = kb.query(query, profile=profile, limit=limit)
    
    print(f"Query: {query}")
    print(f"Profile: {profile}")
    print("-" * 80)
    
    if not results:
        print("No results found.")
    else:
        for i, result in enumerate(results, 1):
            print(f"\n[{i}] Score: {result['score']:.4f}")
            print(f"    Collection: {result['collection']}")
            print(f"    Type: {result['type']}")
            if result['title']:
                print(f"    Title: {result['title']}")
            if result['author']:
                print(f"    Author: {result['author']}")
            print(f"    Preview: {result['text'][:200]}...")