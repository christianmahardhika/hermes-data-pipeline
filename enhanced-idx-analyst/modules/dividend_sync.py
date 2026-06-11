"""
Dividend Sync Module — Populate Notion dengan Historical Dividend Data
Sync dari yfinance/IDX scraper ke Notion Portfolio database
"""

import sys
import os
from typing import Dict, List, Any, Optional
from datetime import datetime
from pathlib import Path

try:
    from notion_client import Client
except ImportError:
    print("⚠️ notion-client not installed. Install: pip install notion-client", file=sys.stderr)
    Client = None


class DividendSyncer:
    """Sync historical dividend records to Notion"""
    
    def __init__(self, notion_api_key: Optional[str] = None, portfolio_db_id: Optional[str] = None):
        self.notion_api_key = notion_api_key or os.getenv("NOTION_API_KEY")
        self.portfolio_db_id = portfolio_db_id
        
        if not self.notion_api_key:
            print("❌ NOTION_API_KEY not set in env", file=sys.stderr)
            self.client = None
            return
        
        if Client:
            self.client = Client(auth=self.notion_api_key)
        else:
            self.client = None
    
    def get_portfolio_stocks(self) -> List[Dict[str, Any]]:
        """Fetch all stocks from Notion Portfolio database"""
        if not self.client or not self.portfolio_db_id:
            return []
        
        try:
            response = self.client.databases.query(
                database_id=self.portfolio_db_id,
                page_size=100
            )
            
            stocks = []
            for page in response.get("results", []):
                props = page.get("properties", {})
                ticker = self._extract_text(props.get("Ticker"))
                if ticker:
                    stocks.append({
                        "page_id": page["id"],
                        "ticker": ticker,
                        "properties": props
                    })
            
            return stocks
        except Exception as e:
            print(f"❌ Error fetching portfolio stocks: {e}", file=sys.stderr)
            return []
    
    def _extract_text(self, prop: Any) -> Optional[str]:
        """Extract text from Notion property"""
        if prop is None:
            return None
        
        if isinstance(prop, dict):
            if prop.get("type") == "title":
                title_list = prop.get("title", [])
                if title_list:
                    return title_list[0].get("plain_text", "")
            elif prop.get("type") == "rich_text":
                rich_text_list = prop.get("rich_text", [])
                if rich_text_list:
                    return rich_text_list[0].get("plain_text", "")
            elif prop.get("type") == "select":
                return prop.get("select", {}).get("name", "")
        
        return None
    
    def _extract_number(self, prop: Any) -> Optional[float]:
        """Extract number from Notion property"""
        if prop is None:
            return None
        
        if isinstance(prop, dict) and prop.get("type") == "number":
            return prop.get("number")
        
        return None
    
    def sync_dividends_for_stock(
        self,
        page_id: str,
        ticker: str,
        historical_dividends: List[Dict[str, Any]]
    ) -> bool:
        """Update Notion page with dividend history"""
        if not self.client:
            print(f"⚠️ Notion client not available, skipping sync for {ticker}", file=sys.stderr)
            return False
        
        try:
            # Format dividend history as markdown table
            dividend_text = self._format_dividend_history(historical_dividends)
            
            # Update Notion page
            self.client.pages.update(
                page_id=page_id,
                properties={
                    "Dividend History": {
                        "type": "rich_text",
                        "rich_text": [
                            {
                                "type": "text",
                                "text": {"content": dividend_text}
                            }
                        ]
                    }
                }
            )
            
            print(f"✅ Synced {len(historical_dividends)} dividends for {ticker}", file=sys.stderr)
            return True
        except Exception as e:
            print(f"⚠️ Error syncing dividends for {ticker}: {e}", file=sys.stderr)
            return False
    
    def _format_dividend_history(self, dividends: List[Dict[str, Any]]) -> str:
        """Format dividend history as readable text"""
        if not dividends:
            return "No dividend history"
        
        lines = []
        total_per_year = {}
        
        for div in dividends:
            date_str = div.get("date", "")
            per_share = div.get("per_share", 0)
            
            if date_str:
                year = date_str[:4]
                if year not in total_per_year:
                    total_per_year[year] = 0
                total_per_year[year] += per_share
                
                lines.append(f"• {date_str}: Rp {per_share:,.2f}/share")
        
        # Add summary
        summary_lines = []
        for year in sorted(total_per_year.keys(), reverse=True):
            summary_lines.append(f"  {year}: Rp {total_per_year[year]:,.2f}/share")
        
        result = "Recent Dividends:\n" + "\n".join(lines[:10])
        if summary_lines:
            result += "\n\nAnnual Dividend Summary:\n" + "\n".join(summary_lines)
        
        return result
    
    def sync_all_portfolio(self, historical_dividends_map: Dict[str, List[Dict[str, Any]]]) -> Dict[str, bool]:
        """Sync all portfolio stocks with dividend history"""
        stocks = self.get_portfolio_stocks()
        results = {}
        
        for stock in stocks:
            ticker = stock["ticker"]
            page_id = stock["page_id"]
            
            dividends = historical_dividends_map.get(ticker, [])
            if dividends:
                results[ticker] = self.sync_dividends_for_stock(page_id, ticker, dividends)
            else:
                results[ticker] = None  # No dividend data
        
        return results
    
    def update_latest_dividend(
        self,
        page_id: str,
        ticker: str,
        latest_dividend: Dict[str, Any]
    ) -> bool:
        """Update latest dividend yield and amount in Notion"""
        if not self.client:
            return False
        
        try:
            per_share = latest_dividend.get("per_share", 0)
            yield_pct = latest_dividend.get("yield", 0)
            
            # Update properties
            self.client.pages.update(
                page_id=page_id,
                properties={
                    "Latest Dividend": {
                        "type": "number",
                        "number": per_share
                    },
                    "Dividend Yield": {
                        "type": "number",
                        "number": yield_pct
                    }
                }
            )
            
            print(f"✅ Updated latest dividend for {ticker}: Rp {per_share:.2f} ({yield_pct:.2f}%)", file=sys.stderr)
            return True
        except Exception as e:
            print(f"⚠️ Error updating latest dividend for {ticker}: {e}", file=sys.stderr)
            return False


class DividendRecorder:
    """Record dividend transactions to Notion Transaction database"""
    
    def __init__(self, notion_api_key: Optional[str] = None, transaction_db_id: Optional[str] = None):
        self.notion_api_key = notion_api_key or os.getenv("NOTION_API_KEY")
        self.transaction_db_id = transaction_db_id
        
        if not self.notion_api_key:
            print("❌ NOTION_API_KEY not set in env", file=sys.stderr)
            self.client = None
            return
        
        if Client:
            self.client = Client(auth=self.notion_api_key)
        else:
            self.client = None
    
    def record_dividend_receipt(
        self,
        ticker: str,
        shares_held: int,
        dividend_per_share: float,
        payment_date: str,
        notes: str = ""
    ) -> bool:
        """Record dividend payment received"""
        if not self.client or not self.transaction_db_id:
            return False
        
        try:
            total_dividend = shares_held * dividend_per_share
            
            self.client.pages.create(
                parent={"database_id": self.transaction_db_id},
                properties={
                    "Date": {
                        "type": "date",
                        "date": {"start": payment_date}
                    },
                    "Type": {
                        "type": "select",
                        "select": {"name": "Dividend"}
                    },
                    "Ticker": {
                        "type": "rich_text",
                        "rich_text": [{"type": "text", "text": {"content": ticker}}]
                    },
                    "Shares": {
                        "type": "number",
                        "number": shares_held
                    },
                    "Amount Per Share": {
                        "type": "number",
                        "number": dividend_per_share
                    },
                    "Total Amount": {
                        "type": "number",
                        "number": total_dividend
                    },
                    "Notes": {
                        "type": "rich_text",
                        "rich_text": [{"type": "text", "text": {"content": notes}}]
                    }
                }
            )
            
            print(f"✅ Recorded dividend for {ticker}: {shares_held} shares × Rp {dividend_per_share:.2f} = Rp {total_dividend:,.0f}", file=sys.stderr)
            return True
        except Exception as e:
            print(f"⚠️ Error recording dividend for {ticker}: {e}", file=sys.stderr)
            return False


def sync_dividends_to_notion(
    portfolio_stocks: List[str],
    dividend_data_map: Dict[str, List[Dict[str, Any]]],
    portfolio_db_id: str,
    notion_api_key: Optional[str] = None
) -> Dict[str, bool]:
    """Convenience function to sync dividends"""
    syncer = DividendSyncer(notion_api_key, portfolio_db_id)
    
    # Fetch stocks and sync
    stocks = syncer.get_portfolio_stocks()
    results = {}
    
    for stock in stocks:
        ticker = stock["ticker"]
        page_id = stock["page_id"]
        
        dividends = dividend_data_map.get(ticker, [])
        if dividends:
            results[ticker] = syncer.sync_dividends_for_stock(page_id, ticker, dividends)
    
    return results
