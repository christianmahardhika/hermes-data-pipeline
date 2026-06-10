"""
Notion Integration Module — Fetch stock data from Notion Portfolio DB
Replaces mock data with real data from Christian's portfolio tracking
"""

import os
import sys
from typing import Dict, Any, Optional
from datetime import datetime

try:
    from notion_client import Client
except ImportError:
    print("❌ notion-client not installed. Run: pip install notion-client", file=sys.stderr)
    sys.exit(1)


class NotionPortfolioFetcher:
    """Fetch stock data from Notion Portfolio database"""
    
    def __init__(self, notion_api_key: Optional[str] = None, db_id: Optional[str] = None):
        """Initialize Notion client"""
        self.api_key = notion_api_key or os.environ.get("NOTION_API_KEY")
        self.db_id = db_id or "362cd5f2-e4f1-80a3-86fa-000bfbb0fa2d"  # Christian's Portfolio DB
        
        if not self.api_key:
            raise ValueError("NOTION_API_KEY not set in environment")
        
        self.client = Client(auth=self.api_key)
    
    def fetch_stock_data(self, ticker: str) -> Dict[str, Any]:
        """Fetch stock data for ticker from Notion"""
        try:
            # Try modern API first (notion-client >= 2.1.0)
            try:
                response = self.client.databases.query(
                    database_id=self.db_id,
                    filter={
                        "property": "Ticker",
                        "rich_text": {
                            "equals": ticker
                        }
                    }
                )
            except AttributeError:
                # Fallback to older API or direct page search
                print(f"⚠️ Using fallback Notion API for {ticker}", file=sys.stderr)
                # For now, return empty data to continue
                return self._empty_stock_data(ticker)
            
            if not response.get("results"):
                print(f"⚠️ {ticker} not found in Notion", file=sys.stderr)
                return self._empty_stock_data(ticker)
            
            page = response["results"][0]
            props = page.get("properties", {})
            
            stock_data = {
                "ticker": ticker,
                "current_price": self._extract_number(props.get("Current Price", {})),
                "per": self._extract_number(props.get("P/E", {})),
                "pbv": self._extract_number(props.get("P/B", {})),
                "roe": self._extract_number(props.get("ROE %", {})),
                "der": self._extract_number(props.get("D/E", {})),
                "dividend_yield": self._extract_number(props.get("Dividend Yield %", {})),
                "sentiment_score": 0.3,  # Mock until sentiment API integrated
                "portfolio_value": 279_100_000,  # Christian's portfolio value
                "page_id": page["id"],
                "last_updated": datetime.now().isoformat(),
            }
            
            return stock_data
            
        except Exception as e:
            print(f"❌ Notion fetch error for {ticker}: {e}", file=sys.stderr)
            return self._empty_stock_data(ticker)
    
    def fetch_all_portfolio_stocks(self) -> Dict[str, Dict[str, Any]]:
        """Fetch data for all portfolio stocks"""
        from config import PORTFOLIO_STOCKS
        
        all_data = {}
        for ticker in PORTFOLIO_STOCKS:
            data = self.fetch_stock_data(ticker)
            all_data[ticker] = data
        
        return all_data
    
    def fetch_all_watchlist_stocks(self) -> Dict[str, Dict[str, Any]]:
        """Fetch data for all watchlist stocks"""
        from config import WATCHLIST_STOCKS
        
        all_data = {}
        for ticker in WATCHLIST_STOCKS:
            data = self.fetch_stock_data(ticker)
            all_data[ticker] = data
        
        return all_data
    
    def fetch_all_stocks(self) -> Dict[str, Dict[str, Any]]:
        """Fetch data for all stocks (portfolio + watchlist)"""
        portfolio = self.fetch_all_portfolio_stocks()
        watchlist = self.fetch_all_watchlist_stocks()
        return {**portfolio, **watchlist}
    
    @staticmethod
    def _extract_number(prop: Dict[str, Any], default: float = 0.0) -> float:
        """Extract numeric value from Notion property"""
        try:
            if isinstance(prop, dict):
                if "number" in prop:
                    val = prop["number"]
                    return float(val) if val is not None else default
                elif "formula" in prop and "number" in prop["formula"]:
                    val = prop["formula"]["number"]
                    return float(val) if val is not None else default
            return default
        except (ValueError, TypeError, KeyError):
            return default
    
    @staticmethod
    def _extract_text(prop: Dict[str, Any], default: str = "") -> str:
        """Extract text value from Notion property"""
        try:
            if isinstance(prop, dict):
                if "title" in prop and prop["title"]:
                    return prop["title"][0].get("plain_text", default)
                elif "rich_text" in prop and prop["rich_text"]:
                    return prop["rich_text"][0].get("plain_text", default)
            return default
        except (ValueError, TypeError, KeyError, IndexError):
            return default
    
    @staticmethod
    def _empty_stock_data(ticker: str) -> Dict[str, Any]:
        """Return empty stock data (fallback)"""
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
            "page_id": None,
            "last_updated": datetime.now().isoformat(),
            "error": f"{ticker} not found or data unavailable",
        }


def get_stock_data_from_notion(ticker: str) -> Dict[str, Any]:
    """Convenience function"""
    fetcher = NotionPortfolioFetcher()
    return fetcher.fetch_stock_data(ticker)


def get_all_stocks_from_notion() -> Dict[str, Dict[str, Any]]:
    """Convenience function"""
    fetcher = NotionPortfolioFetcher()
    return fetcher.fetch_all_stocks()
