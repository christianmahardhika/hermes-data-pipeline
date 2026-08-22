#!/usr/bin/env python3
"""
ArangoDB News Intelligence Monitor
Replaces Qdrant monitoring with ArangoDB-focused metrics
"""

import requests
import json
import os
from datetime import datetime, timedelta

class ArangoNewsMonitor:
    def __init__(self):
        self.arango_url = 'http://localhost:8529'
        self.db_name = 'news_analysis'
        self.log_dir = '/home/ctianm/.hermes/profiles/social-politic-lab/logs'
        self.benchmark_file = os.path.join(self.log_dir, 'arangodb_benchmark.jsonl')
        
        # Ensure log directory exists
        os.makedirs(self.log_dir, exist_ok=True)
    
    def get_collection_stats(self):
        """Get ArangoDB collection statistics"""
        try:
            # Total articles count
            count_query = '''
            FOR article IN articles
            COLLECT WITH COUNT INTO total
            RETURN total
            '''
            
            response = requests.post(f'{self.arango_url}/_db/{self.db_name}/_api/cursor',
                                   json={'query': count_query}, timeout=30)
            
            if response.status_code == 201:
                total_articles = response.json().get('result', [0])[0]
            else:
                total_articles = 0
            
            # Source distribution
            source_query = '''
            FOR article IN articles
            COLLECT source = article.source WITH COUNT INTO count
            SORT count DESC
            LIMIT 15
            RETURN {source: source, count: count}
            '''
            
            source_response = requests.post(f'{self.arango_url}/_db/{self.db_name}/_api/cursor',
                                          json={'query': source_query}, timeout=30)
            
            sources = []
            if source_response.status_code == 201:
                sources = source_response.json().get('result', [])
            
            # Recent activity (last 24 hours)
            recent_query = '''
            FOR article IN articles
            FILTER article._key >= @yesterday_key
            COLLECT WITH COUNT INTO recent_total
            RETURN recent_total
            '''
            
            yesterday = datetime.now() - timedelta(days=1)
            yesterday_key = yesterday.strftime('%Y%m%d')
            
            recent_response = requests.post(f'{self.arango_url}/_db/{self.db_name}/_api/cursor',
                                          json={'query': recent_query, 
                                               'bindVars': {'yesterday_key': yesterday_key}}, timeout=30)
            
            recent_articles = 0
            if recent_response.status_code == 201:
                recent_articles = recent_response.json().get('result', [0])[0]
            
            return {
                'total_articles': total_articles,
                'sources': sources,
                'recent_24h': recent_articles,
                'status': 'healthy' if total_articles > 5000 else 'warning' if total_articles > 1000 else 'critical'
            }
            
        except Exception as e:
            return {
                'error': str(e),
                'total_articles': 0,
                'sources': [],
                'recent_24h': 0,
                'status': 'error'
            }
    
    def log_benchmark(self, stats):
        """Log benchmark data for trend analysis"""
        timestamp = datetime.now().isoformat()
        
        benchmark_entry = {
            'timestamp': timestamp,
            'total_articles': stats['total_articles'],
            'recent_24h': stats['recent_24h'],
            'status': stats['status'],
            'top_sources': stats['sources'][:5] if 'sources' in stats else []
        }
        
        try:
            with open(self.benchmark_file, 'a') as f:
                f.write(json.dumps(benchmark_entry) + '\n')
        except Exception as e:
            print(f"Failed to log benchmark: {e}")
    
    def generate_report(self, stats):
        """Generate monitoring report"""
        if 'error' in stats:
            return f"""
ArangoDB News Monitor Report - {datetime.now().strftime('%B %d, %Y')}

❌ CONNECTION ERROR: {stats['error']}

Status: Database connection failed. Check ArangoDB service.
"""
        
        # Status indicators
        status_icons = {
            'healthy': '✅',
            'warning': '⚠️', 
            'critical': '❌',
            'error': '💥'
        }
        
        status_icon = status_icons.get(stats['status'], '❓')
        
        # Indonesian vs International breakdown
        indonesian_count = 0
        international_count = 0
        
        for source_info in stats['sources']:
            source = source_info['source'].lower()
            count = source_info['count']
            
            if any(indo in source for indo in ['indonesia', 'kompas', 'liputan6', 'detik', 'tempo', 'antara', 'republika']):
                indonesian_count += count
            else:
                international_count += count
        
        report = f"""
ArangoDB News Intelligence Monitor Report - {datetime.now().strftime('%B %d, %Y')}

Database Status:
- Total Articles: {stats['total_articles']:,} {status_icon}
- Recent Activity (24h): {stats['recent_24h']} new articles
- System Status: {stats['status'].upper()}

Content Distribution:
- 🇮🇩 Indonesian Sources: {indonesian_count:,} articles
- 🌍 International Sources: {international_count:,} articles

Top Active Sources:
"""
        
        for i, source_info in enumerate(stats['sources'][:8], 1):
            source = source_info['source']
            count = source_info['count']
            
            # Clean source name
            source_clean = source.replace('smart_recovery_', '').replace('agent_reach_', '').replace('enhanced_indonesian_parser_', '').replace('_', ' ').title()[:30]
            
            # Source flag
            flag = '🇮🇩' if any(indo in source.lower() for indo in ['indonesia', 'kompas', 'liputan6', 'detik', 'tempo', 'antara']) else '🌍'
            
            report += f"- {flag} {source_clean:<30} {count:>6} articles\n"
        
        # System health assessment
        if stats['status'] == 'healthy':
            report += f"\n✅ STATUS: Excellent - Database healthy with robust article coverage"
        elif stats['status'] == 'warning':
            report += f"\n⚠️ STATUS: Warning - Article count below optimal threshold"  
        else:
            report += f"\n❌ STATUS: Critical - Low article count or system issues"
        
        report += f"\n📊 Benchmark logged to: {self.benchmark_file}"
        report += f"\n🔄 Next monitoring cycle: 24 hours"
        
        return report

def main():
    monitor = ArangoNewsMonitor()
    
    print("🔍 ArangoDB News Intelligence Monitor Starting...")
    
    # Get statistics
    stats = monitor.get_collection_stats()
    
    # Log benchmark data
    monitor.log_benchmark(stats)
    
    # Generate and print report
    report = monitor.generate_report(stats)
    print(report)
    
    return report

if __name__ == '__main__':
    main()