#!/usr/bin/env python3
"""
Portfolio Fundamental Scraper — Updated Watchlist
Screens qualified stocks: LSIP, BBRI, PTBA, BJTM, ADMF, JPFA, TSPC, BMRI, ASII, ANTM
Runs daily at 10:00 WIB
"""

import yfinance as yf
import json
from datetime import datetime

# ✅ QUALIFIED WATCHLIST (Score 4-5/5 criteria)
QUALIFIED_STOCKS = [
    "LSIP.JK",  # 5/5 — Plantation, best overall
    "KMDS.JK",  # 5/5 — Food Distribution (PER 5.70, ROE 21.3%)
    "AVIA.JK",  # 5/5 — Paint & Building Materials (PER 10.25, ROE 18.1%)
    "PTBA.JK",  # 4/5 — Mining (Coal)
    "BJTM.JK",  # 4/5 — Infrastructure
    "ADMF.JK",  # 4/5 — Agroindustry
    "JPFA.JK",  # 4/5 — Agriculture
    "TSPC.JK",  # 4/5 — Chemicals
    "BMRI.JK",  # 4/5 — Banking
    "ASII.JK",  # 4/5 — Automotive
    "ANTM.JK",  # 4/5 — Mining (Nickel)
]

CRITERIA = {"PER": 15, "PBV": 2, "DY": 3, "ROE": 10, "DER": 1}

def get_fundamentals(ticker):
    """Fetch fundamental metrics from Yahoo Finance."""
    try:
        stock = yf.Ticker(ticker)
        info = stock.info
        
        per = info.get("trailingPE")
        pbv = info.get("priceToBook")
        roe = info.get("returnOnEquity")
        dy = info.get("dividendYield")
        der = info.get("debtToEquity")
        current_price = info.get("currentPrice")
        market_cap = info.get("marketCap")
        
        # Safe percentage conversions
        if roe and isinstance(roe, float) and 0 < roe < 1:
            roe = round(roe * 100, 2)
        if dy and isinstance(dy, float):
            dy = round(dy * 100, 2) if dy < 1 else dy
        
        return {
            "ticker": ticker,
            "price": current_price,
            "market_cap": market_cap,
            "per": per,
            "pbv": pbv,
            "roe": roe,
            "dy": dy,
            "der": der,
        }
    except Exception as e:
        return None

def score_stock(data):
    """Count criteria met (DER optional due to banking sector variance)."""
    score = 0
    if data['per'] and data['per'] < CRITERIA['PER']:
        score += 1
    if data['pbv'] and data['pbv'] < CRITERIA['PBV']:
        score += 1
    if data['dy'] and data['dy'] > CRITERIA['DY']:
        score += 1
    if data['roe'] and data['roe'] > CRITERIA['ROE']:
        score += 1
    # DER is optional — banking stocks typically have high DER
    if data['der'] and data['der'] < CRITERIA['DER']:
        score += 1
    return score

def get_der_note(ticker, der):
    """Return note about DER status."""
    if der is None:
        return "⚠️  DER: missing"
    elif der > CRITERIA['DER']:
        is_bank = 'BRI' in ticker or 'BMRI' in ticker or 'BJTM' in ticker
        note = "(banking sector)" if is_bank else ""
        return f"⚠️  DER: {der:.3f} high {note}"
    return None

# Fetch all stocks
print("=" * 80)
print("PORTFOLIO FUNDAMENTAL SCRAPER — QUALIFIED WATCHLIST")
print(f"Updated: {datetime.now().strftime('%Y-%m-%d %H:%M WIB')}")
print("=" * 80)

results = []
for ticker in QUALIFIED_STOCKS:
    data = get_fundamentals(ticker)
    if data:
        # Accept if core metrics present (PER, PBV, ROE, DY) — DER optional for banking
        core_fields = ['per', 'pbv', 'roe', 'dy']
        if all(data.get(field) is not None for field in core_fields):
            score = score_stock(data)
            data['score'] = score
            results.append(data)
            
            # Print summary
            harga = f"Rp {data['price']:>8,.0f}" if data['price'] else "N/A"
            der_str = f"{data['der']:>6.3f}" if data['der'] else "N/A"
            print(f"\n{ticker:<10} | Score: {score}/5 | Harga: {harga}")
            print(f"  PER: {data['per']:<6.2f} | PBV: {data['pbv']:<6.2f} | ROE: {data['roe']:>6.1f}% | DY: {data['dy']:>6.2f}% | DER: {der_str}")
            
            # Flag DER if needed
            der_note = get_der_note(ticker, data['der'])
            if der_note:
                print(f"  {der_note}")
        else:
            # Log incomplete data (missing core metrics)
            missing = [f for f in core_fields if data.get(f) is None]
            print(f"\n{ticker:<10} | ⚠️  Incomplete — missing: {', '.join(missing)}")

# Summary
print("\n" + "=" * 80)
print("SUMMARY")
print("=" * 80)
print(f"Total screened: {len(results)} qualified stocks")
print(f"Average PER: {sum([r['per'] for r in results if r['per']]) / len([r for r in results if r['per']]):.2f}")
print(f"Average DY: {sum([r['dy'] for r in results if r['dy']]) / len([r for r in results if r['dy']]):.2f}%")
print(f"Average ROE: {sum([r['roe'] for r in results if r['roe']]) / len([r for r in results if r['roe']]):.2f}%")
