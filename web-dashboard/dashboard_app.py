#!/usr/bin/env python3
"""
News Intelligence Web Dashboard
Visualisasi Prof Jiang Framework Analysis
"""

from flask import Flask, render_template, jsonify, request
import requests
import json
from datetime import datetime, timedelta
import networkx as nx
import plotly.graph_objects as go
import plotly.express as px
from plotly.utils import PlotlyJSONEncoder

app = Flask(__name__)

class IntelligenceDashboard:
    def __init__(self):
        self.db_url = "http://localhost:8529/_db/news_intelligence"
    
    def query_aql(self, query, bind_vars=None):
        """Execute AQL query"""
        data = {"query": query}
        if bind_vars:
            data["bindVars"] = bind_vars
        
        try:
            response = requests.post(f"{self.db_url}/_api/cursor", 
                                   json=data,
                                   headers={"Content-Type": "application/json"})
            return response.json()
        except Exception as e:
            return {"error": str(e)}
    
    def get_dashboard_metrics(self):
        """Get key metrics for dashboard"""
        query = """
        RETURN {
            total_articles: LENGTH(articles),
            total_actors: LENGTH(actors),
            total_insights: LENGTH(jiang_insights),
            geostrategy_insights: LENGTH(FOR insight IN jiang_insights FILTER insight.framework_type == 'geostrategy' RETURN 1),
            game_theory_insights: LENGTH(FOR insight IN jiang_insights FILTER insight.framework_type == 'game_theory' RETURN 1),
            secret_history_insights: LENGTH(FOR insight IN jiang_insights FILTER insight.framework_type == 'secret_history' RETURN 1)
        }
        """
        
        result = self.query_aql(query)
        return result.get('result', [{}])[0] if result.get('result') else {}
    
    def get_top_actors(self, limit=10):
        """Get top actors by influence"""
        query = """
        FOR actor IN actors
            SORT actor.influence_score DESC
            LIMIT @limit
            RETURN {
                name: actor.name,
                influence: actor.influence_score,
                type: actor.actor_type,
                description: actor.description
            }
        """
        
        result = self.query_aql(query, {"limit": limit})
        return result.get('result', [])
    
    def get_recent_insights(self, limit=10):
        """Get recent Prof Jiang insights"""
        query = """
        FOR insight IN jiang_insights
            SORT insight.created DESC
            LIMIT @limit
            RETURN {
                framework: insight.framework_type,
                analysis: insight.analysis,
                prediction: insight.prediction,
                confidence: insight.confidence_score,
                created: insight.created,
                related_actors: insight.related_actors
            }
        """
        
        result = self.query_aql(query, {"limit": limit})
        return result.get('result', [])
    
    def get_recent_articles(self, limit=20):
        """Get recent articles with metadata"""
        query = """
        FOR article IN articles
            SORT article.date DESC
            LIMIT @limit
            RETURN {
                title: article.title,
                source: article.source,
                date: article.date,
                url: article.url,
                actors: article.actors,
                events: article.events,
                sentiment: article.sentiment_score,
                keywords: article.keywords
            }
        """
        
        result = self.query_aql(query, {"limit": limit})
        return result.get('result', [])
    
    def get_actor_network(self):
        """Get actor correlation network for visualization"""
        query = """
        FOR edge IN correlates
            FILTER edge.correlation_type == 'co_occurrence'
            RETURN {
                from: SPLIT(edge._from, '/')[1],
                to: SPLIT(edge._to, '/')[1],
                strength: edge.strength
            }
        """
        
        result = self.query_aql(query)
        edges = result.get('result', [])
        
        # Get actor details
        actors_query = """
        FOR actor IN actors
            RETURN {
                id: actor._key,
                name: actor.name,
                influence: actor.influence_score,
                type: actor.actor_type
            }
        """
        
        actors_result = self.query_aql(actors_query)
        actors = actors_result.get('result', [])
        
        return {"nodes": actors, "edges": edges}
    
    def get_sentiment_timeline(self, days=7):
        """Get sentiment trends over time"""
        query = """
        FOR article IN articles
            FILTER article.date >= DATE_SUB(DATE_NOW(), @days, 'day')
            COLLECT date = DATE_FORMAT(article.date, '%Y-%m-%d') INTO articles_by_date
            LET avg_sentiment = AVG(articles_by_date[*].sentiment_score)
            LET article_count = LENGTH(articles_by_date)
            SORT date ASC
            RETURN {
                date: date,
                sentiment: avg_sentiment,
                count: article_count
            }
        """
        
        result = self.query_aql(query, {"days": days})
        return result.get('result', [])

dashboard = IntelligenceDashboard()

@app.route('/')
def index():
    """Main dashboard page"""
    return render_template('dashboard.html')

@app.route('/api/metrics')
def api_metrics():
    """API endpoint for dashboard metrics"""
    metrics = dashboard.get_dashboard_metrics()
    return jsonify(metrics)

@app.route('/api/actors')
def api_actors():
    """API endpoint for top actors"""
    limit = request.args.get('limit', 10, type=int)
    actors = dashboard.get_top_actors(limit)
    return jsonify(actors)

@app.route('/api/insights')
def api_insights():
    """API endpoint for Prof Jiang insights"""
    limit = request.args.get('limit', 10, type=int)
    insights = dashboard.get_recent_insights(limit)
    return jsonify(insights)

@app.route('/api/articles')
def api_articles():
    """API endpoint for recent articles with pagination"""
    limit = request.args.get('limit', 5, type=int)  # Reduce to 5 articles for speed
    articles = dashboard.get_recent_articles(limit)
    return jsonify(articles)

@app.route('/api/network')
def api_network():
    """API endpoint for actor network with size limit"""
    max_nodes = request.args.get('max_nodes', 15, type=int)  # Limit nodes for performance
    network = dashboard.get_actor_network()
    
    # Limit network size for performance
    if 'nodes' in network and len(network['nodes']) > max_nodes:
        # Keep only top actors by influence
        sorted_nodes = sorted(network['nodes'], key=lambda x: x.get('influence', 0), reverse=True)
        network['nodes'] = sorted_nodes[:max_nodes]
        
        # Filter edges to only include remaining nodes
        remaining_ids = {node['id'] for node in network['nodes']}
        network['links'] = [
            link for link in network.get('links', [])
            if link['source'] in remaining_ids and link['target'] in remaining_ids
        ]
    
    return jsonify(network)

@app.route('/api/sentiment')
def api_sentiment():
    """API endpoint for sentiment timeline"""
    days = request.args.get('days', 7, type=int)
    sentiment = dashboard.get_sentiment_timeline(days)
    return jsonify(sentiment)

@app.route('/api/insights/<framework_type>')
def api_insights_by_framework(framework_type):
    """API endpoint for insights by framework type"""
    query = """
    FOR insight IN jiang_insights
        FILTER insight.framework_type == @framework_type
        SORT insight.confidence_score DESC
        RETURN {
            analysis: insight.analysis,
            prediction: insight.prediction,
            confidence: insight.confidence_score,
            created: insight.created,
            related_actors: insight.related_actors
        }
    """
    
    result = dashboard.query_aql(query, {"framework_type": framework_type})
    return jsonify(result.get('result', []))

if __name__ == '__main__':
    print("🌐 Starting News Intelligence Dashboard...")
    print("📊 Dashboard will be available at: http://localhost:5000")
    app.run(debug=True, host='0.0.0.0', port=5000)