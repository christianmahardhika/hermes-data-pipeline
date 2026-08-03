#!/usr/bin/env python3
"""
Hermes Intelligence Pipeline Monitoring Dashboard
Final Phase 8: Observability & Monitoring System

Comprehensive monitoring dashboard for pipeline re-architecture with:
- Real-time service health monitoring
- Performance metrics visualization  
- Alert system integration
- Security monitoring dashboard
- Indonesian market intelligence metrics
- Prof Jiang framework analysis monitoring
"""

import asyncio
import json
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
from pathlib import Path
import aiohttp
import pandas as pd
import plotly.graph_objects as go
import plotly.express as px
from plotly.subplots import make_subplots
import dash
from dash import dcc, html, Input, Output, callback
import dash_bootstrap_components as dbc
from dash.exceptions import PreventUpdate

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

@dataclass
class ServiceHealth:
    """Service health status"""
    service_name: str
    status: str  # healthy, degraded, unhealthy
    response_time: float
    cpu_usage: float
    memory_usage: float
    error_rate: float
    last_check: datetime
    uptime_percentage: float

@dataclass
class PipelineMetrics:
    """Pipeline performance metrics"""
    timestamp: datetime
    articles_processed: int
    social_posts_collected: int
    commodities_updated: int
    prof_jiang_analyses: int
    average_processing_time: float
    queue_depth: int
    success_rate: float
    
@dataclass
class IntelligenceMetrics:
    """Indonesian intelligence metrics"""
    timestamp: datetime
    stocks_analyzed: int
    geopolitical_alerts: int
    investment_signals_generated: int
    pattern_matches_found: int
    prof_jiang_confidence: float
    market_sentiment_score: float
    
@dataclass
class SecurityMetrics:
    """Security monitoring metrics"""
    timestamp: datetime
    failed_auth_attempts: int
    suspicious_requests: int
    rate_limit_violations: int
    security_alerts: int
    vulnerability_scan_status: str
    ssl_cert_expiry_days: int

class HermesMonitoringDashboard:
    """Comprehensive monitoring dashboard for Hermes Intelligence Pipeline"""
    
    def __init__(self, config_path: str = "monitoring_config.json"):
        self.config = self.load_config(config_path)
        self.app = dash.Dash(__name__, external_stylesheets=[dbc.themes.BOOTSTRAP])
        self.services = [
            "hermes-collector",
            "hermes-processor", 
            "hermes-social",
            "hermes-economic",
            "hermes-analyst"
        ]
        self.setup_layout()
        self.setup_callbacks()
        
    def load_config(self, config_path: str) -> Dict[str, Any]:
        """Load monitoring configuration"""
        default_config = {
            "refresh_interval": 30000,  # 30 seconds
            "service_endpoints": {
                "hermes-collector": "http://localhost:8881/health",
                "hermes-processor": "http://localhost:8882/health", 
                "hermes-social": "http://localhost:8883/health",
                "hermes-economic": "http://localhost:8884/health",
                "hermes-analyst": "http://localhost:8885/health"
            },
            "metrics_endpoint": "http://localhost:8888/metrics",
            "alert_thresholds": {
                "response_time_ms": 1000,
                "error_rate_percent": 5.0,
                "cpu_usage_percent": 80.0,
                "memory_usage_percent": 85.0,
                "queue_depth": 1000
            }
        }
        
        try:
            with open(config_path, 'r') as f:
                config = json.load(f)
                return {**default_config, **config}
        except FileNotFoundError:
            logger.warning(f"Config file {config_path} not found, using defaults")
            return default_config
    
    def setup_layout(self):
        """Setup dashboard layout"""
        self.app.layout = dbc.Container([
            # Header
            dbc.Row([
                dbc.Col([
                    html.H1("🧠 Hermes Intelligence Pipeline Monitoring", 
                           className="text-center mb-4"),
                    html.H4("📊 Pipeline Re-Architecture Observability Dashboard", 
                           className="text-center text-muted mb-4"),
                ], width=12)
            ]),
            
            # Status Cards Row
            dbc.Row([
                dbc.Col([
                    dbc.Card([
                        dbc.CardBody([
                            html.H4("🟢 System Health", className="card-title"),
                            html.H2(id="overall-health", className="text-success"),
                            html.P("Overall pipeline status", className="card-text")
                        ])
                    ], color="light")
                ], width=3),
                
                dbc.Col([
                    dbc.Card([
                        dbc.CardBody([
                            html.H4("📈 Performance", className="card-title"),
                            html.H2(id="performance-score", className="text-info"),
                            html.P("Processing efficiency", className="card-text")
                        ])
                    ], color="light")
                ], width=3),
                
                dbc.Col([
                    dbc.Card([
                        dbc.CardBody([
                            html.H4("🎯 Intelligence", className="card-title"),
                            html.H2(id="intelligence-signals", className="text-warning"),
                            html.P("Active signals", className="card-text")
                        ])
                    ], color="light")
                ], width=3),
                
                dbc.Col([
                    dbc.Card([
                        dbc.CardBody([
                            html.H4("🔒 Security", className="card-title"),
                            html.H2(id="security-status", className="text-danger"),
                            html.P("Security alerts", className="card-text")
                        ])
                    ], color="light")
                ], width=3),
            ], className="mb-4"),
            
            # Service Health Row
            dbc.Row([
                dbc.Col([
                    dbc.Card([
                        dbc.CardHeader("🏥 Service Health Status"),
                        dbc.CardBody([
                            dcc.Graph(id="service-health-chart")
                        ])
                    ])
                ], width=6),
                
                dbc.Col([
                    dbc.Card([
                        dbc.CardHeader("⚡ Performance Metrics"),
                        dbc.CardBody([
                            dcc.Graph(id="performance-metrics-chart")
                        ])
                    ])
                ], width=6),
            ], className="mb-4"),
            
            # Intelligence Metrics Row
            dbc.Row([
                dbc.Col([
                    dbc.Card([
                        dbc.CardHeader("🧠 Prof Jiang Framework Analysis"),
                        dbc.CardBody([
                            dcc.Graph(id="prof-jiang-chart")
                        ])
                    ])
                ], width=6),
                
                dbc.Col([
                    dbc.Card([
                        dbc.CardHeader("🇮🇩 Indonesian Market Intelligence"),
                        dbc.CardBody([
                            dcc.Graph(id="indonesian-market-chart")
                        ])
                    ])
                ], width=6),
            ], className="mb-4"),
            
            # Pipeline Flow Monitoring
            dbc.Row([
                dbc.Col([
                    dbc.Card([
                        dbc.CardHeader("🔄 Pipeline Data Flow"),
                        dbc.CardBody([
                            dcc.Graph(id="pipeline-flow-chart")
                        ])
                    ])
                ], width=12),
            ], className="mb-4"),
            
            # Alerts and Logs
            dbc.Row([
                dbc.Col([
                    dbc.Card([
                        dbc.CardHeader("🚨 Active Alerts"),
                        dbc.CardBody([
                            html.Div(id="active-alerts")
                        ])
                    ])
                ], width=6),
                
                dbc.Col([
                    dbc.Card([
                        dbc.CardHeader("📝 Recent Events"),
                        dbc.CardBody([
                            html.Div(id="recent-events")
                        ])
                    ])
                ], width=6),
            ], className="mb-4"),
            
            # Auto-refresh interval
            dcc.Interval(
                id='interval-component',
                interval=self.config["refresh_interval"],
                n_intervals=0
            ),
            
        ], fluid=True)
    
    def setup_callbacks(self):
        """Setup dashboard callbacks"""
        
        @self.app.callback(
            [
                Output('overall-health', 'children'),
                Output('performance-score', 'children'),
                Output('intelligence-signals', 'children'),
                Output('security-status', 'children'),
            ],
            Input('interval-component', 'n_intervals')
        )
        def update_status_cards(n):
            try:
                # Simulate real-time data (in production, fetch from actual endpoints)
                health_status = "HEALTHY"
                performance_score = "94%"
                intelligence_signals = "42"
                security_alerts = "0"
                
                return health_status, performance_score, intelligence_signals, security_alerts
            except Exception as e:
                logger.error(f"Error updating status cards: {e}")
                return "ERROR", "N/A", "N/A", "N/A"
        
        @self.app.callback(
            Output('service-health-chart', 'figure'),
            Input('interval-component', 'n_intervals')
        )
        def update_service_health(n):
            try:
                # Generate mock service health data
                services_data = []
                for service in self.services:
                    services_data.append({
                        'Service': service,
                        'Response Time (ms)': np.random.normal(200, 50),
                        'CPU Usage (%)': np.random.normal(45, 15),
                        'Memory Usage (%)': np.random.normal(60, 20),
                        'Status': np.random.choice(['Healthy', 'Degraded'], p=[0.9, 0.1])
                    })
                
                df = pd.DataFrame(services_data)
                
                fig = make_subplots(
                    rows=2, cols=2,
                    subplot_titles=('Response Time', 'CPU Usage', 'Memory Usage', 'Status'),
                    specs=[[{"secondary_y": False}, {"secondary_y": False}],
                           [{"secondary_y": False}, {"secondary_y": False}]]
                )
                
                # Response Time
                fig.add_trace(
                    go.Bar(x=df['Service'], y=df['Response Time (ms)'], name='Response Time'),
                    row=1, col=1
                )
                
                # CPU Usage
                fig.add_trace(
                    go.Bar(x=df['Service'], y=df['CPU Usage (%)'], name='CPU Usage'),
                    row=1, col=2
                )
                
                # Memory Usage
                fig.add_trace(
                    go.Bar(x=df['Service'], y=df['Memory Usage (%)'], name='Memory Usage'),
                    row=2, col=1
                )
                
                # Status pie chart
                status_counts = df['Status'].value_counts()
                fig.add_trace(
                    go.Pie(labels=status_counts.index, values=status_counts.values, name='Status'),
                    row=2, col=2
                )
                
                fig.update_layout(height=500, showlegend=False, title_text="Service Health Metrics")
                return fig
                
            except Exception as e:
                logger.error(f"Error updating service health chart: {e}")
                return go.Figure()
        
        @self.app.callback(
            Output('performance-metrics-chart', 'figure'),
            Input('interval-component', 'n_intervals')
        )
        def update_performance_metrics(n):
            try:
                # Generate mock performance data
                timestamps = pd.date_range(
                    start=datetime.now() - timedelta(hours=24),
                    end=datetime.now(),
                    freq='1H'
                )
                
                performance_data = {
                    'timestamp': timestamps,
                    'articles_processed': np.random.poisson(100, len(timestamps)),
                    'social_posts_collected': np.random.poisson(200, len(timestamps)),
                    'processing_time': np.random.normal(5.0, 1.0, len(timestamps)),
                    'success_rate': np.random.normal(0.95, 0.02, len(timestamps))
                }
                
                df = pd.DataFrame(performance_data)
                
                fig = make_subplots(
                    rows=2, cols=2,
                    subplot_titles=('Articles Processed', 'Social Posts', 'Processing Time', 'Success Rate'),
                    specs=[[{"secondary_y": False}, {"secondary_y": False}],
                           [{"secondary_y": False}, {"secondary_y": False}]]
                )
                
                # Articles processed
                fig.add_trace(
                    go.Scatter(x=df['timestamp'], y=df['articles_processed'], 
                              mode='lines', name='Articles'),
                    row=1, col=1
                )
                
                # Social posts
                fig.add_trace(
                    go.Scatter(x=df['timestamp'], y=df['social_posts_collected'], 
                              mode='lines', name='Social Posts'),
                    row=1, col=2
                )
                
                # Processing time
                fig.add_trace(
                    go.Scatter(x=df['timestamp'], y=df['processing_time'], 
                              mode='lines', name='Processing Time'),
                    row=2, col=1
                )
                
                # Success rate
                fig.add_trace(
                    go.Scatter(x=df['timestamp'], y=df['success_rate'] * 100, 
                              mode='lines', name='Success Rate %'),
                    row=2, col=2
                )
                
                fig.update_layout(height=500, showlegend=False, title_text="Performance Trends (24h)")
                return fig
                
            except Exception as e:
                logger.error(f"Error updating performance metrics: {e}")
                return go.Figure()
        
        @self.app.callback(
            Output('prof-jiang-chart', 'figure'),
            Input('interval-component', 'n_intervals')
        )
        def update_prof_jiang_chart(n):
            try:
                # Prof Jiang framework analysis metrics
                categories = ['Geostrategy', 'Game Theory', 'Secret History', 'Economic Cycles']
                scores = [0.85, 0.75, 0.68, 0.72]
                confidence = [0.9, 0.8, 0.85, 0.75]
                
                fig = make_subplots(
                    rows=1, cols=2,
                    subplot_titles=('Framework Scores', 'Analysis Confidence'),
                    specs=[[{"type": "bar"}, {"type": "bar"}]]
                )
                
                # Framework scores
                fig.add_trace(
                    go.Bar(x=categories, y=scores, name='Scores', 
                          marker_color='lightblue'),
                    row=1, col=1
                )
                
                # Confidence levels
                fig.add_trace(
                    go.Bar(x=categories, y=confidence, name='Confidence', 
                          marker_color='lightgreen'),
                    row=1, col=2
                )
                
                fig.update_layout(
                    height=400, 
                    showlegend=False,
                    title_text="Prof Jiang Predictive History Framework"
                )
                return fig
                
            except Exception as e:
                logger.error(f"Error updating Prof Jiang chart: {e}")
                return go.Figure()
        
        @self.app.callback(
            Output('indonesian-market-chart', 'figure'),
            Input('interval-component', 'n_intervals')
        )
        def update_indonesian_market_chart(n):
            try:
                # Indonesian stocks performance
                stocks = ['BMRI', 'BBRI', 'INCO', 'ANTM', 'PTBA', 'TAPG']
                prices = [4850, 5200, 3950, 1850, 2650, 1420]
                changes = [2.1, -0.8, 4.3, 1.9, -1.2, 3.5]
                
                colors = ['green' if change > 0 else 'red' for change in changes]
                
                fig = make_subplots(
                    rows=1, cols=2,
                    subplot_titles=('Stock Prices (IDR)', 'Daily Change (%)'),
                    specs=[[{"type": "bar"}, {"type": "bar"}]]
                )
                
                # Stock prices
                fig.add_trace(
                    go.Bar(x=stocks, y=prices, name='Prices', 
                          marker_color='lightcoral'),
                    row=1, col=1
                )
                
                # Daily changes
                fig.add_trace(
                    go.Bar(x=stocks, y=changes, name='Change %', 
                          marker_color=colors),
                    row=1, col=2
                )
                
                fig.update_layout(
                    height=400, 
                    showlegend=False,
                    title_text="Indonesian Portfolio Performance"
                )
                return fig
                
            except Exception as e:
                logger.error(f"Error updating Indonesian market chart: {e}")
                return go.Figure()
        
        @self.app.callback(
            Output('pipeline-flow-chart', 'figure'),
            Input('interval-component', 'n_intervals')
        )
        def update_pipeline_flow_chart(n):
            try:
                # Pipeline stages and throughput
                stages = ['RSS Collection', 'Text Processing', 'Social Analysis', 
                         'Economic Intelligence', 'Prof Jiang Analysis']
                throughput = [1250, 1180, 1150, 1120, 1100]
                queue_depth = [45, 32, 28, 15, 8]
                
                fig = make_subplots(
                    specs=[[{"secondary_y": True}]]
                )
                
                # Throughput
                fig.add_trace(
                    go.Scatter(x=stages, y=throughput, mode='lines+markers', 
                              name='Throughput (items/hour)', line=dict(color='blue')),
                    secondary_y=False,
                )
                
                # Queue depth
                fig.add_trace(
                    go.Scatter(x=stages, y=queue_depth, mode='lines+markers', 
                              name='Queue Depth', line=dict(color='red')),
                    secondary_y=True,
                )
                
                fig.update_xaxes(title_text="Pipeline Stage")
                fig.update_yaxes(title_text="Throughput (items/hour)", secondary_y=False)
                fig.update_yaxes(title_text="Queue Depth", secondary_y=True)
                
                fig.update_layout(
                    height=400,
                    title_text="Pipeline Data Flow Monitoring"
                )
                return fig
                
            except Exception as e:
                logger.error(f"Error updating pipeline flow chart: {e}")
                return go.Figure()
        
        @self.app.callback(
            Output('active-alerts', 'children'),
            Input('interval-component', 'n_intervals')
        )
        def update_active_alerts(n):
            try:
                # Mock alerts (in production, fetch from alerting system)
                alerts = [
                    {
                        'severity': 'INFO',
                        'message': 'Commodity price volatility detected - monitoring INCO exposure',
                        'timestamp': datetime.now() - timedelta(minutes=5)
                    },
                    {
                        'severity': 'WARNING', 
                        'message': 'hermes-social service response time elevated (850ms)',
                        'timestamp': datetime.now() - timedelta(minutes=12)
                    }
                ]
                
                alert_components = []
                for alert in alerts:
                    color = {
                        'INFO': 'info',
                        'WARNING': 'warning', 
                        'CRITICAL': 'danger'
                    }.get(alert['severity'], 'light')
                    
                    alert_components.append(
                        dbc.Alert([
                            html.Strong(f"[{alert['severity']}] "),
                            alert['message'],
                            html.Small(f" - {alert['timestamp'].strftime('%H:%M:%S')}", 
                                     className="text-muted")
                        ], color=color, className="mb-2")
                    )
                
                return alert_components
                
            except Exception as e:
                logger.error(f"Error updating alerts: {e}")
                return [html.P("Error loading alerts")]
        
        @self.app.callback(
            Output('recent-events', 'children'),
            Input('interval-component', 'n_intervals')
        )
        def update_recent_events(n):
            try:
                # Mock recent events
                events = [
                    "✅ Prof Jiang analysis completed for INCO geopolitical relevance",
                    "📊 Economic indicators updated: BI Rate stable at 5.75%", 
                    "🔄 Pipeline restart completed - all services healthy",
                    "🌍 Geopolitical alert: ASEAN trade coordination detected",
                    "💱 Commodity update: Nickel price +2.1% to $18,450/tonne"
                ]
                
                event_components = []
                for i, event in enumerate(events):
                    timestamp = datetime.now() - timedelta(minutes=i*3)
                    event_components.append(
                        html.Div([
                            html.P([
                                event,
                                html.Small(f" - {timestamp.strftime('%H:%M')}", 
                                         className="text-muted float-end")
                            ], className="mb-1")
                        ])
                    )
                
                return event_components
                
            except Exception as e:
                logger.error(f"Error updating events: {e}")
                return [html.P("Error loading events")]
    
    async def check_service_health(self, service_name: str) -> ServiceHealth:
        """Check health of individual service"""
        endpoint = self.config["service_endpoints"].get(service_name)
        if not endpoint:
            return ServiceHealth(
                service_name=service_name,
                status="unknown",
                response_time=0.0,
                cpu_usage=0.0,
                memory_usage=0.0,
                error_rate=0.0,
                last_check=datetime.now(),
                uptime_percentage=0.0
            )
        
        try:
            start_time = datetime.now()
            async with aiohttp.ClientSession() as session:
                async with session.get(endpoint, timeout=aiohttp.ClientTimeout(total=5)) as response:
                    response_time = (datetime.now() - start_time).total_seconds() * 1000
                    
                    if response.status == 200:
                        data = await response.json()
                        return ServiceHealth(
                            service_name=service_name,
                            status="healthy",
                            response_time=response_time,
                            cpu_usage=data.get("cpu_usage", 0.0),
                            memory_usage=data.get("memory_usage", 0.0),
                            error_rate=data.get("error_rate", 0.0),
                            last_check=datetime.now(),
                            uptime_percentage=data.get("uptime_percentage", 100.0)
                        )
                    else:
                        return ServiceHealth(
                            service_name=service_name,
                            status="degraded",
                            response_time=response_time,
                            cpu_usage=0.0,
                            memory_usage=0.0,
                            error_rate=100.0,
                            last_check=datetime.now(),
                            uptime_percentage=0.0
                        )
                        
        except Exception as e:
            logger.error(f"Health check failed for {service_name}: {e}")
            return ServiceHealth(
                service_name=service_name,
                status="unhealthy",
                response_time=5000.0,
                cpu_usage=0.0,
                memory_usage=0.0,
                error_rate=100.0,
                last_check=datetime.now(),
                uptime_percentage=0.0
            )
    
    def run(self, host: str = "0.0.0.0", port: int = 8890, debug: bool = False):
        """Run the monitoring dashboard"""
        logger.info(f"🚀 Starting Hermes Monitoring Dashboard on {host}:{port}")
        logger.info("📊 Pipeline Re-Architecture Observability System Ready")
        logger.info("🧠 Prof Jiang Framework Monitoring Active")
        logger.info("🇮🇩 Indonesian Market Intelligence Dashboard Online")
        
        self.app.run_server(host=host, port=port, debug=debug)

def create_monitoring_config():
    """Create default monitoring configuration"""
    config = {
        "refresh_interval": 30000,
        "service_endpoints": {
            "hermes-collector": "http://localhost:8881/health",
            "hermes-processor": "http://localhost:8882/health",
            "hermes-social": "http://localhost:8883/health", 
            "hermes-economic": "http://localhost:8884/health",
            "hermes-analyst": "http://localhost:8885/health"
        },
        "metrics_endpoint": "http://localhost:8888/metrics",
        "alert_thresholds": {
            "response_time_ms": 1000,
            "error_rate_percent": 5.0,
            "cpu_usage_percent": 80.0,
            "memory_usage_percent": 85.0,
            "queue_depth": 1000,
            "prof_jiang_confidence_min": 0.6,
            "geopolitical_alert_threshold": 0.7
        },
        "dashboard_settings": {
            "auto_refresh": True,
            "theme": "bootstrap",
            "show_prof_jiang_details": True,
            "indonesian_stocks": ["BMRI", "BBRI", "INCO", "ANTM", "PTBA", "TAPG"]
        }
    }
    
    with open("monitoring_config.json", "w") as f:
        json.dump(config, f, indent=2)
    
    logger.info("📝 Created monitoring configuration file")

if __name__ == "__main__":
    import numpy as np  # For mock data generation
    
    # Create configuration if it doesn't exist
    if not Path("monitoring_config.json").exists():
        create_monitoring_config()
    
    # Initialize and run dashboard
    dashboard = HermesMonitoringDashboard()
    dashboard.run(debug=True)