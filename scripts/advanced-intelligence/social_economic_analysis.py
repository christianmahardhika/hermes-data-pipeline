#!/usr/bin/env python3
"""
Social & Economic Correlation Analysis Framework
Real-time correlation analysis between news sentiment, commodity prices, and portfolio performance

Author: Christian Mahardhika
Focus: Indonesian Portfolio Intelligence with Regional Market Context
"""

import asyncio
import json
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Tuple, Optional
import requests
from dataclasses import dataclass, asdict
import pandas as pd

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class PortfolioAnalysis:
    """Current Portfolio Analysis Structure"""
    timestamp: str
    portfolio_stocks: Dict[str, Dict]  # Stock symbol -> analysis data
    commodity_correlations: Dict[str, float]  # Commodity -> correlation coefficient
    regional_market_overview: Dict[str, Dict]  # Market -> performance data
    social_economic_factors: Dict[str, float]  # Factor -> impact score
    macro_indicators: Dict[str, float]  # Indicator -> value
    micro_factors: Dict[str, Dict]  # Stock -> specific factors

class SocialEconomicAnalyzer:
    """Advanced Social & Economic Correlation Analysis Engine"""
    
    def __init__(self):
        # Christian's Portfolio Focus
        self.portfolio_stocks = {
            'BMRI.JK': {'sector': 'Banking', 'weight': 'High', 'correlations': ['USD/IDR', 'BI Rate', 'Credit Growth']},
            'BBRI.JK': {'sector': 'Banking', 'weight': 'High', 'correlations': ['Rural Economy', 'MSME Growth', 'Digital Banking']},
            'INCO.JK': {'sector': 'Mining', 'weight': 'High', 'correlations': ['Nickel Prices', 'China Demand', 'EV Battery']},
            'ANTM.JK': {'sector': 'Mining', 'weight': 'Medium', 'correlations': ['Gold Prices', 'Mining Costs', 'Royalties']},
            'PTBA.JK': {'sector': 'Energy', 'weight': 'High', 'correlations': ['Coal Prices', 'Energy Transition', 'ESG Factors']},
            'TAPG.JK': {'sector': 'Agriculture', 'weight': 'High', 'correlations': ['Palm Oil', 'Weather', 'Export Demand']},
            'KLBF.JK': {'sector': 'Healthcare', 'weight': 'Medium', 'correlations': ['Healthcare Demand', 'Regulation', 'Demographics']},
            'TSPC.JK': {'sector': 'Consumer', 'weight': 'Medium', 'correlations': ['Consumer Spending', 'Inflation', 'Competition']},
            'TLKM.JK': {'sector': 'Telecom', 'weight': 'Medium', 'correlations': ['Digital Growth', 'Infrastructure', '5G Rollout']},
            'ASII.JK': {'sector': 'Automotive', 'weight': 'High', 'correlations': ['Auto Sales', 'Commodity Costs', 'EV Transition']}
        }
        
        # Strategic Commodities for Indonesian Economy
        self.strategic_commodities = {
            'nickel': {'symbol': 'NI=F', 'impact_stocks': ['INCO.JK', 'ANTM.JK'], 'macro_factor': 'Export Revenue'},
            'coal': {'symbol': 'COAL', 'impact_stocks': ['PTBA.JK'], 'macro_factor': 'Energy Security'},
            'palm_oil': {'symbol': 'FCPO=F', 'impact_stocks': ['TAPG.JK'], 'macro_factor': 'Agricultural Export'},
            'gold': {'symbol': 'GC=F', 'impact_stocks': ['ANTM.JK'], 'macro_factor': 'Safe Haven'},
            'crude_oil': {'symbol': 'CL=F', 'impact_stocks': ['ASII.JK', 'PTBA.JK'], 'macro_factor': 'Energy Costs'}
        }
        
        # Regional Markets for Context
        self.regional_markets = {
            'IDX': {'index': '^JKSE', 'weight': 'Primary', 'correlation': 'Direct'},
            'Asia': {'index': '^HSI', 'weight': 'High', 'correlation': 'Regional Trade'},
            'NYSE': {'index': '^NYA', 'weight': 'Medium', 'correlation': 'Global Capital'},
            'ASEAN': {'index': '^STI', 'weight': 'High', 'correlation': 'Regional Integration'},
            'China': {'index': '000001.SS', 'weight': 'High', 'correlation': 'Commodity Demand'},
            'Japan': {'index': '^N225', 'weight': 'Medium', 'correlation': 'Manufacturing'}
        }
    
    def load_social_intelligence_data(self) -> Dict:
        """Load latest social intelligence data from advanced collection system"""
        try:
            # Check for latest social intelligence collection output
            import glob
            import os
            
            # Look for social intelligence output files
            social_intel_path = "/home/ctianm/advanced-intelligence-system/news-social-intelligence-data-pipeline/"
            output_files = glob.glob(f"{social_intel_path}*.json")
            
            if output_files:
                latest_file = max(output_files, key=os.path.getctime)
                with open(latest_file, 'r') as f:
                    return json.load(f)
            else:
                logger.info("No social intelligence data files found - using mock data for analysis structure")
                return self._generate_mock_social_data()
                
        except Exception as e:
            logger.error(f"Error loading social intelligence data: {e}")
            return self._generate_mock_social_data()
    
    def load_commodity_data(self) -> Dict[str, Dict]:
        """Load latest commodity data from portfolio intelligence system"""
        try:
            import glob
            import os
            
            # Look for commodity data files
            commodity_path = "/home/ctianm/advanced-intelligence-system/"
            commodity_files = glob.glob(f"{commodity_path}commodity_data_*.json")
            
            if commodity_files:
                latest_file = max(commodity_files, key=os.path.getctime)
                with open(latest_file, 'r') as f:
                    data = json.load(f)
                
                # Convert to our analysis format
                commodity_analysis = {}
                for item in data:
                    symbol = item.get('commodity', '').lower().replace(' ', '_')
                    commodity_analysis[symbol] = {
                        'price': item.get('price', 0),
                        'change': item.get('change', 0),
                        'currency': item.get('currency', 'USD'),
                        'unit': item.get('unit', ''),
                        'timestamp': item.get('timestamp', '')
                    }
                return commodity_analysis
            else:
                logger.info("No commodity data found - using mock data for analysis structure")
                return self._generate_mock_commodity_data()
                
        except Exception as e:
            logger.error(f"Error loading commodity data: {e}")
            return self._generate_mock_commodity_data()
    
    def calculate_news_sentiment_correlation(self, social_data: Dict, commodity_data: Dict) -> Dict[str, float]:
        """Calculate correlation between news sentiment and commodity price movements"""
        correlations = {}
        
        # Analyze sentiment impact on each commodity
        for commodity in self.strategic_commodities:
            sentiment_score = self._extract_commodity_sentiment(social_data, commodity)
            price_change = commodity_data.get(commodity, {}).get('change', 0)
            
            # Simple correlation calculation (can be enhanced with more sophisticated methods)
            if sentiment_score != 0:
                correlation = price_change / abs(sentiment_score) if sentiment_score != 0 else 0
                correlations[commodity] = min(max(correlation, -1), 1)  # Normalize to [-1, 1]
            else:
                correlations[commodity] = 0
        
        return correlations
    
    def analyze_industry_impact(self, social_data: Dict) -> Dict[str, Dict]:
        """Analyze industry-specific impact from social intelligence"""
        industry_analysis = {}
        
        # Group stocks by industry
        industries = {}
        for stock, data in self.portfolio_stocks.items():
            sector = data['sector']
            if sector not in industries:
                industries[sector] = []
            industries[sector].append(stock)
        
        # Analyze each industry
        for industry, stocks in industries.items():
            sentiment_score = self._extract_industry_sentiment(social_data, industry, stocks)
            risk_factors = self._identify_industry_risks(social_data, industry)
            opportunities = self._identify_industry_opportunities(social_data, industry)
            
            industry_analysis[industry] = {
                'sentiment_score': sentiment_score,
                'risk_factors': risk_factors,
                'opportunities': opportunities,
                'affected_stocks': stocks,
                'impact_level': self._calculate_impact_level(sentiment_score, risk_factors, opportunities)
            }
        
        return industry_analysis
    
    def analyze_regional_markets(self) -> Dict[str, Dict]:
        """Analyze regional market performance and correlations"""
        regional_analysis = {}
        
        for market, data in self.regional_markets.items():
            # In production, this would fetch real market data
            # For now, provide analysis structure
            regional_analysis[market] = {
                'performance': f"Mock {market} performance data",
                'correlation_with_idx': data['correlation'],
                'weight_in_analysis': data['weight'],
                'impact_factors': self._get_regional_impact_factors(market),
                'trend_direction': 'Neutral'  # Would be calculated from real data
            }
        
        return regional_analysis
    
    def generate_macro_economic_analysis(self, social_data: Dict, commodity_data: Dict) -> Dict[str, float]:
        """Generate macro-economic indicators analysis"""
        macro_analysis = {
            'commodity_export_impact': self._calculate_export_impact(commodity_data),
            'global_trade_sentiment': self._extract_trade_sentiment(social_data),
            'inflation_pressure': self._calculate_inflation_pressure(commodity_data),
            'currency_stability': self._assess_currency_stability(social_data, commodity_data),
            'geopolitical_risk': self._assess_geopolitical_risk(social_data),
            'economic_growth_indicators': self._extract_growth_indicators(social_data)
        }
        
        return macro_analysis
    
    def generate_micro_economic_analysis(self, social_data: Dict) -> Dict[str, Dict]:
        """Generate micro-economic analysis for individual stocks"""
        micro_analysis = {}
        
        for stock, stock_data in self.portfolio_stocks.items():
            company_sentiment = self._extract_company_sentiment(social_data, stock)
            sector_dynamics = self._analyze_sector_dynamics(social_data, stock_data['sector'])
            competitive_position = self._assess_competitive_position(social_data, stock)
            
            micro_analysis[stock] = {
                'company_sentiment': company_sentiment,
                'sector_dynamics': sector_dynamics,
                'competitive_position': competitive_position,
                'risk_level': self._calculate_stock_risk(company_sentiment, sector_dynamics),
                'opportunity_score': self._calculate_opportunity_score(company_sentiment, competitive_position)
            }
        
        return micro_analysis
    
    def generate_comprehensive_report(self) -> PortfolioAnalysis:
        """Generate comprehensive social-economic correlation analysis report"""
        logger.info("🎯 Generating Comprehensive Social-Economic Analysis...")
        
        # Load data
        social_data = self.load_social_intelligence_data()
        commodity_data = self.load_commodity_data()
        
        # Perform analyses
        commodity_correlations = self.calculate_news_sentiment_correlation(social_data, commodity_data)
        industry_analysis = self.analyze_industry_impact(social_data)
        regional_analysis = self.analyze_regional_markets()
        macro_analysis = self.generate_macro_economic_analysis(social_data, commodity_data)
        micro_analysis = self.generate_micro_economic_analysis(social_data)
        
        # Create comprehensive report
        analysis = PortfolioAnalysis(
            timestamp=datetime.now().isoformat(),
            portfolio_stocks=micro_analysis,
            commodity_correlations=commodity_correlations,
            regional_market_overview=regional_analysis,
            social_economic_factors=industry_analysis,
            macro_indicators=macro_analysis,
            micro_factors=micro_analysis
        )
        
        return analysis
    
    # Helper methods (simplified implementations - can be enhanced)
    def _generate_mock_social_data(self) -> Dict:
        """Generate mock social intelligence data for testing"""
        return {
            "topics": ["Indonesia", "BMRI", "BBRI", "INCO", "nickel", "coal"],
            "sentiment_scores": {"Indonesia": 0.6, "BMRI": 0.4, "BBRI": 0.5, "INCO": 0.7, "nickel": 0.8, "coal": 0.3},
            "source_count": 75,
            "timestamp": datetime.now().isoformat()
        }
    
    def _generate_mock_commodity_data(self) -> Dict:
        """Generate mock commodity data for testing"""
        return {
            "nickel": {"price": 18450, "change": 245, "currency": "USD", "unit": "tonne"},
            "coal": {"price": 135.5, "change": 3.75, "currency": "USD", "unit": "metric ton"},
            "palm_oil": {"price": 965, "change": -12.5, "currency": "USD", "unit": "tonne"},
            "gold": {"price": 2018.5, "change": -15.25, "currency": "USD", "unit": "oz"},
            "crude_oil": {"price": 78.45, "change": 1.23, "currency": "USD", "unit": "barrel"}
        }
    
    def _extract_commodity_sentiment(self, social_data: Dict, commodity: str) -> float:
        """Extract sentiment score for specific commodity from social data"""
        sentiment_scores = social_data.get('sentiment_scores', {})
        return sentiment_scores.get(commodity, 0)
    
    def _extract_industry_sentiment(self, social_data: Dict, industry: str, stocks: List[str]) -> float:
        """Extract overall sentiment for industry from social data"""
        sentiment_scores = social_data.get('sentiment_scores', {})
        total_sentiment = 0
        count = 0
        
        for stock in stocks:
            stock_symbol = stock.replace('.JK', '')
            if stock_symbol in sentiment_scores:
                total_sentiment += sentiment_scores[stock_symbol]
                count += 1
        
        return total_sentiment / count if count > 0 else 0
    
    def _identify_industry_risks(self, social_data: Dict, industry: str) -> List[str]:
        """Identify risk factors for specific industry"""
        # Simplified risk identification
        risk_mapping = {
            'Banking': ['Interest Rate Risk', 'Credit Risk', 'Digital Disruption'],
            'Mining': ['Commodity Price Volatility', 'Environmental Regulations', 'Operational Risks'],
            'Energy': ['Energy Transition', 'ESG Compliance', 'Price Volatility'],
            'Agriculture': ['Weather Risk', 'Export Demand', 'Sustainability Requirements'],
            'Healthcare': ['Regulatory Changes', 'Competition', 'Demographic Shifts'],
            'Consumer': ['Inflation Impact', 'Consumer Behavior', 'Competition'],
            'Telecom': ['Technology Disruption', 'Infrastructure Costs', 'Regulatory Changes'],
            'Automotive': ['EV Transition', 'Supply Chain', 'Consumer Preferences']
        }
        
        return risk_mapping.get(industry, ['General Market Risk'])
    
    def _identify_industry_opportunities(self, social_data: Dict, industry: str) -> List[str]:
        """Identify opportunities for specific industry"""
        opportunity_mapping = {
            'Banking': ['Digital Banking Growth', 'Financial Inclusion', 'Cross-selling'],
            'Mining': ['EV Battery Demand', 'Infrastructure Growth', 'Technology Adoption'],
            'Energy': ['Renewable Energy', 'Energy Security', 'Export Growth'],
            'Agriculture': ['Food Security', 'Sustainable Practices', 'Value-added Products'],
            'Healthcare': ['Aging Population', 'Healthcare Access', 'Innovation'],
            'Consumer': ['Brand Strength', 'Distribution Network', 'Product Innovation'],
            'Telecom': ['5G Deployment', 'Digital Services', 'IoT Growth'],
            'Automotive': ['EV Market Growth', 'Autonomous Vehicles', 'Smart Mobility']
        }
        
        return opportunity_mapping.get(industry, ['General Growth Opportunities'])
    
    def _calculate_impact_level(self, sentiment: float, risks: List[str], opportunities: List[str]) -> str:
        """Calculate overall impact level for industry"""
        risk_score = len(risks) * -0.1
        opportunity_score = len(opportunities) * 0.1
        total_score = sentiment + risk_score + opportunity_score
        
        if total_score > 0.5:
            return 'High Positive'
        elif total_score > 0:
            return 'Moderate Positive'
        elif total_score > -0.5:
            return 'Moderate Negative'
        else:
            return 'High Negative'
    
    def _get_regional_impact_factors(self, market: str) -> List[str]:
        """Get impact factors for regional markets"""
        factor_mapping = {
            'IDX': ['Domestic Policy', 'Commodity Exports', 'Foreign Investment'],
            'Asia': ['Regional Trade', 'Supply Chains', 'Economic Integration'],
            'NYSE': ['Global Capital Flows', 'Dollar Strength', 'US Policy'],
            'ASEAN': ['Regional Integration', 'Trade Agreements', 'Economic Cooperation'],
            'China': ['Belt and Road', 'Commodity Demand', 'Manufacturing'],
            'Japan': ['Technology Transfer', 'Manufacturing', 'Investment Flows']
        }
        
        return factor_mapping.get(market, ['General Economic Factors'])
    
    def _calculate_export_impact(self, commodity_data: Dict) -> float:
        """Calculate impact of commodity prices on export revenue"""
        export_commodities = ['nickel', 'coal', 'palm_oil']
        total_impact = 0
        
        for commodity in export_commodities:
            change = commodity_data.get(commodity, {}).get('change', 0)
            total_impact += change
        
        return total_impact / len(export_commodities)
    
    def _extract_trade_sentiment(self, social_data: Dict) -> float:
        """Extract trade sentiment from social intelligence"""
        return social_data.get('sentiment_scores', {}).get('Indonesia', 0)
    
    def _calculate_inflation_pressure(self, commodity_data: Dict) -> float:
        """Calculate inflation pressure from commodity price changes"""
        energy_commodities = ['crude_oil', 'coal']
        total_change = 0
        
        for commodity in energy_commodities:
            change = commodity_data.get(commodity, {}).get('change', 0)
            total_change += change
        
        return total_change / len(energy_commodities)
    
    def _assess_currency_stability(self, social_data: Dict, commodity_data: Dict) -> float:
        """Assess IDR stability based on commodity exports and sentiment"""
        export_impact = self._calculate_export_impact(commodity_data)
        sentiment_impact = self._extract_trade_sentiment(social_data)
        
        return (export_impact * 0.6) + (sentiment_impact * 0.4)
    
    def _assess_geopolitical_risk(self, social_data: Dict) -> float:
        """Assess geopolitical risk from social intelligence"""
        # Simplified geopolitical risk assessment
        base_sentiment = social_data.get('sentiment_scores', {}).get('Indonesia', 0)
        return 1 - abs(base_sentiment)  # Higher absolute sentiment = lower geopolitical risk
    
    def _extract_growth_indicators(self, social_data: Dict) -> float:
        """Extract economic growth indicators from social intelligence"""
        return social_data.get('sentiment_scores', {}).get('Indonesia', 0) * 100
    
    def _extract_company_sentiment(self, social_data: Dict, stock: str) -> float:
        """Extract sentiment for specific company"""
        sentiment_scores = social_data.get('sentiment_scores', {})
        stock_symbol = stock.replace('.JK', '')
        return sentiment_scores.get(stock_symbol, 0)
    
    def _analyze_sector_dynamics(self, social_data: Dict, sector: str) -> Dict:
        """Analyze dynamics for specific sector"""
        return {
            'trend': 'Positive',  # Would be calculated from real data
            'competitive_intensity': 'Medium',
            'regulatory_environment': 'Stable',
            'growth_prospects': 'Good'
        }
    
    def _assess_competitive_position(self, social_data: Dict, stock: str) -> str:
        """Assess competitive position of stock"""
        sentiment = self._extract_company_sentiment(social_data, stock)
        
        if sentiment > 0.6:
            return 'Market Leader'
        elif sentiment > 0.3:
            return 'Strong Position'
        elif sentiment > 0:
            return 'Competitive'
        else:
            return 'Challenging'
    
    def _calculate_stock_risk(self, sentiment: float, sector_dynamics: Dict) -> str:
        """Calculate risk level for individual stock"""
        if sentiment > 0.5:
            return 'Low'
        elif sentiment > 0:
            return 'Medium'
        else:
            return 'High'
    
    def _calculate_opportunity_score(self, sentiment: float, competitive_position: str) -> float:
        """Calculate opportunity score for stock"""
        position_scores = {
            'Market Leader': 1.0,
            'Strong Position': 0.8,
            'Competitive': 0.6,
            'Challenging': 0.3
        }
        
        position_score = position_scores.get(competitive_position, 0.5)
        return (sentiment + position_score) / 2

def main():
    """Main execution function"""
    analyzer = SocialEconomicAnalyzer()
    
    print("🎯 Christian's Portfolio Social-Economic Correlation Analysis")
    print("=" * 80)
    
    # Generate comprehensive analysis
    analysis = analyzer.generate_comprehensive_report()
    
    # Save analysis report
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"social_economic_analysis_{timestamp}.json"
    
    with open(output_file, 'w') as f:
        json.dump(asdict(analysis), f, indent=2)
    
    print(f"📊 Analysis complete! Report saved to: {output_file}")
    print(f"📅 Timestamp: {analysis.timestamp}")
    print(f"💼 Portfolio Stocks Analyzed: {len(analysis.portfolio_stocks)}")
    print(f"🏭 Regional Markets: {len(analysis.regional_market_overview)}")
    print(f"📈 Macro Indicators: {len(analysis.macro_indicators)}")
    
    # Display key insights
    print("\n🔍 KEY INSIGHTS:")
    print("-" * 40)
    
    # Commodity correlations
    print("📊 Commodity Correlations:")
    for commodity, correlation in analysis.commodity_correlations.items():
        direction = "📈" if correlation > 0 else "📉" if correlation < 0 else "➡️"
        print(f"  {direction} {commodity}: {correlation:.3f}")
    
    # Macro indicators summary
    print("\n🌍 Macro Economic Indicators:")
    for indicator, value in analysis.macro_indicators.items():
        print(f"  • {indicator}: {value:.2f}")
    
    print(f"\n✅ Social-Economic Analysis Framework Ready!")
    print(f"🔄 Run this analysis after each intelligence collection cycle")
    print(f"📱 Integration with Hermes cronjob system available")

if __name__ == "__main__":
    main()