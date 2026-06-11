#!/usr/bin/env python3
"""
Comparison Script — Enhanced vs Existing 5-Persona System
Run both systems on same stocks and highlight differences
"""

import sys
import subprocess
from pathlib import Path
from typing import Dict, Any
import json

def run_existing_system(tickers: list) -> str:
    """Run existing idx_ai_analyst.py from pagupon profile"""
    pagupon_path = Path("~/.hermes/profiles/pagupon-finance/tools/idx_ai_analyst.py").expanduser()
    
    if not pagupon_path.exists():
        print(f"⚠️ Existing system not found at {pagupon_path}", file=sys.stderr)
        return ""
    
    try:
        cmd = f"cd ~/.hermes/profiles/pagupon-finance && python tools/idx_ai_analyst.py {' '.join(tickers)} 2>/dev/null"
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=60)
        return result.stdout
    except Exception as e:
        print(f"❌ Error running existing system: {e}", file=sys.stderr)
        return ""

def run_enhanced_system(tickers: list) -> str:
    """Run new enhanced system"""
    try:
        cmd = f"python idx_ai_analyst_enhanced.py {' '.join(tickers)} --mock 2>/dev/null"
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=60, cwd=Path(__file__).parent)
        return result.stdout
    except Exception as e:
        print(f"❌ Error running enhanced system: {e}", file=sys.stderr)
        return ""

def extract_signal(output: str, ticker: str) -> Dict[str, Any]:
    """Extract signal from output"""
    lines = output.split("\n")
    
    for i, line in enumerate(lines):
        # Look for ticker lines in both formats
        if ticker in line:
            # Check current and next few lines for signal
            for j in range(i, min(i+3, len(lines))):
                search_line = lines[j]
                
                # Existing system format: "BMRI   │ 3/5   │ 🟡 HOLD    │ MEDIUM"
                if "│" in search_line and ("BUY" in search_line or "HOLD" in search_line or "PASS" in search_line):
                    signal = "UNKNOWN"
                    confidence = "UNKNOWN"
                    
                    if "STRONG BUY" in search_line or "STRONG" in search_line:
                        signal = "STRONG BUY"
                    elif "BUY" in search_line:
                        signal = "BUY"
                    elif "HOLD" in search_line:
                        signal = "HOLD"
                    elif "PASS" in search_line:
                        signal = "PASS"
                    
                    parts = search_line.split("│")
                    if len(parts) > 0:
                        confidence = parts[-1].strip() if parts[-1].strip() else "MEDIUM"
                    
                    return {"signal": signal, "confidence": confidence}
                
                # Enhanced system format: "**Signal:** BUY | **Confidence:** MEDIUM"
                if "**Signal:**" in search_line and ("BUY" in search_line or "HOLD" in search_line or "PASS" in search_line):
                    signal = "UNKNOWN"
                    confidence = "UNKNOWN"
                    
                    if "STRONG BUY" in search_line:
                        signal = "STRONG BUY"
                    elif "BUY" in search_line:
                        signal = "BUY"
                    elif "HOLD" in search_line:
                        signal = "HOLD"
                    elif "PASS" in search_line:
                        signal = "PASS"
                    
                    if "**Confidence:**" in search_line:
                        conf_part = search_line.split("**Confidence:**")[1].strip()
                        confidence = conf_part.split()[0] if conf_part else "UNKNOWN"
                    
                    return {"signal": signal, "confidence": confidence}
    
    return {"signal": "NOT_FOUND", "confidence": "N/A"}

def compare(tickers: list):
    """Run comparison"""
    print(f"\n🔄 Running comparison on: {', '.join(tickers)}\n")
    
    print("=" * 70)
    print("EXISTING SYSTEM (5-Persona)")
    print("=" * 70)
    existing_output = run_existing_system(tickers)
    print(existing_output[:500] + "..." if len(existing_output) > 500 else existing_output)
    
    print("\n" + "=" * 70)
    print("ENHANCED SYSTEM (Debate + Risk + Memory)")
    print("=" * 70)
    enhanced_output = run_enhanced_system(tickers)
    print(enhanced_output[:500] + "..." if len(enhanced_output) > 500 else enhanced_output)
    
    # Extract and compare signals
    print("\n" + "=" * 70)
    print("SIGNAL COMPARISON")
    print("=" * 70)
    
    print(f"\n{'Ticker':<10} {'Existing Signal':<25} {'Enhanced Signal':<25} {'Match?':<10}")
    print("-" * 70)
    
    signal_matches = 0
    signal_mismatches = 0
    
    for ticker in tickers:
        existing_sig = extract_signal(existing_output, ticker)
        enhanced_sig = extract_signal(enhanced_output, ticker)
        
        match = "✅" if existing_sig["signal"] == enhanced_sig["signal"] else "❌"
        
        print(f"{ticker:<10} {existing_sig['signal']:<25} {enhanced_sig['signal']:<25} {match:<10}")
        
        if existing_sig["signal"] == enhanced_sig["signal"]:
            signal_matches += 1
        else:
            signal_mismatches += 1
    
    print("-" * 70)
    print(f"Match Rate: {signal_matches}/{len(tickers)} ({signal_matches*100//len(tickers)}%)")
    
    if signal_mismatches > 0:
        print(f"\n⚠️ Signal Differences: {signal_mismatches} stocks")
        print("\nPossible Reasons:")
        print("• Debate mechanism may weight criteria differently")
        print("• Risk assessment might flag stocks existing system approves")
        print("• Mock data causes both to behave similarly — need real Notion data")
    
    print("\n" + "=" * 70)
    print("RECOMMENDATION")
    print("=" * 70)
    
    if signal_matches == len(tickers):
        print("✅ Systems aligned — debate mechanism validates existing approach")
        print("   Next: Test with real Notion data to see if debate adds value")
    else:
        print("⚠️ Signal differences detected — requires deeper analysis")
        print("   Action: Review mismatches and adjust debate thresholds")

def main():
    """Entry point"""
    if "--help" in sys.argv or "-h" in sys.argv:
        print("Usage: python compare_systems.py [TICKERS...]")
        print("Example: python compare_systems.py BMRI KLBF BBRI")
        sys.exit(0)
    
    # Parse tickers
    tickers = [arg for arg in sys.argv[1:] if not arg.startswith("--")]
    if not tickers:
        tickers = ["BMRI", "KLBF", "BBRI", "TLKM", "PTBA"]  # Default comparison set
    
    compare(tickers)

if __name__ == "__main__":
    main()
