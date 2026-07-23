#!/usr/bin/env python3
"""
ArangoDB News Correlation System
Production-ready interface untuk news analysis dengan Prof Jiang framework
"""

import requests
import json
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, asdict
import feedparser
import re
from urllib.parse import urljoin

@dataclass
class NewsArticle:
    id: str
    title: str
    source: str
    date: datetime
    url: str
    content_summary: str
    actors: List[str]
    events: List[str] 
    topics: List[str]
    
@dataclass
class Actor:
    name: str
    actor_type: str
    influence_score: float
    description: str
    
@dataclass
class Event:
    name: str
    date: datetime
    event_type: str
    significance_score: float
    description: str

class ArangoNewsDB:
    def __init__(self, host="localhost", port=8529, database="news_analysis"):
        self.base_url = f"http://{host}:{port}"
        self.database = database
        self.db_url = f"{self.base_url}/_db/{database}"
        
    def _request(self, method: str, endpoint: str, data: Optional[Dict] = None) -> Dict:
        """Make HTTP request to ArangoDB"""
        # Fix URL construction - don't use urljoin with leading slash
        url = f"{self.db_url}{endpoint}"
        headers = {"Content-Type": "application/json"}
        
        response = requests.request(method, url, 
                                  json=data if data else None, 
                                  headers=headers)
        return response.json()
    
    def insert_article(self, article: NewsArticle) -> Dict:
        """Insert article into ArangoDB"""
        article_data = asdict(article)
        article_data['date'] = article.date.isoformat()
        article_data['_key'] = article.id
        
        return self._request("POST", "/_api/document/articles", article_data)
    
    def insert_actor(self, actor: Actor) -> Dict:
        """Insert actor into ArangoDB"""
        actor_data = asdict(actor)
        actor_data['_key'] = re.sub(r'[^a-zA-Z0-9_-]', '_', actor.name)
        
        return self._request("POST", "/_api/document/actors", actor_data)
    
    def insert_event(self, event: Event) -> Dict:
        """Insert event into ArangoDB"""
        event_data = asdict(event)
        event_data['date'] = event.date.isoformat()
        event_data['_key'] = re.sub(r'[^a-zA-Z0-9_-]', '_', event.name)
        
        return self._request("POST", "/_api/document/events", event_data)
    
    def create_mention_edge(self, article_id: str, actor_name: str, relationship: str = "mentions") -> Dict:
        """Create edge between article and actor"""
        actor_key = re.sub(r'[^a-zA-Z0-9_-]', '_', actor_name)
        edge_data = {
            "_from": f"articles/{article_id}",
            "_to": f"actors/{actor_key}",
            "relationship": relationship,
            "created": datetime.now().isoformat()
        }
        
        return self._request("POST", "/_api/document/mentions", edge_data)
    
    def create_cover_edge(self, article_id: str, event_name: str, relationship: str = "covers") -> Dict:
        """Create edge between article and event"""
        event_key = re.sub(r'[^a-zA-Z0-9_-]', '_', event_name)
        edge_data = {
            "_from": f"articles/{article_id}",
            "_to": f"events/{event_key}",
            "relationship": relationship,
            "created": datetime.now().isoformat()
        }
        
        return self._request("POST", "/_api/document/covers", edge_data)
    
    def create_actor_relation(self, actor1: str, actor2: str, relationship: str, strength: float = 1.0) -> Dict:
        """Create relationship between actors"""
        actor1_key = re.sub(r'[^a-zA-Z0-9_-]', '_', actor1)
        actor2_key = re.sub(r'[^a-zA-Z0-9_-]', '_', actor2)
        
        edge_data = {
            "_from": f"actors/{actor1_key}",
            "_to": f"actors/{actor2_key}",
            "relationship": relationship,
            "strength": strength,
            "created": datetime.now().isoformat()
        }
        
        return self._request("POST", "/_api/document/relates_to", edge_data)
    
    def query_aql(self, query: str, bind_vars: Optional[Dict] = None) -> Dict:
        """Execute AQL query"""
        data = {"query": query}
        if bind_vars:
            data["bindVars"] = bind_vars
            
        return self._request("POST", "/_api/cursor", data)
    
    def get_actor_network(self, actor_name: str, depth: int = 2) -> Dict:
        """Get network around specific actor using AQL"""
        actor_key = re.sub(r'[^a-zA-Z0-9_-]', '_', actor_name)
        
        query = f"""
        FOR v, e, p IN 1..{depth} ANY 'actors/{actor_key}' 
        GRAPH 'news_graph'
        RETURN {{
            vertex: v,
            edge: e,
            path: p
        }}
        """
        
        return self.query_aql(query)
    
    def get_news_timeline(self, days: int = 7) -> Dict:
        """Get articles from last N days with connections"""
        cutoff_date = (datetime.now() - timedelta(days=days)).isoformat()
        
        query = """
        FOR article IN articles
            FILTER article.date >= @cutoff_date
            LET actors = (
                FOR v IN 1..1 OUTBOUND article mentions
                RETURN v.name
            )
            LET events = (
                FOR v IN 1..1 OUTBOUND article covers  
                RETURN v.name
            )
            SORT article.date DESC
            RETURN {
                article: article,
                actors: actors,
                events: events
            }
        """
        
        return self.query_aql(query, {"cutoff_date": cutoff_date})
    
    def find_correlation_patterns(self) -> Dict:
        """Find patterns using Prof Jiang framework"""
        
        # Actor co-occurrence analysis
        cooccurrence_query = """
        FOR article IN articles
            LET article_actors = (
                FOR v IN 1..1 OUTBOUND article mentions
                RETURN v.name
            )
            FOR i IN 0..LENGTH(article_actors)-2
                FOR j IN (i+1)..LENGTH(article_actors)-1
                    COLLECT actor1 = article_actors[i], actor2 = article_actors[j] 
                    WITH COUNT INTO occurrences
                    SORT occurrences DESC
                    LIMIT 10
                    RETURN {actors: [actor1, actor2], count: occurrences}
        """
        
        # Event chains (temporal sequences)
        event_chain_query = """
        FOR event1 IN events
            FOR event2 IN events
                FILTER event1.date < event2.date
                LET days_diff = DATE_DIFF(event1.date, event2.date, 'd')
                FILTER days_diff <= 30 AND days_diff > 0
                SORT event1.date
                RETURN {
                    from: event1.name,
                    to: event2.name,
                    days_apart: days_diff
                }
        """
        
        patterns = {}
        patterns['actor_cooccurrence'] = self.query_aql(cooccurrence_query)
        patterns['event_chains'] = self.query_aql(event_chain_query)
        
        return patterns

class NewsIngester:
    def __init__(self, arango_db: ArangoNewsDB):
        self.db = arango_db
        self.rss_feeds = [
            "https://feeds.reuters.com/reuters/world",
            "https://rss.cnn.com/rss/edition.rss",
            "https://feeds.bbci.co.uk/news/world/rss.xml",
            "https://techcrunch.com/feed/"
        ]
    
    def extract_entities(self, text: str) -> tuple[List[str], List[str], List[str]]:
        """Simple entity extraction (can be enhanced with NLP)"""
        
        # Common actors in news
        actors = []
        actor_patterns = [
            r'\b(OpenAI|Anthropic|Google|Microsoft|Meta|Apple|Tesla|SpaceX)\b',
            r'\b(Putin|Biden|Trump|Xi Jinping|Zelensky)\b',
            r'\b(US Government|EU|NATO|UN|Fed|ECB)\b'
        ]
        
        for pattern in actor_patterns:
            matches = re.findall(pattern, text, re.IGNORECASE)
            actors.extend(matches)
        
        # Events (simplified detection)
        events = []
        if 'IPO' in text or 'initial public offering' in text.lower():
            events.append('IPO Event')
        if 'acquisition' in text.lower() or 'merger' in text.lower():
            events.append('M&A Event')
        if 'lawsuit' in text.lower() or 'legal action' in text.lower():
            events.append('Legal Event')
        
        # Topics
        topics = []
        if any(word in text.lower() for word in ['ai', 'artificial intelligence', 'machine learning']):
            topics.append('AI')
        if any(word in text.lower() for word in ['ukraine', 'russia', 'war', 'conflict']):
            topics.append('Geopolitics')
        if any(word in text.lower() for word in ['election', 'vote', 'campaign']):
            topics.append('Politics')
        
        return actors, events, topics
    
    def ingest_rss_feed(self, feed_url: str) -> List[NewsArticle]:
        """Ingest articles from RSS feed"""
        articles = []
        
        try:
            print(f"  Parsing feed: {feed_url}")
            feed = feedparser.parse(feed_url)
            
            if not hasattr(feed, 'entries') or not feed.entries:
                print(f"  ⚠️  No entries found in feed")
                return articles
            
            print(f"  Found {len(feed.entries)} entries")
            
            for i, entry in enumerate(feed.entries):
                # Extract entities
                content = entry.get('summary', '') + ' ' + entry.get('title', '')
                actors, events, topics = self.extract_entities(content)
                
                # Create safe key (no spaces, slashes, special chars)
                feed_name = re.sub(r'[^a-zA-Z0-9_-]', '_', feed.feed.get('title', 'unknown'))
                safe_key = f"{feed_name}_{i}_{int(datetime.now().timestamp())}"
                
                # Create article
                article = NewsArticle(
                    id=safe_key,
                    title=entry.title,
                    source=feed.feed.get('title', feed_url),
                    date=datetime(*entry.published_parsed[:6]) if hasattr(entry, 'published_parsed') else datetime.now(),
                    url=entry.link,
                    content_summary=entry.get('summary', '')[:500],
                    actors=list(set(actors)),
                    events=list(set(events)),
                    topics=list(set(topics))
                )
                
                articles.append(article)
                
        except Exception as e:
            print(f"  ❌ Error ingesting {feed_url}: {e}")
            
        return articles
    
    def ingest_all_feeds(self) -> Dict[str, int]:
        """Ingest from all RSS feeds"""
        results = {"articles": 0, "actors": 0, "events": 0}
        
        for feed_url in self.rss_feeds:
            print(f"Ingesting from: {feed_url}")
            articles = self.ingest_rss_feed(feed_url)
            
            for article in articles:
                # Insert article
                try:
                    self.db.insert_article(article)
                    results["articles"] += 1
                    
                    # Insert actors and create edges
                    for actor_name in article.actors:
                        actor = Actor(actor_name, "unknown", 5.0, f"Actor from {article.source}")
                        try:
                            self.db.insert_actor(actor)
                            results["actors"] += 1
                        except:
                            pass  # Actor might already exist
                        
                        self.db.create_mention_edge(article.id, actor_name)
                    
                    # Insert events and create edges
                    for event_name in article.events:
                        event = Event(event_name, article.date, "news_event", 5.0, f"Event from {article.source}")
                        try:
                            self.db.insert_event(event)
                            results["events"] += 1
                        except:
                            pass  # Event might already exist
                        
                        self.db.create_cover_edge(article.id, event_name)
                        
                except Exception as e:
                    print(f"Error inserting article {article.title}: {e}")
        
        return results

def main():
    """Demo ArangoDB news analysis with real data"""
    print("🚀 Starting ArangoDB News Analysis...")
    
    # Connect to ArangoDB
    db = ArangoNewsDB()
    
    # Test connection
    try:
        result = db.query_aql("RETURN 'Connected to ArangoDB!'")
        print("✓ ArangoDB connection successful")
    except Exception as e:
        print(f"❌ ArangoDB connection failed: {e}")
        return
    
    # Ingest real news data
    print("\n📡 Ingesting real news data...")
    ingester = NewsIngester(db)
    results = ingester.ingest_all_feeds()
    
    print(f"✓ Ingested: {results['articles']} articles, {results['actors']} actors, {results['events']} events")
    
    # Demo queries
    print("\n📊 CORRELATION ANALYSIS:")
    try:
        patterns = db.find_correlation_patterns()
        
        cooccurrence_result = patterns.get('actor_cooccurrence', {})
        if 'result' in cooccurrence_result and cooccurrence_result['result']:
            print("Top actor co-occurrences:")
            for item in cooccurrence_result['result'][:5]:
                print(f"  • {item['actors']} ({item['count']} times)")
        else:
            print("No significant actor co-occurrences found yet")
            
        chains_result = patterns.get('event_chains', {})
        if 'result' in chains_result and chains_result['result']:
            print("Event chains found:")
            for item in chains_result['result'][:3]:
                print(f"  • {item['from']} → {item['to']} ({item['days_apart']} days)")
        else:
            print("No event chains detected yet")
            
    except Exception as e:
        print(f"Correlation analysis error: {e}")
    
    print("\n📰 WEEKLY TIMELINE:")
    try:
        timeline = db.get_news_timeline(7)
        if 'result' in timeline and timeline['result']:
            print("Recent articles with connections:")
            for item in timeline['result'][:5]:
                print(f"• {item['article']['title'][:80]}...")
                if item['actors']:
                    print(f"  Actors: {item['actors']}")
                if item['events']:
                    print(f"  Events: {item['events']}")
                print()
        else:
            print("No recent articles found or timeline query failed")
    except Exception as e:
        print(f"Timeline query error: {e}")
    
    # Let's also check what data we have
    print("\n🔍 DATABASE STATS:")
    try:
        articles_count = db.query_aql("RETURN LENGTH(articles)")
        actors_count = db.query_aql("RETURN LENGTH(actors)")  
        events_count = db.query_aql("RETURN LENGTH(events)")
        
        if 'result' in articles_count:
            print(f"Articles in database: {articles_count['result'][0] if articles_count['result'] else 0}")
        if 'result' in actors_count:
            print(f"Actors in database: {actors_count['result'][0] if actors_count['result'] else 0}")
        if 'result' in events_count:
            print(f"Events in database: {events_count['result'][0] if events_count['result'] else 0}")
            
    except Exception as e:
        print(f"Stats query error: {e}")

if __name__ == "__main__":
    main()