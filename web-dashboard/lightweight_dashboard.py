#!/usr/bin/env python3
"""
Ultra-lightweight News Intelligence Dashboard
Using Bottle (103KB) instead of Flask + heavy deps
"""

from bottle import route, run, static_file, response, request
import json
import sys
import os

# Add current directory to path for imports
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from news_intelligence import NewsIntelligenceDB

# Initialize database
db = NewsIntelligenceDB()

@route('/')
def dashboard():
    """Serve main dashboard"""
    return """<!DOCTYPE html>
<html>
<head>
    <title>News Intelligence - Light</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, sans-serif; background: #f5f7fa; }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        .header { text-align: center; margin-bottom: 30px; }
        .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; margin-bottom: 30px; }
        .metric-card { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); text-align: center; }
        .metric-value { font-size: 2em; font-weight: bold; color: #2563eb; }
        .metric-label { color: #6b7280; font-size: 0.9em; margin-top: 5px; }
        .section { background: white; margin-bottom: 20px; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .section h3 { margin-bottom: 15px; color: #374151; border-bottom: 2px solid #e5e7eb; padding-bottom: 10px; }
        .actor-item { display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid #f3f4f6; }
        .insight-card { margin-bottom: 15px; padding: 15px; border-left: 4px solid #3b82f6; background: #f8fafc; }
        .framework-badge { 
            display: inline-block; padding: 2px 8px; border-radius: 12px; font-size: 0.8em; font-weight: bold; 
            color: white; text-transform: uppercase;
        }
        .badge-geostrategy { background: #dc2626; }
        .badge-game_theory { background: #059669; }
        .badge-secret_history { background: #7c3aed; }
        .loading { text-align: center; color: #6b7280; padding: 20px; }
        .sentiment-metrics {
            display: flex;
            gap: 20px;
            margin-bottom: 15px;
        }
        
        .sentiment-stat {
            flex: 1;
            padding: 15px;
            border-radius: 8px;
            text-align: center;
            font-weight: bold;
        }
        
        .sentiment-stat.positive {
            background: linear-gradient(135deg, #10b981, #34d399);
            color: white;
        }
        
        .sentiment-stat.neutral {
            background: linear-gradient(135deg, #6b7280, #9ca3af);
            color: white;
        }
        
        .sentiment-stat.negative {
            background: linear-gradient(135deg, #ef4444, #f87171);
            color: white;
        }
        
        .sentiment-stat .label {
            display: block;
            font-size: 0.9em;
            margin-bottom: 5px;
        }
        
        .sentiment-stat span:last-child {
            font-size: 1.5em;
        }
        .refresh-btn { 
            background: #2563eb; color: white; border: none; padding: 10px 20px; 
            border-radius: 5px; cursor: pointer; float: right; margin-bottom: 15px;
        }
        .refresh-btn:hover { background: #1d4ed8; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📊 News Intelligence Dashboard</h1>
            <p>Prof Jiang Predictive History Framework - Ultra Light Edition</p>
        </div>

        <div class="metrics" id="metrics">
            <div class="loading">Loading metrics...</div>
        </div>

        <div class="section">
            <button class="refresh-btn" onclick="loadData()">🔄 Refresh</button>
            <h3>👥 Top Actors by Influence</h3>
            <div id="actors">
                <div class="loading">Loading actors...</div>
            </div>
        </div>

        <div class="section">
            <h3>🧠 Prof Jiang Insights</h3>
            <div id="insights">
                <div class="loading">Loading insights...</div>
            </div>
        </div>

        <div class="section">
            <h3>📈 Sentiment Analysis</h3>
            <div id="sentiment-container">
                <div class="sentiment-metrics">
                    <div class="sentiment-stat positive">
                        <span class="label">Positive:</span>
                        <span id="positive-count">-</span>
                    </div>
                    <div class="sentiment-stat neutral">
                        <span class="label">Neutral:</span>
                        <span id="neutral-count">-</span>
                    </div>
                    <div class="sentiment-stat negative">
                        <span class="label">Negative:</span>
                        <span id="negative-count">-</span>
                    </div>
                </div>
                <div id="sentiment-chart" style="height: 200px; margin-top: 10px;"></div>
            </div>
        </div>

        <div class="section">
            <h3>🌐 Actor Network Graph</h3>
            <div id="network-container" style="width: 100%; height: 400px; border: 1px solid #e5e7eb; border-radius: 8px; background: #f9fafb;">
                <svg id="network-svg" width="100%" height="400"></svg>
            </div>
        </div>

        <div class="section">
            <h3>📰 Recent Articles</h3>
            <div id="articles">
                <div class="loading">Loading articles...</div>
            </div>
        </div>
    </div>

    <script>
        async function fetchData(endpoint) {
            try {
                const response = await fetch(endpoint);
                return await response.json();
            } catch (error) {
                console.error('Fetch error:', error);
                return null;
            }
        }

        async function loadMetrics() {
            const data = await fetchData('/api/metrics');
            if (!data) return;
            
            document.getElementById('metrics').innerHTML = `
                <div class="metric-card">
                    <div class="metric-value">${data.articles}</div>
                    <div class="metric-label">Articles</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">${data.actors}</div>
                    <div class="metric-label">Actors</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">${data.geostrategy}</div>
                    <div class="metric-label">Geostrategy</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">${data.game_theory}</div>
                    <div class="metric-label">Game Theory</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">${data.secret_history}</div>
                    <div class="metric-label">Secret History</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">${data.total_insights}</div>
                    <div class="metric-label">Total Insights</div>
                </div>
            `;
        }

        async function loadActors() {
            const data = await fetchData('/api/actors?limit=8');
            if (!data) return;
            
            document.getElementById('actors').innerHTML = data.map(actor => `
                <div class="actor-item">
                    <div>
                        <strong>${actor.name}</strong>
                        <div style="color: #6b7280; font-size: 0.9em;">${actor.type}</div>
                    </div>
                    <div style="color: #2563eb; font-weight: bold;">${actor.influence}/10</div>
                </div>
            `).join('');
        }

        async function loadInsights() {
            const data = await fetchData('/api/insights?limit=6');
            if (!data) return;
            
            document.getElementById('insights').innerHTML = data.map(insight => `
                <div class="insight-card">
                    <div style="margin-bottom: 10px;">
                        <span class="framework-badge badge-${insight.framework}">${insight.framework}</span>
                        <span style="float: right; color: #6b7280;">${insight.confidence}/10</span>
                    </div>
                    <div style="margin-bottom: 8px;"><strong>Analysis:</strong> ${insight.analysis}</div>
                    <div style="color: #2563eb;"><strong>Prediction:</strong> ${insight.prediction}</div>
                </div>
            `).join('');
        }

        async function loadSentiment() {
            const data = await fetchData('/api/sentiment');
            if (!data) return;
            
            // Update sentiment counters
            document.getElementById('positive-count').textContent = data.positive || 0;
            document.getElementById('neutral-count').textContent = data.neutral || 0;
            document.getElementById('negative-count').textContent = data.negative || 0;
            
            // Create simple bar chart
            const total = data.positive + data.neutral + data.negative;
            if (total > 0) {
                const chartContainer = document.getElementById('sentiment-chart');
                const positivePercent = (data.positive / total * 100).toFixed(1);
                const neutralPercent = (data.neutral / total * 100).toFixed(1);
                const negativePercent = (data.negative / total * 100).toFixed(1);
                
                chartContainer.innerHTML = `
                    <div style="display: flex; height: 30px; border-radius: 15px; overflow: hidden; background: #f3f4f6;">
                        <div style="width: ${positivePercent}%; background: #10b981; display: flex; align-items: center; justify-content: center; color: white; font-size: 0.8em; font-weight: bold;">
                            ${positivePercent > 10 ? positivePercent + '%' : ''}
                        </div>
                        <div style="width: ${neutralPercent}%; background: #6b7280; display: flex; align-items: center; justify-content: center; color: white; font-size: 0.8em; font-weight: bold;">
                            ${neutralPercent > 10 ? neutralPercent + '%' : ''}
                        </div>
                        <div style="width: ${negativePercent}%; background: #ef4444; display: flex; align-items: center; justify-content: center; color: white; font-size: 0.8em; font-weight: bold;">
                            ${negativePercent > 10 ? negativePercent + '%' : ''}
                        </div>
                    </div>
                    <div style="display: flex; justify-content: space-between; margin-top: 8px; font-size: 0.8em; color: #6b7280;">
                        <span>🟢 Positive: ${positivePercent}%</span>
                        <span>🟡 Neutral: ${neutralPercent}%</span>
                        <span>🔴 Negative: ${negativePercent}%</span>
                    </div>
                `;
            }
        }

        async function loadNetwork() {
            const data = await fetchData('/api/network');
            if (!data || !data.nodes) return;
            
            const container = document.getElementById('network-container');
            container.innerHTML = ''; // Clear loading text
            
            // Simple D3-style network visualization using SVG
            const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            svg.setAttribute('width', '100%');
            svg.setAttribute('height', '400');
            svg.style.background = '#f9fafb';
            
            const width = 800, height = 400;
            
            // Color mapping for actor types
            const colors = {
                'world_leader': '#dc2626',
                'politician': '#2563eb', 
                'organization': '#059669',
                'person': '#7c3aed',
                'country': '#ea580c'
            };
            
            // Position nodes in a circle layout
            const nodes = data.nodes.map((node, i) => ({
                ...node,
                x: width/2 + Math.cos(2 * Math.PI * i / data.nodes.length) * 150,
                y: height/2 + Math.sin(2 * Math.PI * i / data.nodes.length) * 120
            }));
            
            // Draw links first (behind nodes)
            data.links.forEach(link => {
                const sourceNode = nodes.find(n => n.id === link.source);
                const targetNode = nodes.find(n => n.id === link.target);
                if (sourceNode && targetNode) {
                    const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                    line.setAttribute('x1', sourceNode.x);
                    line.setAttribute('y1', sourceNode.y);
                    line.setAttribute('x2', targetNode.x);
                    line.setAttribute('y2', targetNode.y);
                    line.setAttribute('stroke', '#6b7280');
                    line.setAttribute('stroke-width', Math.max(1, link.strength * 3));
                    line.setAttribute('opacity', '0.6');
                    svg.appendChild(line);
                }
            });
            
            // Draw nodes
            nodes.forEach(node => {
                const group = document.createElementNS('http://www.w3.org/2000/svg', 'g');
                
                // Node circle
                const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
                circle.setAttribute('cx', node.x);
                circle.setAttribute('cy', node.y);
                circle.setAttribute('r', Math.max(8, node.size / 2));
                circle.setAttribute('fill', colors[node.type] || '#6b7280');
                circle.setAttribute('stroke', '#ffffff');
                circle.setAttribute('stroke-width', '2');
                circle.style.cursor = 'pointer';
                
                // Node label
                const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                text.setAttribute('x', node.x);
                text.setAttribute('y', node.y - node.size/2 - 5);
                text.setAttribute('text-anchor', 'middle');
                text.setAttribute('font-size', '12');
                text.setAttribute('font-weight', 'bold');
                text.setAttribute('fill', '#374151');
                text.textContent = node.name.length > 15 ? node.name.substring(0, 15) + '...' : node.name;
                
                // Influence score
                const scoreText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                scoreText.setAttribute('x', node.x);
                scoreText.setAttribute('y', node.y + 4);
                scoreText.setAttribute('text-anchor', 'middle');
                scoreText.setAttribute('font-size', '10');
                scoreText.setAttribute('font-weight', 'bold');
                scoreText.setAttribute('fill', '#ffffff');
                scoreText.textContent = node.influence;
                
                // Add hover effects
                circle.addEventListener('mouseover', function() {
                    this.setAttribute('r', Math.max(10, node.size / 2 + 2));
                });
                circle.addEventListener('mouseout', function() {
                    this.setAttribute('r', Math.max(8, node.size / 2));
                });
                
                group.appendChild(circle);
                group.appendChild(text);
                group.appendChild(scoreText);
                svg.appendChild(group);
            });
            
            // Add legend
            const legendY = height - 30;
            let legendX = 20;
            
            Object.entries(colors).forEach(([type, color]) => {
                const legendCircle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
                legendCircle.setAttribute('cx', legendX);
                legendCircle.setAttribute('cy', legendY);
                legendCircle.setAttribute('r', '6');
                legendCircle.setAttribute('fill', color);
                
                const legendText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                legendText.setAttribute('x', legendX + 12);
                legendText.setAttribute('y', legendY + 4);
                legendText.setAttribute('font-size', '10');
                legendText.setAttribute('fill', '#374151');
                legendText.textContent = type.replace('_', ' ');
                
                svg.appendChild(legendCircle);
                svg.appendChild(legendText);
                
                legendX += type.length * 8 + 30;
            });
            
            container.appendChild(svg);
            
            if (nodes.length === 0) {
                container.innerHTML = '<div class="loading">No network data available</div>';
            }
        }

        async function loadArticles() {
            const data = await fetchData('/api/articles?limit=5');
            if (!data) return;
            
            document.getElementById('articles').innerHTML = data.map(article => `
                <div style="padding: 10px 0; border-bottom: 1px solid #f3f4f6;">
                    <div style="font-weight: bold; margin-bottom: 5px;">${article.title}</div>
                    <div style="color: #6b7280; font-size: 0.9em;">
                        ${article.source} • 
                        ${article.sentiment > 0 ? '📈' : article.sentiment < 0 ? '📉' : '➡️'} ${article.sentiment}
                    </div>
                </div>
            `).join('');
        }

        async function loadData() {
            console.log('Loading dashboard data...');
            await Promise.all([
                loadMetrics(),
                loadActors(), 
                loadInsights(),
                loadSentiment(),  // Add sentiment loading
                loadNetwork(),  
                loadArticles()
            ]);
            console.log('Dashboard loaded!');
        }

        // Load on page ready
        document.addEventListener('DOMContentLoaded', loadData);
        
        // Auto-refresh every 2 minutes (only if page visible)
        setInterval(() => {
            if (document.visibilityState === 'visible') {
                loadData();
            }
        }, 120000);
    </script>
</body>
</html>"""

@route('/api/metrics')
def api_metrics():
    """Ultra-fast metrics endpoint"""
    response.content_type = 'application/json'
    
    try:
        # Get counts directly from database
        articles_result = db.query_aql("RETURN LENGTH(articles)")
        actors_result = db.query_aql("RETURN LENGTH(actors)")
        
        # Count insights by framework
        insights_result = db.query_aql("""
            FOR insight IN jiang_insights
                COLLECT framework = insight.framework_type WITH COUNT INTO count
                RETURN {framework, count}
        """)
        
        articles_count = articles_result.get('result', [0])[0]
        actors_count = actors_result.get('result', [0])[0]
        insights_data = insights_result.get('result', [])
        
        # Process insights counts
        geostrategy = sum(i['count'] for i in insights_data if i['framework'] == 'geostrategy')
        game_theory = sum(i['count'] for i in insights_data if i['framework'] == 'game_theory') 
        secret_history = sum(i['count'] for i in insights_data if i['framework'] == 'secret_history')
        
        return {
            "articles": articles_count,
            "actors": actors_count,
            "geostrategy": geostrategy,
            "game_theory": game_theory, 
            "secret_history": secret_history,
            "total_insights": geostrategy + game_theory + secret_history
        }
    except Exception as e:
        return {"error": str(e)}

@route('/api/actors')
def api_actors():
    """Top actors endpoint"""
    response.content_type = 'application/json'
    limit = int(request.query.get('limit', 10))
    
    try:
        query = f"""
            FOR actor IN actors
                SORT actor.influence_score DESC
                LIMIT {limit}
                RETURN {{
                    name: actor.name,
                    type: actor.actor_type,
                    influence: actor.influence_score
                }}
        """
        result = db.query_aql(query)
        actors_data = result.get('result', [])
        return json.dumps(actors_data)  # Explicit JSON serialization
    except Exception as e:
        return json.dumps({"error": str(e)})

@route('/api/insights')  
def api_insights():
    """Recent insights endpoint"""
    response.content_type = 'application/json'
    limit = int(request.query.get('limit', 10))
    
    try:
        query = f"""
            FOR insight IN jiang_insights
                SORT insight.confidence_score DESC
                LIMIT {limit}
                RETURN {{
                    framework: insight.framework_type,
                    analysis: insight.analysis,
                    prediction: insight.prediction,
                    confidence: insight.confidence_score,
                    actors: insight.related_actors
                }}
        """
        result = db.query_aql(query)
        insights_data = result.get('result', [])
        return json.dumps(insights_data)  # Explicit JSON serialization
    except Exception as e:
        return json.dumps({"error": str(e)})

@route('/api/articles')
def api_articles():
    """Recent articles endpoint"""  
    response.content_type = 'application/json'
    limit = int(request.query.get('limit', 10))
    
    try:
        query = f"""
            FOR article IN articles
                SORT article.published_date DESC
                LIMIT {limit}
                RETURN {{
                    title: article.title,
                    source: article.source,
                    sentiment: article.sentiment_score || 0,
                    published: article.published_date
                }}
        """
        result = db.query_aql(query)
        articles_data = result.get('result', [])
        return json.dumps(articles_data)  # Explicit JSON serialization
    except Exception as e:
        return json.dumps({"error": str(e)})

@route('/api/sentiment')
def api_sentiment():
    """Sentiment analysis distribution"""
    response.content_type = 'application/json'
    
    try:
        # Simpler approach: get all articles and count manually
        query = "FOR article IN articles RETURN article.sentiment"
        result = db.query_aql(query)
        sentiments = list(result)
        
        # Initialize counters
        sentiment_data = {'positive': 0, 'neutral': 0, 'negative': 0}
        
        # Count manually
        for sentiment in sentiments:
            if isinstance(sentiment, (int, float)):
                if sentiment > 0:
                    sentiment_data['positive'] += 1
                elif sentiment < 0:
                    sentiment_data['negative'] += 1
                else:
                    sentiment_data['neutral'] += 1
        
        print(f"🎯 Sentiment distribution: {sentiment_data}")
        return json.dumps(sentiment_data)
        
    except Exception as e:
        print(f"❌ Sentiment API error: {e}")
        import traceback
        traceback.print_exc()
        return json.dumps({'positive': 0, 'neutral': 0, 'negative': 0, 'error': str(e)})

@route('/api/network')
def api_network():
    """Actor network graph data using existing correlates collection"""
    response.content_type = 'application/json'
    
    try:
        # Get top actors as nodes
        actors_query = """
            FOR actor IN actors
                SORT actor.influence_score DESC
                LIMIT 15
                RETURN {
                    id: actor._key,
                    name: actor.name,
                    type: actor.actor_type,
                    influence: actor.influence_score,
                    size: actor.influence_score * 3 + 10
                }
        """
        
        # Get correlations from existing correlates collection (edge collection)
        links_query = """
            FOR correlation IN correlates
                LET source_actor = DOCUMENT(correlation._from)
                LET target_actor = DOCUMENT(correlation._to)
                FILTER source_actor != null AND target_actor != null
                RETURN {
                    source: source_actor._key,
                    target: target_actor._key, 
                    strength: correlation.strength || 0.5,
                    type: correlation.correlation_type || 'related'
                }
        """
        
        actors_result = db.query_aql(actors_query)
        links_result = db.query_aql(links_query)
        
        nodes = actors_result.get('result', [])
        links = links_result.get('result', [])
        
        # Filter links to only include nodes that exist in our top 15
        node_ids = {node['id'] for node in nodes}
        filtered_links = [
            link for link in links
            if link['source'] in node_ids and link['target'] in node_ids
        ]
        
        # Add some manual high-profile connections if no links exist
        if not filtered_links and len(nodes) >= 3:
            # Create connections between top world leaders  
            manual_links = []
            world_leaders = [n for n in nodes if 'trump' in n['name'].lower() or 'biden' in n['name'].lower() or 'zelensky' in n['name'].lower()]
            
            if len(world_leaders) >= 2:
                for i in range(min(3, len(world_leaders))):
                    for j in range(i+1, min(3, len(world_leaders))):
                        manual_links.append({
                            'source': world_leaders[i]['id'],
                            'target': world_leaders[j]['id'],
                            'strength': 0.8,
                            'type': 'geopolitical'
                        })
            
            filtered_links = manual_links
        
        network_data = {
            "nodes": nodes,
            "links": filtered_links
        }
        
        return json.dumps(network_data)
        
    except Exception as e:
        return json.dumps({"error": str(e), "nodes": [], "links": []})

if __name__ == '__main__':
    print("🚀 Starting Ultra-Light News Intelligence Dashboard...")
    print("📊 Dashboard: http://localhost:3000")
    print("🔗 Tailscale: http://100.70.96.84:3000") 
    print("💡 Memory usage: ~10MB (vs Flask ~50MB)")
    
    run(host='0.0.0.0', port=3000, debug=False, quiet=True)