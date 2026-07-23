#!/usr/bin/env python3
"""
Enhanced Social Intelligence Collection with Domestic vs International Categorization
Organized into: Tech, Social, Politics, Business domains

Author: Christian Mahardhika
Focus: Systematic separation of Indonesian vs International intelligence across domains
"""

import asyncio
import json
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Tuple, Optional
import requests
from dataclasses import dataclass, asdict

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class IntelligenceReport:
    """Structured Intelligence Report with Domain Categorization"""
    timestamp: str
    domestic_intelligence: Dict[str, List[Dict]]  # Domain -> Indonesian content
    international_intelligence: Dict[str, List[Dict]]  # Domain -> Global content
    summary_insights: Dict[str, str]  # Domain -> Key insights
    correlation_analysis: Dict[str, float]  # Cross-domain correlation scores

class EnhancedSocialIntelligenceCollector:
    """Enhanced Social Intelligence with Domestic vs International Categorization"""
    
    def __init__(self):
        # Intelligence Domains
        self.domains = {
            'tech': {
                'domestic_keywords': ['Indonesia tech', 'startup Indonesia', 'fintech Indonesia', 'digital Indonesia', 'e-commerce Indonesia', 'Gojek', 'Tokopedia', 'Bukalapak', 'OVO', 'Dana', 'BRI Agro', 'Mandiri digital'],
                'international_keywords': ['global tech', 'AI breakthrough', 'cloud computing', 'cybersecurity', 'blockchain', 'quantum computing', 'semiconductor', 'EV technology', 'renewable energy tech'],
                'sources': ['HackerNews', 'Reddit Technology', 'YouTube Tech']
            },
            'social': {
                'domestic_keywords': ['masyarakat Indonesia', 'sosial Indonesia', 'budaya Indonesia', 'pendidikan Indonesia', 'kesehatan Indonesia', 'urbanisasi Jakarta', 'migrasi', 'demografi Indonesia'],
                'international_keywords': ['global society', 'social trends', 'demographics', 'migration', 'urbanization', 'social media impact', 'cultural shifts', 'education trends'],
                'sources': ['Reddit Social', 'YouTube Social', 'Twitter Trends']
            },
            'politics': {
                'domestic_keywords': ['politik Indonesia', 'pemerintah Indonesia', 'Jokowi', 'DPR RI', 'pemilu Indonesia', 'kebijakan pemerintah', 'otonomi daerah', 'reformasi birokrasi'],
                'international_keywords': ['geopolitics', 'international relations', 'ASEAN politics', 'US China relations', 'global governance', 'trade wars', 'sanctions', 'diplomatic relations'],
                'sources': ['Reddit Politics', 'YouTube Politics', 'News Feeds']
            },
            'business': {
                'domestic_keywords': ['bisnis Indonesia', 'ekonomi Indonesia', 'BUMN', 'IPO Indonesia', 'merger akuisisi', 'investasi Indonesia', 'startup funding', 'Indonesian corporates', 'IDX listing'],
                'international_keywords': ['global business', 'international trade', 'multinational corporations', 'global markets', 'commodity markets', 'supply chain', 'international investment', 'global economic trends'],
                'sources': ['HackerNews Business', 'Reddit Business', 'YouTube Business']
            }
        }
        
        # Christian's Portfolio Context for Business Intelligence
        self.portfolio_context = {
            'banking': ['BMRI', 'BBRI', 'digital banking', 'fintech regulation'],
            'mining': ['INCO', 'ANTM', 'PTBA', 'nickel industry', 'coal industry'],
            'agriculture': ['TAPG', 'palm oil industry', 'agricultural export'],
            'consumer': ['KLBF', 'TSPC', 'consumer behavior', 'retail trends'],
            'infrastructure': ['TLKM', 'ASII', 'telecommunications', 'automotive industry']
        }
    
    def collect_domain_intelligence(self, domain: str, depth: str = 'default') -> Tuple[List[Dict], List[Dict]]:
        """Collect intelligence for specific domain, separated by domestic vs international"""
        domestic_content = []
        international_content = []
        
        domain_config = self.domains.get(domain, {})
        domestic_keywords = domain_config.get('domestic_keywords', [])
        international_keywords = domain_config.get('international_keywords', [])
        
        # Simulate collection (in production, this would call actual APIs)
        logger.info(f"🔍 Collecting {domain.upper()} intelligence...")
        
        # Domestic collection simulation
        for keyword in domestic_keywords[:3]:  # Limit for testing
            domestic_content.append({
                'keyword': keyword,
                'title': f"Mock Indonesian {domain} news about {keyword}",
                'source': 'Indonesian Sources',
                'sentiment': 0.6,
                'relevance': 0.8,
                'timestamp': datetime.now().isoformat()
            })
        
        # International collection simulation  
        for keyword in international_keywords[:3]:  # Limit for testing
            international_content.append({
                'keyword': keyword,
                'title': f"Mock global {domain} news about {keyword}",
                'source': 'International Sources',
                'sentiment': 0.5,
                'relevance': 0.7,
                'timestamp': datetime.now().isoformat()
            })
        
        logger.info(f"✅ {domain.upper()}: {len(domestic_content)} domestic + {len(international_content)} international items")
        return domestic_content, international_content
    
    def analyze_portfolio_business_correlation(self, business_content: List[Dict]) -> Dict[str, float]:
        """Analyze business intelligence correlation with Christian's portfolio"""
        correlations = {}
        
        for sector, keywords in self.portfolio_context.items():
            correlation_score = 0
            relevant_items = 0
            
            for item in business_content:
                for keyword in keywords:
                    if keyword.lower() in item['title'].lower():
                        correlation_score += item['sentiment'] * item['relevance']
                        relevant_items += 1
            
            correlations[sector] = correlation_score / max(relevant_items, 1)
        
        return correlations
    
    def generate_domain_insights(self, domain: str, domestic_content: List[Dict], international_content: List[Dict]) -> str:
        """Generate key insights for domain"""
        domestic_sentiment = sum(item['sentiment'] for item in domestic_content) / max(len(domestic_content), 1)
        international_sentiment = sum(item['sentiment'] for item in international_content) / max(len(international_content), 1)
        
        sentiment_comparison = "more positive" if domestic_sentiment > international_sentiment else "more challenging" if domestic_sentiment < international_sentiment else "similar"
        
        insight = f"Indonesian {domain} shows {sentiment_comparison} sentiment vs international trends. "
        insight += f"Domestic focus: {domestic_content[0]['keyword'] if domestic_content else 'N/A'}, "
        insight += f"Global trend: {international_content[0]['keyword'] if international_content else 'N/A'}"
        
        return insight
    
    def calculate_cross_domain_correlations(self, all_content: Dict) -> Dict[str, float]:
        """Calculate correlations between different domains"""
        correlations = {}
        
        domains_list = list(self.domains.keys())
        for i, domain1 in enumerate(domains_list):
            for domain2 in domains_list[i+1:]:
                # Simplified correlation calculation
                domain1_sentiment = sum(item['sentiment'] for content_list in [all_content['domestic'][domain1], all_content['international'][domain1]] for item in content_list)
                domain2_sentiment = sum(item['sentiment'] for content_list in [all_content['domestic'][domain2], all_content['international'][domain2]] for item in content_list)
                
                correlation = min(abs(domain1_sentiment - domain2_sentiment) / 10, 1)  # Normalized correlation
                correlations[f"{domain1}_vs_{domain2}"] = correlation
        
        return correlations
    
    def generate_comprehensive_report(self, depth: str = 'default') -> IntelligenceReport:
        """Generate comprehensive intelligence report with domain categorization"""
        logger.info("🎯 Generating Enhanced Social Intelligence Report...")
        
        domestic_intelligence = {}
        international_intelligence = {}
        summary_insights = {}
        
        # Collect intelligence for each domain
        for domain in self.domains.keys():
            domestic_content, international_content = self.collect_domain_intelligence(domain, depth)
            
            domestic_intelligence[domain] = domestic_content
            international_intelligence[domain] = international_content
            summary_insights[domain] = self.generate_domain_insights(domain, domestic_content, international_content)
        
        # Calculate cross-domain correlations
        all_content = {
            'domestic': domestic_intelligence,
            'international': international_intelligence
        }
        correlation_analysis = self.calculate_cross_domain_correlations(all_content)
        
        # Add portfolio correlation for business domain
        if 'business' in domestic_intelligence:
            business_portfolio_correlation = self.analyze_portfolio_business_correlation(
                domestic_intelligence['business'] + international_intelligence['business']
            )
            correlation_analysis.update(business_portfolio_correlation)
        
        report = IntelligenceReport(
            timestamp=datetime.now().isoformat(),
            domestic_intelligence=domestic_intelligence,
            international_intelligence=international_intelligence,
            summary_insights=summary_insights,
            correlation_analysis=correlation_analysis
        )
        
        return report
    
    def format_telegram_report(self, report: IntelligenceReport) -> str:
        """Format report for Telegram delivery"""
        output = "🎯 **ENHANCED SOCIAL INTELLIGENCE REPORT**\n"
        output += f"📅 {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n"
        
        # Domain summaries
        for domain in self.domains.keys():
            domain_emoji = {'tech': '💻', 'social': '👥', 'politics': '🏛️', 'business': '💼'}
            emoji = domain_emoji.get(domain, '📊')
            
            output += f"{emoji} **{domain.upper()} INTELLIGENCE**\n"
            output += f"🇮🇩 Indonesian: {len(report.domestic_intelligence.get(domain, []))} items\n"
            output += f"🌍 International: {len(report.international_intelligence.get(domain, []))}) items\n"
            output += f"💡 Insight: {report.summary_insights.get(domain, 'No insights')}\n\n"
        
        # Correlation analysis
        output += "📊 **CORRELATION ANALYSIS**\n"
        for correlation, score in report.correlation_analysis.items():
            output += f"• {correlation}: {score:.3f}\n"
        
        output += f"\n✅ Enhanced Social Intelligence Collection Complete!"
        return output

def main():
    """Main execution function"""
    collector = EnhancedSocialIntelligenceCollector()
    
    print("🎯 Enhanced Social Intelligence Collection")
    print("=" * 80)
    
    # Generate comprehensive report
    report = collector.generate_comprehensive_report()
    
    # Save detailed report
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"enhanced_social_intelligence_{timestamp}.json"
    
    with open(output_file, 'w') as f:
        json.dump(asdict(report), f, indent=2)
    
    # Generate Telegram summary
    telegram_summary = collector.format_telegram_report(report)
    summary_file = f"social_intelligence_summary_{timestamp}.txt"
    
    with open(summary_file, 'w') as f:
        f.write(telegram_summary)
    
    print(f"📊 Enhanced Social Intelligence Report Generated!")
    print(f"📄 Detailed Report: {output_file}")
    print(f"📱 Telegram Summary: {summary_file}")
    
    # Display summary
    print("\n" + telegram_summary)

if __name__ == "__main__":
    main()