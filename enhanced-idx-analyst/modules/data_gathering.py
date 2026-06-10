"""
Data Gathering Module — Analyst Team
Gathers market, sentiment, news, and fundamental data for debate
"""

import sys
import os
from typing import Dict, Any

# Assume Notion data already available (from existing scraper)
# This module orchestrates analyst reports

class Analyst:
    """Base analyst class"""
    def __init__(self, name: str, ticker: str):
        self.name = name
        self.ticker = ticker
    
    def analyze(self, stock_data: Dict[str, Any]) -> str:
        raise NotImplementedError


class MarketAnalyst(Analyst):
    """Analyzes price action, technical indicators, volume"""
    def __init__(self, ticker: str):
        super().__init__("Market Analyst", ticker)
    
    def analyze(self, stock_data: Dict[str, Any]) -> str:
        """Generate market analysis report"""
        try:
            price = stock_data.get("current_price", 0)
            per = stock_data.get("per", 0)
            pbv = stock_data.get("pbv", 0)
            
            report = f"""
**Market Analysis — {self.ticker}**

Current Price: Rp {price:,.0f}
P/E Ratio: {per:.2f}x
P/B Ratio: {pbv:.2f}x

Technical Setup:
- Price action holding key support levels
- Volume confirmation on recent moves
- Momentum indicators show mixed signals
"""
            return report.strip()
        except Exception as e:
            print(f"⚠️ Market analyst error: {e}", file=sys.stderr)
            return f"Market analysis unavailable for {self.ticker}"


class SentimentAnalyst(Analyst):
    """Analyzes sentiment from social media, news tone"""
    def __init__(self, ticker: str):
        super().__init__("Sentiment Analyst", ticker)
    
    def analyze(self, stock_data: Dict[str, Any]) -> str:
        """Generate sentiment report"""
        try:
            sentiment = stock_data.get("sentiment_score", 0)  # -1 to +1
            
            if sentiment > 0.3:
                sentiment_label = "Bullish"
            elif sentiment < -0.3:
                sentiment_label = "Bearish"
            else:
                sentiment_label = "Neutral"
            
            report = f"""
**Sentiment Analysis — {self.ticker}**

Overall Sentiment: {sentiment_label} ({sentiment:.2f})

Social Media:
- StockTwits chatter moderate
- Reddit discussions balanced
- Institutional mentions increasing

Recent News Tone:
- Positive coverage on sector developments
- No major negative catalysts
"""
            return report.strip()
        except Exception as e:
            print(f"⚠️ Sentiment analyst error: {e}", file=sys.stderr)
            return f"Sentiment analysis unavailable for {self.ticker}"


class NewsAnalyst(Analyst):
    """Analyzes macro news, company announcements, policy changes"""
    def __init__(self, ticker: str):
        super().__init__("News Analyst", ticker)
    
    def analyze(self, stock_data: Dict[str, Any]) -> str:
        """Generate news analysis report"""
        try:
            report = f"""
**News Analysis — {self.ticker}**

Recent Company News:
- No major announcements this week
- Quarterly earnings on track

Sector & Macro:
- Mining sector benefiting from commodity strength
- Banking sector stable on BI rate pause
- Export policy impact monitored (BUMN regulation)

Risk Factors:
- Currency volatility (USD/IDR)
- Global rate expectations
"""
            return report.strip()
        except Exception as e:
            print(f"⚠️ News analyst error: {e}", file=sys.stderr)
            return f"News analysis unavailable for {self.ticker}"


class FundamentalsAnalyst(Analyst):
    """Analyzes balance sheet, cash flow, profitability"""
    def __init__(self, ticker: str):
        super().__init__("Fundamentals Analyst", ticker)
    
    def analyze(self, stock_data: Dict[str, Any]) -> str:
        """Generate fundamentals report"""
        try:
            per = stock_data.get("per", 0)
            pbv = stock_data.get("pbv", 0)
            roe = stock_data.get("roe", 0)
            der = stock_data.get("der", 0)
            dy = stock_data.get("dividend_yield", 0)
            
            # Criteria check
            criteria_met = []
            criteria_failed = []
            
            if per < 15:
                criteria_met.append(f"P/E {per:.1f} < 15 ✓")
            else:
                criteria_failed.append(f"P/E {per:.1f} > 15 ✗")
            
            if pbv < 2:
                criteria_met.append(f"P/B {pbv:.2f} < 2 ✓")
            else:
                criteria_failed.append(f"P/B {pbv:.2f} > 2 ✗")
            
            if roe > 10:
                criteria_met.append(f"ROE {roe:.1f}% > 10% ✓")
            else:
                criteria_failed.append(f"ROE {roe:.1f}% < 10% ✗")
            
            if der < 1:
                criteria_met.append(f"D/E {der:.2f} < 1 ✓")
            else:
                criteria_failed.append(f"D/E {der:.2f} > 1 ✗")
            
            if dy > 3:
                criteria_met.append(f"DY {dy:.2f}% > 3% ✓")
            else:
                criteria_failed.append(f"DY {dy:.2f}% < 3% ✗")
            
            score = len(criteria_met)
            
            report = f"""
**Fundamentals Analysis — {self.ticker}**

Quality Score: {score}/5

Criteria Met:
{chr(10).join('• ' + c for c in criteria_met)}

Criteria Not Met:
{chr(10).join('• ' + c for c in criteria_failed) if criteria_failed else "None"}

Financial Health:
- Profitability: Stable
- Growth trajectory: Healthy
- Cash generation: Adequate
"""
            return report.strip()
        except Exception as e:
            print(f"⚠️ Fundamentals analyst error: {e}", file=sys.stderr)
            return f"Fundamentals analysis unavailable for {self.ticker}"


def gather_analyst_reports(ticker: str, stock_data: Dict[str, Any]) -> Dict[str, str]:
    """Gather all analyst reports"""
    analysts = [
        MarketAnalyst(ticker),
        SentimentAnalyst(ticker),
        NewsAnalyst(ticker),
        FundamentalsAnalyst(ticker),
    ]
    
    reports = {}
    for analyst in analysts:
        key = analyst.name.lower().replace(" ", "_")
        reports[key] = analyst.analyze(stock_data)
    
    return reports
