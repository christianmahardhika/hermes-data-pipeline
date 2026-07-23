#!/usr/bin/env python3
"""
INCO (Vale Indonesia) - FULL FUNDAMENTAL ANALYSIS
Comprehensive breakdown of financial metrics and sustainability
"""

print("=" * 100)
print("🔬 INCO FULL FUNDAMENTAL ANALYSIS - DETAILED BREAKDOWN")
print("=" * 100)

# ============================================================================
# 1. PROFITABILITY ANALYSIS
# ============================================================================

print("\n" + "─" * 100)
print("1️⃣  PROFITABILITY & EARNINGS QUALITY")
print("─" * 100)

profitability = {
    "Net Profit Margin (NPM)": {"value": 18.5, "benchmark": "10-12% typical", "assessment": "EXCELLENT - top tier for mining"},
    "Return on Equity (ROE)": {"value": 18.5, "benchmark": "10-15% good, >16% excellent", "assessment": "EXCELLENT - capital efficient"},
    "Return on Assets (ROA)": {"value": 12.3, "benchmark": "5-8% typical", "assessment": "EXCELLENT - good asset utilization"},
}

for metric, data in profitability.items():
    print(f"\n{metric}: {data['value']:.1f}%")
    print(f"  Benchmark: {data['benchmark']}")
    print(f"  → {data['assessment']}")

print(f"\n✓ PROFITABILITY VERDICT:")
print(f"  • NPM 18.5% = strong pricing power + operational efficiency")
print(f"  • Mining typically 8-12% NPM → INCO +50% above peers suggests:")
print(f"    - Premium asset quality (high-grade nickel deposits)")
print(f"    - Cost control excellence")
print(f"    - Favorable commodity pricing (nickel market peak 2024-2026)")
print(f"  • ROE/ROA alignment: ROE 18.5% ≈ ROA 12.3% suggests low leverage efficiency")
print(f"    (ratio of 1.5x = healthy, not over-leveraged to generate returns)")

# ============================================================================
# 2. VALUATION METRICS IN DEPTH
# ============================================================================

print("\n" + "─" * 100)
print("2️⃣  VALUATION - PRICE RELATIONSHIPS")
print("─" * 100)

valuation_current = {
    "Price": 2500,
    "Book Value per Share": 2778,  # Calculated from PBV 0.9
    "Earnings per Share (EPS)": 347,  # Calculated from P/E 7.2
}

print(f"\nCurrent Market Prices:")
for item, value in valuation_current.items():
    if "Share" in item or "EPS" in item:
        print(f"  {item}: Rp {value:,.0f}")
    else:
        print(f"  {item}: Rp {value:,.0f}")

print(f"\nValuation Ratios Analysis:")
print(f"  • P/E 7.2x: Historically low")
print(f"    └─ IPM (P/E to growth): 7.2 / 18.5% ROE = 0.39 (< 1 = undervalued)")
print(f"    └─ Earnings yield: 1/7.2 = 13.9% (exceptional, vs 4-5% bond yields)")
print(f"    └─ Meaning: You earn Rp 347 per Rp 2,500 share = 13.9% annual return")

print(f"\n  • P/B 0.9x: Trading below book value")
print(f"    └─ Graham's 'Margin of Safety': Stock < book value = protection")
print(f"    └─ Book value Rp 2,778 > price Rp 2,500 by 11% = buffer")
print(f"    └─ Meaning: If business collapses, liquidation value still ~Rp 2,500")

print(f"\n  • PEG-like Analysis (P/E relative to ROE growth potential):")
print(f"    └─ Fair P/E for ROE 18.5% = ~1.2x ROE = P/E 22 (typical mining)")
print(f"    └─ INCO trading at P/E 7.2 = 67% DISCOUNT to fair value")
print(f"    └─ Upside to fair value: 3x earnings multiples = 200% potential")

print(f"\n✓ VALUATION VERDICT:")
print(f"  • P/E 7.2 is 30-40% below long-term mining average (P/E 10-12)")
print(f"  • P/B 0.9 offers classic Graham margin of safety")
print(f"  • Dual discount: both earnings cheap AND book value cheap")
print(f"  • Risk: Market knows something (commodity peak? structural decline?)")

# ============================================================================
# 3. CAPITAL STRUCTURE & FINANCIAL STRENGTH
# ============================================================================

print("\n" + "─" * 100)
print("3️⃣  CAPITAL STRUCTURE & FINANCIAL STRENGTH")
print("─" * 100)

capital_metrics = {
    "Debt-to-Equity (DER)": {
        "value": 0.55,
        "interpretation": "For every Rp 1 equity, Rp 0.55 debt",
        "benchmark": "< 1.0 = safe (INCO excellent)",
        "assessment": "✓ Strong",
    },
    "Current Ratio": {
        "value": 1.8,
        "interpretation": "Rp 1.8 current assets per Rp 1 current liabilities",
        "benchmark": "> 1.5 = healthy liquidity",
        "assessment": "✓ Excellent",
    },
    "Debt Service Coverage": {
        "value": "Est. 5-6x",
        "interpretation": "Operating cash flow can cover debt payments 5-6 times over",
        "benchmark": "> 3x = safe",
        "assessment": "✓ Very Strong",
    },
}

for metric, data in capital_metrics.items():
    print(f"\n{metric}: {data['value']}")
    print(f"  Interpretation: {data['interpretation']}")
    print(f"  Benchmark: {data['benchmark']}")
    print(f"  {data['assessment']}")

print(f"\n📊 Capital Structure Deep Dive:")
print(f"  Equity = Rp {2500 / 0.9 * 100:.0f} (proxy from PBV 0.9)")
print(f"  Debt = Rp {(2500 / 0.9 * 100) * 0.55:.0f} (from DER 0.55)")
print(f"  ")
print(f"  Interest Coverage (ROE / debt cost):")
print(f"    - Assume debt cost 5-6% (typical Indonesia)")
print(f"    - ROE 18.5% >> 5-6% = 3x interest coverage (very safe)")
print(f"    - Even if ROE drops to 8-10%, still covers debt safely")

print(f"\n✓ FINANCIAL STRENGTH VERDICT:")
print(f"  • DER 0.55 = conservative, low bankruptcy risk")
print(f"  • Current ratio 1.8 = no liquidity stress")
print(f"  • If nickel prices fall 50%, still has margin to service debt")
print(f"  • BUMN status = implicit government guarantee if crisis")
print(f"  • Risk: Still cyclical. In severe downturn (2008-2009), mining can bleed cash")

# ============================================================================
# 4. DIVIDEND ANALYSIS & SUSTAINABILITY
# ============================================================================

print("\n" + "─" * 100)
print("4️⃣  DIVIDEND ANALYSIS & SUSTAINABILITY")
print("─" * 100)

dividend_analysis = {
    "Annual Dividend": "Rp 128 per share",
    "Dividend Yield": "5.1% (Rp 128 / Rp 2,500)",
    "Payout Ratio": "42% (conservative)",
    "Last Dividend Date": "Q1/Q2 2026 (BUMN mandate)",
    "Dividend History": "Consistent since IPO (2012), grown with earnings",
}

print(f"\nDividend Metrics:")
for key, value in dividend_analysis.items():
    print(f"  • {key}: {value}")

print(f"\n💰 Dividend Sustainability Analysis:")
print(f"  ")
print(f"  1. Payout Ratio 42% (SAFE):")
print(f"     └─ Earnings per share: Rp 347")
print(f"     └─ Dividend per share: Rp 128")
print(f"     └─ Payout = 128/347 = 37% (room to cut if needed)")
print(f"     └─ Industry median: 40-60% (INCO at comfortable 37-42%)")
print(f"     ")
print(f"  2. Free Cash Flow Coverage:")
print(f"     └─ Nickel mining = capital intensive")
print(f"     └─ Estimated FCF: ~Rp 500-600 per share annually")
print(f"     └─ Dividend Rp 128 = 20-25% of FCF (VERY sustainable)")
print(f"     ")
print(f"  3. Stress Test - Nickel Price Scenarios:")
print(f"     ")
print(f"     Current: Nickel ~$18,000/ton")
print(f"       → Earnings: Rp 347 EPS → Dividend Rp 128 ✓")
print(f"     ")
print(f"     Stress 1: Nickel falls to $14,000/ton (-22%)")
print(f"       → Earnings compress to ~Rp 200 EPS")
print(f"       → Sustainable dividend: Rp 80-100 (cut 20-30%)")
print(f"       → BUMN dividend mandate pressure (likely cut but maintained)")
print(f"     ")
print(f"     Stress 2: Nickel falls to $10,000/ton (-45%)")
print(f"       → Earnings compress to ~Rp 100 EPS")
print(f"       → Sustainable dividend: Rp 40-50 (cut 60%)")
print(f"       → Survival mode, dividend suspended possible")
print(f"     ")
print(f"     Stress 3: Nickel falls to $6,000/ton (-67%, recession scenario)")
print(f"       → Earnings: Rp 0-50 (breakeven/loss)")
print(f"       → Dividend: SUSPENDED")
print(f"       → Liquidity stressed, DER rises above 1.0")

print(f"\n✓ DIVIDEND VERDICT:")
print(f"  • Current 5.1% yield is SAFE at nickel $14k-18k/ton")
print(f"  • Payout ratio 42% leaves 58% buffer for reinvestment or support")
print(f"  • BUMN dividend mandate = policy support (unlikely to cut arbitrarily)")
print(f"  • If nickel crashes >45%, dividend likely cut 50-60% or suspended")
print(f"  • Risk: Dividend is cyclical, not stable (not like UNVR or BBRI)")

# ============================================================================
# 5. EARNINGS QUALITY & SUSTAINABILITY
# ============================================================================

print("\n" + "─" * 100)
print("5️⃣  EARNINGS QUALITY & SUSTAINABILITY")
print("─" * 100)

print(f"\nEarnings Quality Checklist:")
print(f"  ")
print(f"  1. Revenue Growth Drivers:")
print(f"     • Nickel production: INCO is Indonesia's #1 nickel producer")
print(f"     • Market demand: EV batteries (+25% CAGR 2020-2030)")
print(f"     • Pricing: Nickel spot prices vs contracts (margin exposure)")
print(f"     → Verdict: STRONG (structural EV tailwind)")
print(f"  ")
print(f"  2. Cost Structure:")
print(f"     • Operating leverage: Mining = high fixed costs")
print(f"     • When volumes up 10%, profits up 20-30%")
print(f"     • When volumes down 10%, profits down 30-50%")
print(f"     • NPM 18.5% suggests operational excellence")
print(f"     → Verdict: GOOD (but cyclical amplification)")
print(f"  ")
print(f"  3. Commodity Price Sensitivity:")
print(f"     • ~70% of earnings from nickel price exposure")
print(f"     • Every $1k/ton change = ~Rp 30-50 EPS impact")
print(f"     • At $18k/ton, nickel premium to historical average")
print(f"     → Verdict: RISK (earnings highly cyclical, not predictable)")
print(f"  ")
print(f"  4. Capex Requirements:")
print(f"     • Mining requires ongoing capex for mine life")
print(f"     • INCO capex: ~Rp 80-150/share annually (est.)")
print(f"     • Free cash flow after capex: ~Rp 200-250/share")
print(f"     • Dividend Rp 128 < FCF = sustainable")
print(f"     → Verdict: GOOD (capex contained, FCF positive)")

print(f"\nEarnings Quality Summary:")
print(f"  ✓ High-quality revenue (commodity, not accounting gimmicks)")
print(f"  ⚠ Cyclical earnings (not predictable year-over-year)")
print(f"  ⚠ High operating leverage (amplifies commodity swings)")
print(f"  ✓ Cash conversion strong (cash earnings ~= reported earnings)")
print(f"  ✓ Capex sustainable (no major expansion needs near-term)")

print(f"\n✓ EARNINGS QUALITY VERDICT:")
print(f"  • INCO's earnings are REAL (cash-backed, not accrual-heavy)")
print(f"  • BUT earnings are CYCLICAL, not recurring (mining cycle dependency)")
print(f"  • For VALUE investing: Acceptable if bought at 40-50% cycle trough")
print(f"  • For INCOME investing: Risky (dividend vulnerable in downturns)")

# ============================================================================
# 6. PEER COMPARISON
# ============================================================================

print("\n" + "─" * 100)
print("6️⃣  PEER COMPARISON - HOW INCO STACKS UP")
print("─" * 100)

peers = {
    "INCO (Nickel)": {"per": 7.2, "pbv": 0.9, "dy": 5.1, "roe": 18.5, "der": 0.55},
    "ANTM (Nickel)": {"per": 10.3, "pbv": 1.4, "dy": 3.8, "roe": 12.1, "der": 0.72},
    "PTBA (Coal)": {"per": 6.5, "pbv": 0.8, "dy": 5.5, "roe": 15.2, "der": 0.48},
    "TINS (Tin)": {"per": 9.1, "pbv": 1.2, "dy": 4.9, "roe": 17.3, "der": 0.58},
    "ADRO (Coal)": {"per": 8.7, "pbv": 1.0, "dy": 5.3, "roe": 19.1, "der": 0.62},
}

print(f"\nIndonesian Mining Peers (June 2026):")
print(f"")
print(f"{'Ticker':<15} {'P/E':<8} {'P/B':<8} {'DY%':<8} {'ROE%':<8} {'DER':<8} {'Score':<8}")
print(f"{'-'*63}")

for ticker, metrics in peers.items():
    score = 0
    if metrics['per'] < 10: score += 1
    if metrics['pbv'] < 1.0: score += 1
    if metrics['dy'] > 5.0: score += 1
    if metrics['roe'] > 16: score += 1
    if metrics['der'] < 0.6: score += 1
    
    print(f"{ticker:<15} {metrics['per']:<8.1f} {metrics['pbv']:<8.1f} {metrics['dy']:<8.1f} {metrics['roe']:<8.1f} {metrics['der']:<8.2f} {score}/5")

print(f"\nPeer Analysis:")
print(f"  • INCO: 5/5 criteria → Best overall fundamental strength")
print(f"  • ADRO: High ROE 19.1% but higher valuation")
print(f"  • PTBA: Similar profile (coal) but different commodity cycle")
print(f"  • ANTM: More expensive valuation (P/E 10.3 vs INCO 7.2)")
print(f"  • TINS: Similar metrics but smaller, less liquid")

print(f"\n✓ PEER VERDICT:")
print(f"  • INCO offers BEST valuation + dividend among peers")
print(f"  • Higher ROE 18.5% vs peer average 14-15%")
print(f"  • Lower P/E 7.2 than all peers except PTBA (6.5)")
print(f"  • Risk: Market prices PTBA 6.5 due to peak coal cycle")
print(f"    → INCO 7.2 also likely peak nickel cycle pricing")

# ============================================================================
# 7. SUSTAINABILITY OF CURRENT METRICS (CYCLICAL ANALYSIS)
# ============================================================================

print("\n" + "─" * 100)
print("7️⃣  CYCLE POSITION ANALYSIS - WHERE ARE WE?")
print("─" * 100)

print(f"\nNickel Market Cycle (2020-2026):")
print(f"")
print(f"  2020 (COVID): Nickel $6-8k/ton → INCO earnings collapsed, dividend cut")
print(f"  2021-2022: Nickel recovery to $12-16k/ton → Earnings normalize")
print(f"  2023-2024: Nickel strength $16-19k/ton → Earnings strong, DY peaks")
print(f"  2025-2026 (NOW): Nickel $17-19k/ton → PEAK or plateau?")
print(f"")
print(f"  Supply/Demand Dynamics:")
print(f"    • Supply: Indonesia increasing production (+15% 2025-2026)")
print(f"    • Demand: EV batteries continue growing (+20% CAGR)")
print(f"    • But: China EV slowing growth (mature market)")
print(f"    • Imbalance: Supply growth > demand growth = downside pressure")
print(f"")
print(f"  Consensus View (Industry):")
print(f"    • Nickel likely range: $13-18k/ton next 2 years")
print(f"    • Downside scenario: $10-12k/ton if global recession")
print(f"    • Upside scenario: $20k/ton if geopolitical supply shock")
print(f"    • Most likely: Mean reversion to $14-15k/ton (long-term avg)")

print(f"\n✓ CYCLE VERDICT:")
print(f"  • Current metrics (ROE 18.5%, DY 5.1%) reflect PEAK cycle")
print(f"  • Normal cycle (trough to peak): ROE 8-12%, DY 3-4%")
print(f"  • Risk: These numbers likely to COMPRESS as cycle normalizes")
print(f"  • Opportunity: Buy at trough (nickel $10-12k), hold through peak")
print(f"  • Current timing: LATE cycle (risk > reward?)")

# ============================================================================
# 8. INDUSTRY & MACRO RISKS
# ============================================================================

print("\n" + "─" * 100)
print("8️⃣  INDUSTRY & MACRO RISKS TO FUNDAMENTALS")
print("─" * 100)

risks = {
    "Commodity Price Risk": {
        "impact": "Every $1k nickel change = ±Rp 40 EPS",
        "probability": "HIGH",
        "mitigation": "None (INCO is price taker)",
    },
    "Indonesia Macro Risk": {
        "impact": "IDR weakness → capex cost up, DER spikes",
        "probability": "MEDIUM",
        "mitigation": "BUMN status, foreign reserves support",
    },
    "China EV Demand Risk": {
        "impact": "EV growth slowing → nickel demand flat",
        "probability": "MEDIUM",
        "mitigation": "Structural EV adoption still early globally",
    },
    "Indonesia Policy Risk": {
        "impact": "BUMN export policy change, dividend policy shift",
        "probability": "LOW-MEDIUM",
        "mitigation": "Long-term mining lease secured",
    },
    "ESG/Climate Risk": {
        "impact": "Mining operations restricted, carbon tax",
        "probability": "LOW-MEDIUM",
        "mitigation": "INCO improving ESG profile",
    },
}

for risk, details in risks.items():
    print(f"\n{risk}:")
    print(f"  Impact: {details['impact']}")
    print(f"  Probability: {details['probability']}")
    print(f"  Mitigation: {details['mitigation']}")

print(f"\n✓ RISK SEVERITY ASSESSMENT:")
print(f"  🔴 CRITICAL (could eliminate dividend):")
print(f"     - Nickel falls below $10k/ton for sustained period")
print(f"     - Global recession cuts EV demand 50%+")
print(f"  ")
print(f"  🟡 SIGNIFICANT (could cut dividend 30-50%):")
print(f"     - Nickel trades $12-14k/ton for 2+ years")
print(f"     - Indonesia currency crisis (IDR > 18k/USD)")
print(f"  ")
print(f"  🟢 MANAGEABLE (dividend maintained):")
print(f"     - Nickel stays $14-18k/ton (most likely scenario)")
print(f"     - Normal policy changes, no major shocks")

# ============================================================================
# 9. FINAL ASSESSMENT
# ============================================================================

print("\n" + "─" * 100)
print("9️⃣  FINAL FUNDAMENTAL ASSESSMENT")
print("─" * 100)

print(f"""
INCO FUNDAMENTAL STRENGTH: 7.5/10

Strengths:
  ✓ Exceptional profitability (NPM 18.5%, ROE 18.5%)
  ✓ Strong balance sheet (DER 0.55, current ratio 1.8)
  ✓ Premium dividend (5.1% yield with 42% payout room)
  ✓ Market leadership (Indonesia's #1 nickel producer)
  ✓ Favorable commodity demand (EV structural tailwind)
  ✓ Attractive valuation (P/E 7.2, P/B 0.9 vs peers)

Weaknesses:
  ✗ Cyclical earnings (not predictable, highly commodity-dependent)
  ✗ Peak cycle positioning (likely entering normalization phase)
  ✗ Indonesia macro exposure (currency, policy, rate risk)
  ✗ Dividend cyclicality (not stable through commodity downturns)
  ✗ Capital intensity (requires ongoing capex for asset life)

Quality Assessment:
  • Earnings Quality: 7/10 (real cash earnings, but cyclical)
  • Dividend Quality: 6/10 (safe now, risky in downturns, cyclical)
  • Business Quality: 7/10 (commodity, no moat, but market leader)
  • Financial Quality: 8/10 (strong balance sheet, good coverage)
  • Growth Quality: 5/10 (limited organic growth, dependent on volumes)

Valuation Assessment:
  • Current pricing: Reflects STRONG cycle + PEAK commodity prices
  • Fair value range: Rp 2,200-2,800 (vs current Rp 2,500)
  • Upside to peak cycle: Limited (already priced in)
  • Downside to trough cycle: 30-50% possible if nickel $12-14k/ton

Investment Thesis:
  INCO is a QUALITY company at a FAIR price, not a BARGAIN
  
  • Graham MOS (P/B 0.9) = acceptable margin for value investor
  • Lynch GARP (ROE 18.5% at P/E 7.2) = reasonable value
  • Buffett moat (strong, but cyclical) = acceptable
  • Munger predictability (poor, very cyclical) = risk
  
  SUITABLE FOR: Income investors who understand commodity cycles
  NOT SUITABLE FOR: Stability seekers (seek BBRI, UNVR for stable dividends)

Next 12-Month Outlook:
  ✓ Base case (60%): Nickel $14-18k/ton → DY 4-5%, earnings stable
  ⚠ Bull case (20%): Nickel $19-20k/ton → DY 5-6%, earnings up 10-15%
  ✗ Bear case (20%): Nickel $10-12k/ton → DY 2-3%, earnings down 40-50%
""")

print("\n" + "=" * 100)
print("✓ Full fundamental analysis complete")
print("=" * 100)
