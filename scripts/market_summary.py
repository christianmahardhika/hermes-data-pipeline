#!/usr/bin/env python3
"""
Market Summary Script - Fetch IHSG market data and top movers
Runs daily at 16:00 WIB
"""

import subprocess
import json
import sys

def run_command(cmd):
    """Run shell command and return output"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=30)
        return result.stdout.strip()
    except Exception as e:
        print(f"Error running command: {e}", file=sys.stderr)
        return None

def main():
    print("📊 IHSG Market Summary - " + run_command("date '+%d %B %Y %H:%M WIB'"))
    print("=" * 70)
    
    # Try to get market summary using hermes CLI
    cmd = """hermes mcp call mcp_technical_mcp_get_market_summary '{"include_movers": true, "movers_limit": 5}' 2>/dev/null || echo '{"error": "Market data unavailable"}'"""
    
    output = run_command(cmd)
    
    if output and "error" not in output.lower():
        try:
            data = json.loads(output)
            print(json.dumps(data, indent=2, ensure_ascii=False))
        except:
            print(output)
    else:
        # Fallback: show portfolio stocks status
        print("\n⚠️  Market data temporarily unavailable")
        print("Showing portfolio status instead:\n")
        
        portfolio = ["PTBA", "INCO", "ANTM", "BMRI", "BBRI", "KLBF", "TLKM"]
        print(f"Portfolio stocks to monitor: {', '.join(portfolio)}")
        print("\nTip: Check Stockbit or IDX website for real-time prices")

if __name__ == "__main__":
    main()
