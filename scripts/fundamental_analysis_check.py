#!/usr/bin/env python3
"""
Fundamental Analysis Check - Script-based cron job
Fetches fundamental data for portfolio stocks and checks valuation criteria
"""

import json
import subprocess
import sys
from datetime import datetime

# Target stocks
TICKERS = ["PTBA", "INCO", "ANTM", "BMRI"]

# Valuation criteria
CRITERIA = {
    "PER": 15,      # < 15
    "PBV": 2,       # < 2
    "DY": 3,        # > 3%
    "ROE": 10,      # > 10%
    "DER": 1        # < 1
}

def get_fundamental_data(ticker):
    """Fetch fundamental data using hermes MCP tools"""
    try:
        # Use hermes CLI to call MCP tool - same as stock_alerts_bumn_export.py
        result = subprocess.run(
            ["hermes", "mcp", "call", "mcp_idx_fundamental_mcp_get_fundamental", 
             "--ticker", ticker],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        if result.returncode == 0:
            try:
                data = json.loads(result.stdout)
                return data
            except json.JSONDecodeError as e:
                print(f"JSON decode error for {ticker}: {e}", file=sys.stderr)
                print(f"Raw output: {result.stdout}", file=sys.stderr)
                return None
        else:
            print(f"Error fetching {ticker}: {result.stderr}", file=sys.stderr)
            return None
    except Exception as e:
        print(f"Exception fetching {ticker}: {e}", file=sys.stderr)
        return None

def check_valuation(data):
    """Check if stock meets valuation criteria"""
    if not data:
        return None
    
    per = data.get("per")
    pbv = data.get("pbv")
    dy = data.get("dividend_yield", 0)
    roe = data.get("roe")
    der = data.get("debt_to_equity")
    
    # Count criteria met
    criteria_met = 0
    total_criteria = 0
    
    if per is not None:
        total_criteria += 1
        if per < CRITERIA["PER"]:
            criteria_met += 1
    
    if pbv is not None:
        total_criteria += 1
        if pbv < CRITERIA["PBV"]:
            criteria_met += 1
    
    if dy is not None:
        total_criteria += 1
        if dy > CRITERIA["DY"]:
            criteria_met += 1
    
    if roe is not None:
        total_criteria += 1
        if roe > CRITERIA["ROE"]:
            criteria_met += 1
    
    if der is not None:
        total_criteria += 1
        if der < CRITERIA["DER"]:
            criteria_met += 1
    
    # Determine status
    if total_criteria == 0:
        status = "N/A"
    elif criteria_met >= 4:
        status = "✅ UNDERVALUED"
    elif criteria_met >= 3:
        status = "⚠️ FAIR"
    else:
        status = "❌ OVERVALUED"
    
    return {
        "per": per,
        "pbv": pbv,
        "dy": dy,
        "roe": roe,
        "der": der,
        "status": status
    }

def main():
    """Main function"""
    results = []
    
    for ticker in TICKERS:
        data = get_fundamental_data(ticker)
        valuation = check_valuation(data)
        
        if valuation:
            results.append({
                "ticker": ticker,
                "data": valuation
            })
    
    if not results:
        # Silent if no data
        print("[SILENT]")
        return
    
    # Format output
    print("## Fundamental Analysis Check")
    print(f"*Updated: {datetime.now().strftime('%Y-%m-%d %H:%M WIB')}*\n")
    
    print("| Ticker | PER | PBV | DY | ROE | DER | Status |")
    print("|--------|-----|-----|-----|-----|-----|--------|")
    
    for result in results:
        ticker = result["ticker"]
        data = result["data"]
        
        per = f"{data['per']:.1f}" if data['per'] is not None else "N/A"
        pbv = f"{data['pbv']:.2f}" if data['pbv'] is not None else "N/A"
        dy = f"{data['dy']:.1f}%" if data['dy'] is not None else "N/A"
        roe = f"{data['roe']:.1f}%" if data['roe'] is not None else "N/A"
        der = f"{data['der']:.2f}" if data['der'] is not None else "N/A"
        status = data['status']
        
        print(f"| {ticker} | {per} | {pbv} | {dy} | {roe} | {der} | {status} |")
    
    print("\n**Criteria:** PER < 15, PBV < 2, DY > 3%, ROE > 10%, DER < 1")

if __name__ == "__main__":
    main()
