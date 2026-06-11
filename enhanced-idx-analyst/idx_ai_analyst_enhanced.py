#!/usr/bin/env python3
"""
Enhanced IDX AI Analyst with Debate Mechanism + RTI Format + IDX Scraper + Dividend Sync
Combines 5-persona system with TradingAgents concepts:
- Data gathering (analyst team)
- Debate mechanism (5-persona bull/bear researchers with detailed reasoning)
- Trade execution (trader agent)
- Risk management (portfolio constraints)
- Decision memory (logging + reflection)
- IDX scraper (curl_cffi bypass Cloudflare)
- RTI Business output formatting
- Notion dividend sync (historical records)

Usage:
    python idx_ai_analyst_enhanced.py --portfolio
    python idx_ai_analyst_enhanced.py --all --debug
    python idx_ai_analyst_enhanced.py BMRI KLBF --mock
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
    CRITERIA, PERSONAS, DEBATE_CONFIG, RISK_CONFIG, MEMORY_CONFIG, EXECUTION_CONFIG, NOTION_CONFIG
)
from modules.data_gathering import gather_analyst_reports
from modules.debate_engine import debate_stock
from modules.trader_executor import generate_trader_proposal
from modules.risk_manager import assess_risk
from modules.memory_logger import MemoryLogger
from modules.notion_integration import NotionPortfolioFetcher, get_stock_data_from_notion
from modules.idx_scraper import IDXDataScraper, scrape_idx_data
from modules.output_formatter import RTIBusinessFormatter, format_multiple_stocks
from modules.dividend_sync import DividendSyncer, DividendRecorder


class EnhancedIDXAnalyst:
    """Main orchestrator for enhanced analysis with all enhancements"""
    
    def __init__(self, debug: bool = False):
        self.debug = debug
        self.memory_logger = MemoryLogger(MEMORY_CONFIG.get("memory_dir"))
        self.results = {}
        self.idx_scraper = IDXDataScraper()
        self.formatter = RTIBusinessFormatter()
        self.dividend_syncer = DividendSyncer(
            portfolio_db_id=NOTION_CONFIG.get("portfolio_db_id")
        )
    
    def analyze_stock(self, ticker: str, stock_data: Dict[str, Any]) -> Dict[str, Any]:
        """Run complete analysis pipeline for one stock with all enhancements"""
        
        if self.debug:
            print(f"📊 Analyzing {ticker}...", file=sys.stderr)
        
        # Step 0: Fetch complete IDX profile (curl_cffi + yfinance)
        idx_profile = self.idx_scraper.fetch_complete_profile(ticker)
        if self.debug:
            print(f"   IDX Profile: {len(idx_profile)} fields", file=sys.stderr)
        
        # Merge IDX profile into stock_data
        stock_data_enriched = {**stock_data, **idx_profile}
        
        # Step 1: Gather analyst reports (old approach, but still useful)
        analyst_reports = gather_analyst_reports(ticker, stock_data_enriched)
        
        # Step 2: Run debate with NEW 5-persona system (passing metrics dict)
        # Extract metrics for debate engine
        metrics = {
            "current_price": stock_data_enriched.get("current_price", 0),
            "per": stock_data_enriched.get("per", 0),
            "pbv": stock_data_enriched.get("pbv", 0),
            "roe": stock_data_enriched.get("roe", 0),
            "roa": stock_data_enriched.get("roa", 0),
            "npm": stock_data_enriched.get("npm", 0),
            "dy": stock_data_enriched.get("dy", 0),
            "der": stock_data_enriched.get("der", 0),
        }
        
        debate_result = debate_stock(
            ticker,
            metrics,
            max_rounds=DEBATE_CONFIG.get("max_rounds", 2)
        )
        
        # Step 3: Generate trader proposal
        trader_proposal = generate_trader_proposal(
            ticker,
            debate_result["final_signal"],
            stock_data_enriched,
            analyst_reports,
            EXECUTION_CONFIG
        )
        
        # Step 4: Risk assessment
        portfolio_value = stock_data_enriched.get("portfolio_value", 279_100_000)
        risk_assessment = assess_risk(
            ticker,
            trader_proposal.position_size_pct,
            stock_data_enriched,
            portfolio_value,
            RISK_CONFIG
        )
        
        # Step 5: Log decision with full debate transcript
        self.memory_logger.log_decision(
            ticker=ticker,
            trade_date=datetime.now().strftime("%Y-%m-%d"),
            signal=debate_result["final_signal"],
            confidence=debate_result["confidence"],
            entry_price=trader_proposal.entry_price,
            stop_loss=trader_proposal.stop_loss,
            take_profit=trader_proposal.take_profit,
            position_size_pct=trader_proposal.position_size_pct,
            debate_summary=debate_result["consensus_summary"],  # Use consensus instead of transcript
            trader_reasoning=trader_proposal.reasoning
        )
        
        return {
            "ticker": ticker,
            "stock_data": stock_data_enriched,
            "idx_profile": idx_profile,
            "analyst_reports": analyst_reports,
            "debate_result": debate_result,
            "trader_proposal": trader_proposal,
            "risk_assessment": risk_assessment,
        }
    
    def format_output_rti(self, analysis_result: Dict[str, Any]) -> str:
        """Format analysis using RTI Business style"""
        
        ticker = analysis_result["ticker"]
        stock_data = analysis_result["stock_data"]
        debate_result = analysis_result["debate_result"]
        trader_proposal = analysis_result["trader_proposal"]
        risk_assessment = analysis_result["risk_assessment"]
        idx_profile = analysis_result.get("idx_profile", {})
        
        # Use RTI formatter
        rti_output = self.formatter.format_analysis(
            ticker=ticker,
            stock_data=stock_data,
            debate_result=debate_result,
            trader_proposal=trader_proposal,
            risk_assessment=risk_assessment,
            idx_profile=idx_profile
        )
        
        return str(rti_output)
    
    def format_output_telegram_compact(self, analysis_result: Dict[str, Any]) -> str:
        """Format analysis for quick Telegram scanning"""
        
        ticker = analysis_result["ticker"]
        stock_data = analysis_result["stock_data"]
        debate_result = analysis_result["debate_result"]
        
        signal = debate_result["final_signal"]
        price = stock_data.get("current_price", 0)
        per = stock_data.get("per", 0)
        dy = stock_data.get("dy", 0)
        roe = stock_data.get("roe", 0)
        der = stock_data.get("der", 0)
        
        return self.formatter.format_telegram_compact(ticker, signal, price, per, dy, roe, der)
    
    def run_analysis(
        self,
        tickers: List[str],
        mock_data: bool = False,
        use_notion: bool = True,
        format_style: str = "compact"  # "compact" or "full"
    ) -> str:
        """Run analysis for multiple tickers with all enhancements"""
        
        time_wib = datetime.now().strftime("%H:%M WIB")
        output = f"📊 **IDX AI ANALYST — ENHANCED DEBATE SYSTEM** | {time_wib}\n"
        output += "=" * 60 + "\n\n"
        
        # Try to initialize Notion fetcher if not using mock
        notion_fetcher = None
        if not mock_data and use_notion:
            try:
                notion_fetcher = NotionPortfolioFetcher()
                print("✅ Notion connected", file=sys.stderr)
            except Exception as e:
                print(f"⚠️ Notion unavailable: {e}. Using mock data.", file=sys.stderr)
                mock_data = True
        
        # Collect all results for dividend sync
        historical_dividends_map = {}
        
        for ticker in tickers:
            try:
                # Fetch stock data (prioritize Notion, fallback to mock)
                if mock_data:
                    stock_data = {
                        "ticker": ticker,
                        "current_price": 10000 + len(ticker) * 100,
                        "per": 12 + (ord(ticker[0]) % 3),
                        "pbv": 1.5 + (ord(ticker[1]) % 2) * 0.2,
                        "roe": 12 + (ord(ticker[2]) % 5),
                        "roa": 5 + (ord(ticker[0]) % 3),
                        "npm": 10 + (ord(ticker[1]) % 5),
                        "der": 0.8 + (ord(ticker[3]) % 2) * 0.1,
                        "dy": 3.5 + (len(ticker) % 2) * 0.5,
                        "sentiment_score": 0.3,
                        "portfolio_value": 279_100_000,
                    }
                elif notion_fetcher:
                    stock_data = notion_fetcher.fetch_stock_data(ticker)
                else:
                    stock_data = self._fetch_stock_data(ticker)
                
                result = self.analyze_stock(ticker, stock_data)
                self.results[ticker] = result
                
                # Collect historical dividends for later sync
                idx_profile = result.get("idx_profile", {})
                dividends = idx_profile.get("historical_dividends", [])
                if dividends:
                    historical_dividends_map[ticker] = dividends
                
                # Format output: compact for Telegram, save full debate separately
                if format_style == "full":
                    output += self.format_output_rti(result) + "\n\n"
                    # Also save full debate to separate file
                    self._save_full_debate(ticker, result)
                else:
                    output += self.format_output_telegram_compact(result) + "\n\n"
                    # Still save full debate for reference
                    self._save_full_debate(ticker, result)
                
            except Exception as e:
                print(f"❌ Error analyzing {ticker}: {e}", file=sys.stderr)
                output += f"❌ Error analyzing {ticker}: {str(e)}\n\n"
        
        # Step 6: Sync dividends to Notion (NEW enhancement #4)
        if not mock_data and historical_dividends_map:
            print(f"\n📊 Syncing {len(historical_dividends_map)} stocks' dividend history to Notion...", file=sys.stderr)
            try:
                sync_results = self.dividend_syncer.sync_all_portfolio(historical_dividends_map)
                synced_count = sum(1 for v in sync_results.values() if v is True)
                print(f"✅ Synced {synced_count} stocks", file=sys.stderr)
            except Exception as e:
                print(f"⚠️ Dividend sync partial: {e}", file=sys.stderr)
        
        # Add memory summary
        output += "\n" + self.memory_logger.format_memory_summary()
        
        return output
    
    def _save_full_debate(self, ticker: str, result: Dict[str, Any]) -> None:
        """Save full debate transcript to debate results file"""
        try:
            debate_result = result.get("debate_result", {})
            if not debate_result.get("debate_rounds"):
                return
            
            # Build full debate text
            debate_text = f"🎓 FULL DEBATE TRANSCRIPT — {ticker}\n"
            debate_text += "=" * 80 + "\n"
            debate_text += f"Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S WIB')}\n"
            debate_text += f"Signal: {debate_result.get('final_signal', 'HOLD')} (Confidence: {debate_result.get('confidence', 'MEDIUM')})\n"
            debate_text += f"Bull win rate: {debate_result.get('bull_win_rate', 0.5):.0%}\n\n"
            
            # Full debate rounds
            for r in debate_result.get("debate_rounds", []):
                debate_text += f"\n{'='*80}\n"
                debate_text += f"ROUND {r.round_num}\n"
                debate_text += f"{'='*80}\n\n"
                
                persona_emoji_map = {
                    "buffett": "🦉", "graham": "📚", "lynch": "🎯", 
                    "munger": "🧠", "guru_id": "🇮🇩"
                }
                
                bull_emoji = persona_emoji_map.get(r.bull_persona, "🦉")
                bear_emoji = persona_emoji_map.get(r.bear_persona, "📚")
                
                debate_text += f"{bull_emoji} **{r.bull_persona.upper()} (BULL)**\n"
                debate_text += f"Confidence: {r.bull_confidence}\n"
                debate_text += f"{r.bull_argument}\n\n"
                
                debate_text += f"{bear_emoji} **{r.bear_persona.upper()} (BEAR)**\n"
                debate_text += f"Confidence: {r.bear_confidence}\n"
                debate_text += f"{r.bear_argument}\n"
            
            debate_text += f"\n{'='*80}\n"
            debate_text += f"CONSENSUS\n"
            debate_text += f"{'='*80}\n"
            debate_text += debate_result.get("consensus_summary", "Debate concluded") + "\n"
            
            # Save to file
            debate_dir = Path("~/.hermes/profiles/pagupon-finance/debates").expanduser()
            debate_dir.mkdir(parents=True, exist_ok=True)
            
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            debate_file = debate_dir / f"debate_{ticker}_{timestamp}.txt"
            
            with open(debate_file, "w") as f:
                f.write(debate_text)
            
            if self.debug:
                print(f"   Debate saved: {debate_file}", file=sys.stderr)
        
        except Exception as e:
            if self.debug:
                print(f"   ⚠️ Debate save failed for {ticker}: {e}", file=sys.stderr)
    
    def _fetch_stock_data(self, ticker: str) -> Dict[str, Any]:
        """Fetch stock data from yfinance (fallback)"""
        try:
            import yfinance as yf
            stock = yf.Ticker(f"{ticker}.JK")
            info = stock.info
            
            return {
                "ticker": ticker,
                "current_price": info.get("currentPrice", 0),
                "per": info.get("trailingPE", 0),
                "pbv": info.get("priceToBook", 0),
                "roe": info.get("returnOnEquity", 0) * 100,
                "roa": info.get("returnOnAssets", 0) * 100,
                "npm": info.get("profitMargin", 0) * 100,
                "der": info.get("debtToEquity", 0),
                "dy": info.get("dividendYield", 0) * 100,
                "sentiment_score": 0,
                "portfolio_value": 279_100_000,
            }
        except Exception as e:
            print(f"⚠️ yfinance fetch failed for {ticker}: {e}", file=sys.stderr)
            return {
                "ticker": ticker,
                "current_price": 0,
                "per": 0,
                "pbv": 0,
                "roe": 0,
                "roa": 0,
                "npm": 0,
                "der": 0,
                "dy": 0,
                "sentiment_score": 0,
                "portfolio_value": 279_100_000,
            }


def main():
    """Entry point"""
    
    # Parse arguments
    debug = "--debug" in sys.argv
    mock_mode = "--mock" in sys.argv or "--test" in sys.argv
    format_style = "full" if "--full" in sys.argv else "compact"
    
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
    output = analyst.run_analysis(tickers, mock_data=mock_mode, format_style=format_style)
    
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
