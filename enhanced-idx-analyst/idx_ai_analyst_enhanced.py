#!/usr/bin/env python3
"""
Enhanced IDX AI Analyst with Debate Mechanism
Combines 5-persona system with TradingAgents concepts:
- Data gathering (analyst team)
- Debate mechanism (bull/bear researchers)
- Trade execution (trader agent)
- Risk management (portfolio constraints)
- Decision memory (logging + reflection)

Usage:
    python idx_ai_analyst_enhanced.py --all
    python idx_ai_analyst_enhanced.py BMRI KLBF
"""

import sys
import os
import json
from datetime import datetime
from typing import Dict, List, Any, Optional
from pathlib import Path

# Add modules to path
sys.path.insert(0, str(Path(__file__).parent))

from config import (
    PORTFOLIO_STOCKS, WATCHLIST_STOCKS, ALL_STOCKS,
    CRITERIA, PERSONAS, DEBATE_CONFIG, RISK_CONFIG, MEMORY_CONFIG, EXECUTION_CONFIG
)
from modules.data_gathering import gather_analyst_reports
from modules.debate_engine import debate_stock
from modules.trader_executor import generate_trader_proposal
from modules.risk_manager import assess_risk
from modules.memory_logger import MemoryLogger


class EnhancedIDXAnalyst:
    """Main orchestrator for enhanced analysis"""
    
    def __init__(self, debug: bool = False):
        self.debug = debug
        self.memory_logger = MemoryLogger(MEMORY_CONFIG.get("memory_dir"))
        self.results = {}
    
    def analyze_stock(self, ticker: str, stock_data: Dict[str, Any]) -> Dict[str, Any]:
        """Run complete analysis pipeline for one stock"""
        
        if self.debug:
            print(f"📊 Analyzing {ticker}...", file=sys.stderr)
        
        # Step 1: Gather analyst reports
        analyst_reports = gather_analyst_reports(ticker, stock_data)
        
        # Step 2: Run debate
        debate_result = debate_stock(
            ticker,
            analyst_reports,
            max_rounds=DEBATE_CONFIG.get("max_rounds", 2)
        )
        
        # Step 3: Generate trader proposal
        trader_proposal = generate_trader_proposal(
            ticker,
            debate_result["final_signal"],
            stock_data,
            analyst_reports,
            EXECUTION_CONFIG
        )
        
        # Step 4: Risk assessment
        portfolio_value = stock_data.get("portfolio_value", 100_000_000)  # Default Rp 100M
        risk_assessment = assess_risk(
            ticker,
            trader_proposal.position_size_pct,
            stock_data,
            portfolio_value,
            RISK_CONFIG
        )
        
        # Step 5: Log decision
        self.memory_logger.log_decision(
            ticker=ticker,
            trade_date=datetime.now().strftime("%Y-%m-%d"),
            signal=debate_result["final_signal"],
            confidence=debate_result["confidence"],
            entry_price=trader_proposal.entry_price,
            stop_loss=trader_proposal.stop_loss,
            take_profit=trader_proposal.take_profit,
            position_size_pct=trader_proposal.position_size_pct,
            debate_summary=debate_result["debate_transcript"][:500],  # First 500 chars
            trader_reasoning=trader_proposal.reasoning
        )
        
        return {
            "ticker": ticker,
            "analyst_reports": analyst_reports,
            "debate_result": debate_result,
            "trader_proposal": trader_proposal,
            "risk_assessment": risk_assessment,
        }
    
    def format_output(self, analysis_result: Dict[str, Any]) -> str:
        """Format analysis for Telegram delivery"""
        
        ticker = analysis_result["ticker"]
        debate = analysis_result["debate_result"]
        proposal = analysis_result["trader_proposal"]
        risk = analysis_result["risk_assessment"]
        
        output = f"""
📊 **{ticker} — Enhanced Analysis**
**Signal:** {debate['final_signal']} | **Confidence:** {debate['confidence']}
⏰ {datetime.now().strftime('%H:%M WIB')}

**Debate Summary (2 Rounds):**
{debate['debate_transcript'][:400]}...

**Trader Proposal:**
Action: {proposal.action.value}
{f'Entry: Rp {proposal.entry_price:,.0f}' if proposal.entry_price else 'N/A'}
{f'Stop: Rp {proposal.stop_loss:,.0f}' if proposal.stop_loss else 'N/A'}
{f'Size: {proposal.position_size_pct*100:.1f}%' if proposal.position_size_pct else 'N/A'}

**Risk Status:** {'✅ Approved' if risk.is_approved else '⚠️ Flagged'}
Risk Score: {risk.risk_score:.2f}/1.0

---
"""
        return output.strip()
    
    def run_analysis(self, tickers: List[str], mock_data: bool = False) -> str:
        """Run analysis for multiple tickers"""
        
        time_wib = datetime.now().strftime("%H:%M WIB")
        output = f"📊 **Enhanced IDX Analyst** | {time_wib}\n"
        output += "=" * 60 + "\n\n"
        
        for ticker in tickers:
            # Mock stock data (in production, fetch from Notion)
            if mock_data:
                stock_data = {
                    "ticker": ticker,
                    "current_price": 10000 + len(ticker) * 100,  # Mock price
                    "per": 12 + (ord(ticker[0]) % 3),
                    "pbv": 1.5 + (ord(ticker[1]) % 2) * 0.2,
                    "roe": 12 + (ord(ticker[2]) % 5),
                    "der": 0.8 + (ord(ticker[3]) % 2) * 0.1,
                    "dividend_yield": 3.5 + (len(ticker) % 2) * 0.5,
                    "sentiment_score": 0.3,
                    "portfolio_value": 279_100_000,  # Christian's portfolio
                }
            else:
                # Would fetch from Notion in production
                stock_data = self._fetch_stock_data(ticker)
            
            try:
                result = self.analyze_stock(ticker, stock_data)
                self.results[ticker] = result
                output += self.format_output(result) + "\n"
            except Exception as e:
                print(f"❌ Error analyzing {ticker}: {e}", file=sys.stderr)
                output += f"❌ Error analyzing {ticker}\n"
        
        # Add memory summary
        output += "\n" + self.memory_logger.format_memory_summary()
        
        return output
    
    def _fetch_stock_data(self, ticker: str) -> Dict[str, Any]:
        """Fetch stock data from Notion (placeholder)"""
        # In production, integrate with existing portfolio scraper
        return {
            "ticker": ticker,
            "current_price": 0,
            "per": 0,
            "pbv": 0,
            "roe": 0,
            "der": 0,
            "dividend_yield": 0,
            "sentiment_score": 0,
            "portfolio_value": 279_100_000,
        }


def main():
    """Entry point"""
    
    # Parse arguments
    debug = "--debug" in sys.argv
    mock_mode = "--mock" in sys.argv or "--test" in sys.argv
    
    # Determine tickers
    if "--all" in sys.argv:
        tickers = ALL_STOCKS
    elif "--portfolio" in sys.argv:
        tickers = PORTFOLIO_STOCKS
    elif "--watchlist" in sys.argv:
        tickers = WATCHLIST_STOCKS
    else:
        # Get from command line
        tickers = [arg for arg in sys.argv[1:] if not arg.startswith("--")]
        tickers = tickers or ["BMRI", "KLBF"]  # Default
    
    # Run analysis
    analyst = EnhancedIDXAnalyst(debug=debug)
    output = analyst.run_analysis(tickers, mock_data=mock_mode)
    
    # Print output
    print(output)
    
    # Optionally save to file
    if "--save" in sys.argv:
        output_dir = Path("~/.hermes/profiles/pagupon-finance/results").expanduser()
        output_dir.mkdir(parents=True, exist_ok=True)
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_file = output_dir / f"analysis_{timestamp}.txt"
        
        with open(output_file, "w") as f:
            f.write(output)
        
        print(f"\n✅ Analysis saved to {output_file}", file=sys.stderr)


if __name__ == "__main__":
    main()
