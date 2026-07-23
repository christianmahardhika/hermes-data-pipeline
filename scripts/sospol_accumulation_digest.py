#!/usr/bin/env python3
"""
SOSPOL-LAB → ACCUMULATION DIGEST PROCESSOR
Real-time conversion of social-political lab sentiment data into actionable buy signals
"""

import json
from datetime import datetime

def process_sospol_data(sospol_input):
    """
    Process sospol-lab data into accumulation tiers and signals
    """
    
    print("=" * 100)
    print(f"📊 ACCUMULATION DIGEST - {datetime.now().strftime('%Y-%m-%d %H:%M WIB')}")
    print("=" * 100)
    
    # Portfolio holdings mapping
    portfolio = {
        "BMRI": {"allocation": 12, "sector": "Banking"},
        "BBRI": {"allocation": 10, "sector": "Banking"},
        "INCO": {"allocation": 2.5, "sector": "Mining"},
        "ANTM": {"allocation": 0, "sector": "Mining"},
        "BJTM": {"allocation": 6, "sector": "Mining"},
        "ADMF": {"allocation": 7, "sector": "Mining"},
        "PTBA": {"allocation": 9, "sector": "Coal"},
        "TAPG": {"allocation": 6, "sector": "Palm"},
        "KLBF": {"allocation": 8, "sector": "Pharma"},
        "TSPC": {"allocation": 7, "sector": "Pharma"},
        "TLKM": {"allocation": 9, "sector": "Telecom"},
        "ASII": {"allocation": 4, "sector": "Auto"},
        "JPFA": {"allocation": 6, "sector": "Agri"},
    }
    
    # Sentiment thresholds for accumulation
    accumulation_tiers = {
        "TIER_1_BUY_NOW": {"sentiment_min": 7.0, "action": "BUY 2-3%", "reason": "Strong positive sentiment + momentum"},
        "TIER_2_BUY_DIP": {"sentiment_min": 5.0, "sentiment_max": 7.0, "action": "BUY 1-1.5%", "reason": "Neutral/mixed sentiment, accumulate"},
        "TIER_3_STAGE": {"sentiment_min": 3.5, "sentiment_max": 5.0, "action": "STAGE BUY (30-40%)", "reason": "Bearish but fundamentals support"},
        "TIER_4_SKIP": {"sentiment_max": 3.5, "action": "SKIP", "reason": "Very bearish, wait for reversal"},
    }
    
    print(f"\n{'-' * 100}")
    print("🎯 ACCUMULATION ACTION PLAN")
    print(f"{'-' * 100}\n")
    
    # Parse input data (expects dict with company sentiment scores)
    # Example: {"BMRI": 7.5, "BBRI": 7.5, "INCO": 4, ...}
    
    tiers = {
        "TIER_1": [],
        "TIER_2": [],
        "TIER_3": [],
        "TIER_4": [],
    }
    
    total_allocation_target = 27  # Rp 27jt portfolio
    allocated = 0
    
    # If sospol_input is provided, parse it
    if sospol_input:
        companies = sospol_input if isinstance(sospol_input, dict) else {}
    else:
        # Use default from recent sospol-lab data Christian forwarded
        companies = {
            "BMRI": 7.5,
            "BBRI": 7.5,
            "INCO": 4.0,
            "ANTM": 7.1,
            "BJTM": 4.0,
            "ADMF": 4.0,
            "PTBA": 3.5,
            "TAPG": 4.0,
            "KLBF": 3.6,
            "TSPC": 7.6,
            "TLKM": 5.3,
            "ASII": 4.0,
            "JPFA": 5.5,
        }
    
    # Categorize into tiers
    for ticker, sentiment_score in sorted(companies.items(), key=lambda x: x[1], reverse=True):
        current_alloc = portfolio.get(ticker, {}).get("allocation", 0)
        sector = portfolio.get(ticker, {}).get("sector", "Other")
        
        if sentiment_score >= 7.0:
            tier = "TIER_1"
            action = "BUY 2-3%"
            rec_alloc = current_alloc + 2.5
        elif sentiment_score >= 5.0:
            tier = "TIER_2"
            action = "BUY 1-1.5%"
            rec_alloc = current_alloc + 1.0
        elif sentiment_score >= 3.5:
            tier = "TIER_3"
            action = "STAGE BUY"
            rec_alloc = current_alloc  # Stage in phases
        else:
            tier = "TIER_4"
            action = "SKIP"
            rec_alloc = current_alloc
        
        entry = {
            "ticker": ticker,
            "sentiment": sentiment_score,
            "sector": sector,
            "current_alloc": current_alloc,
            "rec_alloc": rec_alloc,
            "action": action,
        }
        
        tiers[tier].append(entry)
    
    # Print tiers
    print("🟢 TIER 1 - BUY NOW (Sentiment ≥7.0/10):")
    if tiers["TIER_1"]:
        for item in tiers["TIER_1"]:
            print(f"  • {item['ticker']:6s} ({item['sector']:10s}): {item['sentiment']:.1f}/10")
            print(f"    Current {item['current_alloc']}% → Add 2-3% → New {item['rec_alloc']+1.5:.1f}%")
            print(f"    Action: {item['action']} | Capital: ~Rp {(item['rec_alloc']-item['current_alloc']+1.5)*270000:,.0f}k")
    else:
        print("  (None)")
    
    print(f"\n🟡 TIER 2 - BUY AT DIP (5.0 ≤ Sentiment <7.0):")
    if tiers["TIER_2"]:
        for item in tiers["TIER_2"]:
            print(f"  • {item['ticker']:6s} ({item['sector']:10s}): {item['sentiment']:.1f}/10")
            print(f"    Current {item['current_alloc']}% → Add 1-1.5% → New {item['rec_alloc']+0.75:.1f}%")
            print(f"    Action: {item['action']} | Capital: ~Rp {(item['rec_alloc']-item['current_alloc']+0.75)*270000:,.0f}k")
    else:
        print("  (None)")
    
    print(f"\n🟠 TIER 3 - STAGE ACCUMULATION (3.5 ≤ Sentiment <5.0):")
    if tiers["TIER_3"]:
        for item in tiers["TIER_3"]:
            print(f"  • {item['ticker']:6s} ({item['sector']:10s}): {item['sentiment']:.1f}/10")
            print(f"    Current {item['current_alloc']}%")
            print(f"    Action: Tranche 1 (30%) now, Tranche 2 at lower price | Capital: Flexible")
    else:
        print("  (None)")
    
    print(f"\n🔴 TIER 4 - SKIP (Sentiment <3.5/10):")
    if tiers["TIER_4"]:
        for item in tiers["TIER_4"]:
            print(f"  • {item['ticker']:6s} ({item['sector']:10s}): {item['sentiment']:.1f}/10")
            print(f"    Action: SKIP | Reason: Bearish sentiment, wait for reversal signal")
    else:
        print("  (None)")
    
    # Summary
    print(f"\n{'-' * 100}")
    print("📈 ACCUMULATION SUMMARY")
    print(f"{'-' * 100}\n")
    
    tier1_count = len(tiers["TIER_1"])
    tier2_count = len(tiers["TIER_2"])
    tier3_count = len(tiers["TIER_3"])
    
    print(f"Immediate Action (TIER 1): {tier1_count} companies")
    print(f"  Total Capital: ~Rp {tier1_count * 405000:,.0f}k (if 2-3% each)")
    print(f"  Timeline: BUY TODAY at market open\n")
    
    print(f"Opportunistic (TIER 2): {tier2_count} companies")
    print(f"  Total Capital: ~Rp {tier2_count * 202500:,.0f}k (if 1-1.5% each)")
    print(f"  Timeline: Buy on intra-day dips\n")
    
    print(f"Staged Accumulation (TIER 3): {tier3_count} companies")
    print(f"  Total Capital: Flexible (30-40% per tranche)")
    print(f"  Timeline: Tranche 1 this week, Tranche 2 next week\n")
    
    skip_count = len(tiers["TIER_4"])
    print(f"Skip (TIER 4): {skip_count} companies")
    print(f"  Reason: Bearish sentiment, low conviction\n")
    
    # Action checklist
    print(f"{'-' * 100}")
    print("✅ EXECUTION CHECKLIST")
    print(f"{'-' * 100}\n")
    
    print("[ ] TIER 1 - TODAY:")
    for item in tiers["TIER_1"]:
        print(f"    [ ] {item['ticker']} - BUY 2-3% (Check limit order levels)")
    
    print("\n[ ] TIER 2 - THIS WEEK:")
    for item in tiers["TIER_2"]:
        print(f"    [ ] {item['ticker']} - BUY on dip (Set intra-day alerts)")
    
    print("\n[ ] TIER 3 - STAGED:")
    for item in tiers["TIER_3"]:
        print(f"    [ ] {item['ticker']} - Tranche 1 now, Tranche 2 next week")
    
    print("\n[ ] SKIP:")
    for item in tiers["TIER_4"]:
        print(f"    ⏸ {item['ticker']} - Monitor for reversal signal")
    
    print(f"\n{'-' * 100}")
    print("✓ Accumulation digest generated. Ready for execution.")
    print(f"{'-' * 100}\n")

# Run with default sospol-lab data
if __name__ == "__main__":
    process_sospol_data(None)
